use std::collections::HashMap;
use decoders::basics::*;
use std::str;
use num::FromPrimitive;

enum_from_primitive! {
  #[derive(Debug, Copy, Clone, PartialEq)]
  pub enum Tag {
    PanaWidth        = 0x0002,
    PanaLength       = 0x0003,
    PanaWBsR         = 0x0011,
    PanaWBsB         = 0x0012,
    PanaWBs2R        = 0x0024,
    PanaWBs2G        = 0x0025,
    PanaWBs2B        = 0x0026,
    NewSubFileType   = 0x00FE,
    ImageWidth       = 0x0100,
    ImageLength      = 0x0101,
    BitsPerSample    = 0x0102,
    Compression      = 0x0103,
    Make             = 0x010F,
    Model            = 0x0110,
    StripOffsets     = 0x0111,
    StripByteCounts  = 0x0117,
    PanaOffsets      = 0x0118,
    GrayResponse     = 0x0123,
    SubIFDs          = 0x014A,
    DcrWB            = 0x03FD,
    DcrLinearization = 0x090D,
    EpsonWB          = 0x0E80,
    KodakWB          = 0x0F00,
    OlympusRedMul    = 0x1017,
    OlympusBlueMul   = 0x1018,
    OlympusImgProc   = 0x2040,
    RafOldWB         = 0x2ff0,
    SonyCurve        = 0x7010,
    SonyOffset       = 0x7200,
    SonyLength       = 0x7201,
    SonyKey          = 0x7221,
    SonyGRBG         = 0x7303,
    SonyRGGB         = 0x7313,
    CFAPattern       = 0x828E,
    KodakIFD         = 0x8290,
    ExifIFDPointer   = 0x8769,
    Makernote        = 0x927C,
    SrwSensorAreas   = 0xA010,
    SrwRGGBLevels    = 0xA021,
    SrwRGGBBlacks    = 0xA028,
    DNGVersion       = 0xC612,
    AsShotNeutral    = 0xC628,
    DNGPrivateArea   = 0xC634,
    RafRawSubIFD     = 0xF000,
    RafImageWidth    = 0xF001,
    RafImageLength   = 0xF002,
    RafBitsPerSample = 0xF003,
    RafOffsets       = 0xF007,
    RafWBGRB         = 0xF00E,
    KdcWB            = 0xFA2A,
    KdcWidth         = 0xFD00,
    KdcLength        = 0xFD01,
    KdcOffset        = 0xFD04,
    KdcIFD           = 0xFE00,
  }
}
                          // 0-1-2-3-4-5-6-7-8-9-10-11-12-13
const DATASHIFTS: [u8;14] = [0,0,0,1,2,3,0,0,1,2, 3, 2, 3, 2];

fn t (tag: Tag) -> u16 {
  tag as u16
}

#[derive(Debug, Copy, Clone)]
pub struct TiffEntry<'a> {
  tag: u16,
  typ: u16,
  count: u32,
  parent_offset: usize,
  doffset: usize,
  data: &'a [u8],
  endian: Endian,
}

#[derive(Debug, Clone)]
pub struct TiffIFD<'a> {
  entries: HashMap<u16,TiffEntry<'a>>,
  subifds: Vec<TiffIFD<'a>>,
  nextifd: usize,
  start_offset: usize,
  endian: Endian,
}

impl<'a> TiffIFD<'a> {
  pub fn new_file(buf: &'a[u8]) -> Result<TiffIFD<'a>, String> {
    if buf[0..8] == b"FUJIFILM"[..] {
      let ifd1 = TiffIFD::new_root(buf, (BEu32(buf, 84)+12) as usize)?;
      let endian = ifd1.get_endian();
      let mut subifds = vec![ifd1];
      let mut entries = HashMap::new();

      let ioffset = BEu32(buf, 100) as usize;
      match TiffIFD::new_root(buf, ioffset) {
        Ok(val) => {subifds.push(val);}
        Err(_) => {
          entries.insert(Tag::RafOffsets as u16, TiffEntry{
            tag: t(Tag::RafOffsets),
            typ: 4, // Long
            count: 1,
            parent_offset: 0,
            doffset: 100,
            data: &buf[100..104],
            endian: BIG_ENDIAN,
          });
        },
      }
      match TiffIFD::new_fuji(buf, BEu32(buf, 92) as usize) {
        Ok(val) => subifds.push(val),
        Err(_) => {}
      }

      Ok(TiffIFD {
        entries: entries,
        subifds: subifds,
        nextifd: 0,
        start_offset: 0,
        endian: endian,
      })
    } else {
      TiffIFD::new_root(buf, 0)
    }
  }

  pub fn new_root(buf: &'a[u8], offset: usize) -> Result<TiffIFD<'a>, String> {
    let mut subifds = Vec::new();

    let endian = match LEu16(buf, offset) {
      0x4949 => LITTLE_ENDIAN,
      0x4d4d => BIG_ENDIAN,
      x => {return Err(format!("TIFF: don't know marker 0x{:x}", x).to_string())},
    };
    let mut nextifd = endian.ru32(buf, offset+4) as usize;
    for _ in 0..100 { // Never read more than 100 IFDs
      let ifd = TiffIFD::new(&buf[offset..], nextifd, 0, offset, 0, endian)?;
      nextifd = ifd.nextifd;
      subifds.push(ifd);
      if nextifd == 0 {
        break
      }
    }

    Ok(TiffIFD {
      entries: HashMap::new(),
      subifds: subifds,
      nextifd: 0,
      start_offset: offset,
      endian: endian,
    })
  }

  pub fn new(buf: &'a[u8], offset: usize, base_offset: usize, start_offset: usize, depth: u32, e: Endian) -> Result<TiffIFD<'a>, String> {
    let mut entries = HashMap::new();
    let mut subifds = Vec::new();

    let num = e.ru16(buf, offset); // Directory entries in this IFD
    if num > 4000 {
      return Err(format!("too many entries in IFD ({})", num).to_string())
    }
    for i in 0..num {
      let entry_offset: usize = offset + 2 + (i as usize)*12;
      if Tag::from_u16(e.ru16(buf, entry_offset)).is_none() {
        // Skip entries we don't know about to speedup decoding
        continue;
      }
      let entry = TiffEntry::new(buf, entry_offset, base_offset, offset, e);

      if entry.tag == t(Tag::SubIFDs)
      || entry.tag == t(Tag::ExifIFDPointer)
      || entry.tag == t(Tag::RafRawSubIFD)
      || entry.tag == t(Tag::KodakIFD)
      || entry.tag == t(Tag::KdcIFD) {
        if depth < 10 { // Avoid infinite looping IFDs
          for i in 0..entry.count {
            let ifd = TiffIFD::new(buf, entry.get_u32(i as usize) as usize, base_offset, start_offset, depth+1, e);
            match ifd {
              Ok(val) => {subifds.push(val);},
              Err(_) => {entries.insert(entry.tag, entry);}, // Ignore unparsable IFDs
            }
          }
        }
      } else if entry.tag == t(Tag::Makernote) {
        if depth < 10 { // Avoid infinite looping IFDs
          let ifd = TiffIFD::new_makernote(buf, entry.doffset(), base_offset, depth+1, e);
          match ifd {
            Ok(val) => {subifds.push(val);},
            Err(_) => {entries.insert(entry.tag, entry);}, // Ignore unparsable IFDs
          }
        }
      } else {
        entries.insert(entry.tag, entry);
      }
    }

    Ok(TiffIFD {
      entries: entries,
      subifds: subifds,
      nextifd: e.ru32(buf, offset + (2+num*12) as usize) as usize,
      start_offset: start_offset,
      endian: e,
    })
  }

  pub fn new_makernote(buf: &'a[u8], offset: usize, base_offset: usize, depth: u32, e: Endian) -> Result<TiffIFD<'a>, String> {
    let mut off = offset;
    let data = &buf[offset..];

    // Olympus starts the makernote with their own name, sometimes truncated
    if data[0..5] == b"OLYMP"[..] {
      off += 8;
      if data[0..7] == b"OLYMPUS"[..] {
        off += 4;
      }
    }

    // Epson starts the makernote with its own name
    if data[0..5] == b"EPSON"[..] {
      off += 8;
    }

    TiffIFD::new(buf, off, base_offset, 0, depth, e)
  }

  pub fn new_fuji(buf: &'a[u8], offset: usize) -> Result<TiffIFD<'a>, String> {
    let mut entries = HashMap::new();
    let num = BEu32(buf, offset); // Directory entries in this IFD
    if num > 4000 {
      return Err(format!("too many entries in IFD ({})", num).to_string())
    }
    let mut off = offset+4;
    for _ in 0..num {
      let tag = BEu16(buf, off);
      let len = BEu16(buf, off+2);
      if tag == t(Tag::ImageWidth) {
        entries.insert(t(Tag::ImageWidth), TiffEntry {
          tag: t(Tag::ImageWidth),
          typ: 3, // Short
          count: 2,
          parent_offset: 0,
          doffset: off+4,
          data: &buf[off+4..off+8],
          endian: BIG_ENDIAN,
        });
      } else if tag == t(Tag::RafOldWB) {
        entries.insert(t(Tag::RafOldWB), TiffEntry {
          tag: t(Tag::RafOldWB),
          typ: 3, // Short
          count: 4,
          parent_offset: 0,
          doffset: off+4,
          data: &buf[off+4..off+12],
          endian: BIG_ENDIAN,
        });
      }
      off += (len+4) as usize;
    }

    Ok(TiffIFD {
      entries: entries,
      subifds: Vec::new(),
      nextifd: 0,
      start_offset: 0,
      endian: BIG_ENDIAN,
    })
  }

  pub fn find_entry(&self, tag: Tag) -> Option<&TiffEntry> {
    if self.entries.contains_key(&t(tag)) {
      self.entries.get(&t(tag))
    } else {
      for ifd in &self.subifds {
        match ifd.find_entry(tag) {
          Some(x) => return Some(x),
          None => {},
        }
      }
      None
    }
  }

  pub fn has_entry(&self, tag: Tag) -> bool {
    self.find_entry(tag).is_some()
  }

  pub fn find_ifds_with_tag(&self, tag: Tag) -> Vec<&TiffIFD> {
    let mut ifds = Vec::new();
    if self.entries.contains_key(&t(tag)) {
      ifds.push(self);
    }
    for ifd in &self.subifds {
      if ifd.entries.contains_key(&t(tag)) {
        ifds.push(ifd);
      }
      ifds.extend(ifd.find_ifds_with_tag(tag));
    }
    ifds
  }

  pub fn find_first_ifd(&self, tag: Tag) -> Option<&TiffIFD> {
    let ifds = self.find_ifds_with_tag(tag);
    if ifds.len() == 0 {
      None
    } else {
      Some(ifds[0])
    }
  }

  pub fn get_endian(&self) -> Endian { self.endian }
  pub fn little_endian(&self) -> bool { self.endian.little() }
  pub fn start_offset(&self) -> usize { self.start_offset }
}

impl<'a> TiffEntry<'a> {
  pub fn new(buf: &'a[u8], offset: usize, base_offset: usize, parent_offset: usize, e: Endian) -> TiffEntry<'a> {
    let tag = e.ru16(buf, offset);
    let mut typ = e.ru16(buf, offset+2);
    let count = e.ru32(buf, offset+4);

    // If we don't know the type assume byte data
    if typ == 0 || typ > 13 {
      typ = 1;
    }

    let bytesize: usize = (count as usize) << DATASHIFTS[typ as usize];
    let doffset: usize = if bytesize <= 4 {
      offset + 8
    } else {
      (e.ru32(buf, offset+8) as usize) - base_offset
    };

    TiffEntry {
      tag: tag,
      typ: typ,
      count: count,
      parent_offset: parent_offset,
      doffset: doffset,
      data: &buf[doffset .. doffset+bytesize],
      endian: e,
    }
  }

  pub fn copy_with_new_data(&self, data: &'a[u8]) -> TiffEntry<'a> {
    let mut copy = self.clone();
    copy.data = data;
    copy
  }

  pub fn copy_offset_from_parent(&self, buffer: &'a[u8]) -> TiffEntry<'a> {
    self.copy_with_new_data(&buffer[self.parent_offset+self.doffset..])
  }

  pub fn doffset(&self) -> usize { self.doffset }
  pub fn parent_offset(&self) -> usize { self.parent_offset }
  pub fn count(&self) -> u32 { self.count }
  //pub fn typ(&self) -> u16 { self.typ }

  pub fn get_u32(&self, idx: usize) -> u32 {
    match self.typ {
      3 | 8              => self.endian.ru16(self.data, idx*2) as u32,
      1 | 4 | 7 | 9 | 13 => self.endian.ru32(self.data, idx*4),
      _ => panic!(format!("Trying to read typ {} for a u32", self.typ).to_string()),
    }
  }

  pub fn get_f32(&self, idx: usize) -> f32 {
    if self.typ == 5 { // Rational
      let a = self.endian.ru32(self.data, idx*8) as f32;
      let b = self.endian.ru32(self.data, idx*8+4) as f32;
      a / b
    } else {
      self.get_u32(idx) as f32
    }
  }

  pub fn get_str(&self) -> &str {
    // Truncate the string when there are \0 bytes
    let len = match self.data.iter().position(|&x| x == 0) {
      Some(p) => p,
      None => self.data.len(),
    };
    match str::from_utf8(&self.data[0..len]) {
      Result::Ok(val) => val.trim(),
      Result::Err(err) => panic!(err),
    }
  }

  pub fn get_data(&self) -> &[u8] {
    self.data
  }
}
