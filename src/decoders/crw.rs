use std::f32::NAN;

use crate::decoders::*;
use crate::decoders::ciff::*;
use crate::decoders::basics::*;
use crate::decoders::ljpeg::huffman::*;

const CRW_FIRST_TREE: [[u8;29];3] = [
  [ 0,1,4,2,3,1,2,0,0,0,0,0,0,0,0,0,
    0x04,0x03,0x05,0x06,0x02,0x07,0x01,0x08,0x09,0x00,0x0a,0x0b,0xff ],
  [ 0,2,2,3,1,1,1,1,2,0,0,0,0,0,0,0,
    0x03,0x02,0x04,0x01,0x05,0x00,0x06,0x07,0x09,0x08,0x0a,0x0b,0xff ],
  [ 0,0,6,3,1,1,2,0,0,0,0,0,0,0,0,0,
    0x06,0x05,0x07,0x04,0x08,0x03,0x09,0x02,0x00,0x0a,0x01,0x0b,0xff ],
];

const CRW_SECOND_TREE: [[u8;180];3] = [
  [ 0,2,2,2,1,4,2,1,2,5,1,1,0,0,0,139,
    0x03,0x04,0x02,0x05,0x01,0x06,0x07,0x08,
    0x12,0x13,0x11,0x14,0x09,0x15,0x22,0x00,0x21,0x16,0x0a,0xf0,
    0x23,0x17,0x24,0x31,0x32,0x18,0x19,0x33,0x25,0x41,0x34,0x42,
    0x35,0x51,0x36,0x37,0x38,0x29,0x79,0x26,0x1a,0x39,0x56,0x57,
    0x28,0x27,0x52,0x55,0x58,0x43,0x76,0x59,0x77,0x54,0x61,0xf9,
    0x71,0x78,0x75,0x96,0x97,0x49,0xb7,0x53,0xd7,0x74,0xb6,0x98,
    0x47,0x48,0x95,0x69,0x99,0x91,0xfa,0xb8,0x68,0xb5,0xb9,0xd6,
    0xf7,0xd8,0x67,0x46,0x45,0x94,0x89,0xf8,0x81,0xd5,0xf6,0xb4,
    0x88,0xb1,0x2a,0x44,0x72,0xd9,0x87,0x66,0xd4,0xf5,0x3a,0xa7,
    0x73,0xa9,0xa8,0x86,0x62,0xc7,0x65,0xc8,0xc9,0xa1,0xf4,0xd1,
    0xe9,0x5a,0x92,0x85,0xa6,0xe7,0x93,0xe8,0xc1,0xc6,0x7a,0x64,
    0xe1,0x4a,0x6a,0xe6,0xb3,0xf1,0xd3,0xa5,0x8a,0xb2,0x9a,0xba,
    0x84,0xa4,0x63,0xe5,0xc5,0xf3,0xd2,0xc4,0x82,0xaa,0xda,0xe4,
    0xf2,0xca,0x83,0xa3,0xa2,0xc3,0xea,0xc2,0xe2,0xe3,0xff,0xff ],
  [ 0,2,2,1,4,1,4,1,3,3,1,0,0,0,0,140,
    0x02,0x03,0x01,0x04,0x05,0x12,0x11,0x06,
    0x13,0x07,0x08,0x14,0x22,0x09,0x21,0x00,0x23,0x15,0x31,0x32,
    0x0a,0x16,0xf0,0x24,0x33,0x41,0x42,0x19,0x17,0x25,0x18,0x51,
    0x34,0x43,0x52,0x29,0x35,0x61,0x39,0x71,0x62,0x36,0x53,0x26,
    0x38,0x1a,0x37,0x81,0x27,0x91,0x79,0x55,0x45,0x28,0x72,0x59,
    0xa1,0xb1,0x44,0x69,0x54,0x58,0xd1,0xfa,0x57,0xe1,0xf1,0xb9,
    0x49,0x47,0x63,0x6a,0xf9,0x56,0x46,0xa8,0x2a,0x4a,0x78,0x99,
    0x3a,0x75,0x74,0x86,0x65,0xc1,0x76,0xb6,0x96,0xd6,0x89,0x85,
    0xc9,0xf5,0x95,0xb4,0xc7,0xf7,0x8a,0x97,0xb8,0x73,0xb7,0xd8,
    0xd9,0x87,0xa7,0x7a,0x48,0x82,0x84,0xea,0xf4,0xa6,0xc5,0x5a,
    0x94,0xa4,0xc6,0x92,0xc3,0x68,0xb5,0xc8,0xe4,0xe5,0xe6,0xe9,
    0xa2,0xa3,0xe3,0xc2,0x66,0x67,0x93,0xaa,0xd4,0xd5,0xe7,0xf8,
    0x88,0x9a,0xd7,0x77,0xc4,0x64,0xe2,0x98,0xa5,0xca,0xda,0xe8,
    0xf3,0xf6,0xa9,0xb2,0xb3,0xf2,0xd2,0x83,0xba,0xd3,0xff,0xff ],
  [ 0,0,6,2,1,3,3,2,5,1,2,2,8,10,0,117,
    0x04,0x05,0x03,0x06,0x02,0x07,0x01,0x08,
    0x09,0x12,0x13,0x14,0x11,0x15,0x0a,0x16,0x17,0xf0,0x00,0x22,
    0x21,0x18,0x23,0x19,0x24,0x32,0x31,0x25,0x33,0x38,0x37,0x34,
    0x35,0x36,0x39,0x79,0x57,0x58,0x59,0x28,0x56,0x78,0x27,0x41,
    0x29,0x77,0x26,0x42,0x76,0x99,0x1a,0x55,0x98,0x97,0xf9,0x48,
    0x54,0x96,0x89,0x47,0xb7,0x49,0xfa,0x75,0x68,0xb6,0x67,0x69,
    0xb9,0xb8,0xd8,0x52,0xd7,0x88,0xb5,0x74,0x51,0x46,0xd9,0xf8,
    0x3a,0xd6,0x87,0x45,0x7a,0x95,0xd5,0xf6,0x86,0xb4,0xa9,0x94,
    0x53,0x2a,0xa8,0x43,0xf5,0xf7,0xd4,0x66,0xa7,0x5a,0x44,0x8a,
    0xc9,0xe8,0xc8,0xe7,0x9a,0x6a,0x73,0x4a,0x61,0xc7,0xf4,0xc6,
    0x65,0xe9,0x72,0xe6,0x71,0x91,0x93,0xa6,0xda,0x92,0x85,0x62,
    0xf3,0xc5,0xb2,0xa4,0x84,0xba,0x64,0xa5,0xb3,0xd2,0x81,0xe5,
    0xd3,0xaa,0xc4,0xca,0xf2,0xb1,0xe4,0xd1,0x83,0x63,0xea,0xc3,
    0xe2,0x82,0xf1,0xa3,0xc2,0xa1,0xc1,0xe3,0xa2,0xe1,0xff,0xff ]
];

#[derive(Debug, Clone)]
pub struct CrwDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  ciff: CiffIFD<'a>,
}

impl<'a> CrwDecoder<'a> {
  pub fn new(buf: &'a [u8], ciff: CiffIFD<'a>, rawhide: &'a RawHide) -> CrwDecoder<'a> {
    CrwDecoder {
      buffer: buf,
      ciff: ciff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for CrwDecoder<'a> {
  fn image(&self, dummy: bool) -> Result<RawImage,String> {
    let makemodel = fetch_tag!(self.ciff, CiffTag::MakeModel).get_strings();
    if makemodel.len() < 2 {
      return Err("CRW: MakeModel tag needs to have 2 strings".to_string())
    }
    let camera = self.rawhide.check_supported_with_everything(&makemodel[0], &makemodel[1], "")?;

    let (width, height, image) = if camera.model == "Canon PowerShot Pro70" {
      (1552,1024,decode_10le_lsb16(&self.buffer[26..], 1552, 1024, dummy))
    } else {
      let sensorinfo = fetch_tag!(self.ciff, CiffTag::SensorInfo);
      let width = sensorinfo.get_usize(1);
      let height = sensorinfo.get_usize(2);
      (width, height, self.decode_compressed(&camera, width, height, dummy)?)
    };

    let wb = self.get_wb(&camera)?;
    ok_image(camera, width, height, wb, image)
  }
}

impl<'a> CrwDecoder<'a> {
  fn get_wb(&self, cam: &Camera) -> Result<[f32;4], String> {
    if let Some(levels) = self.ciff.find_entry(CiffTag::WhiteBalance) {
      let offset = cam.wb_offset;
      return Ok([levels.get_f32(offset+0), levels.get_f32(offset+1), levels.get_f32(offset+3), NAN])
    }
    if !cam.find_hint("nocinfo2") {
      if let Some(cinfo) = self.ciff.find_entry(CiffTag::ColorInfo2) {
        return Ok(if cinfo.get_u32(0) > 512 {
          [cinfo.get_f32(62), cinfo.get_f32(63), cinfo.get_f32(60), cinfo.get_f32(61)]
        } else {
          [cinfo.get_f32(51), (cinfo.get_f32(50)+cinfo.get_f32(53))/2.0, cinfo.get_f32(52), NAN]
        })
      }
    }
    if let Some(cinfo) = self.ciff.find_entry(CiffTag::ColorInfo1) {
      if cinfo.count == 768 { // D30
        return Ok([1024.0/(cinfo.get_force_u16(36) as f32),
                   1024.0/(cinfo.get_force_u16(37) as f32),
                   1024.0/(cinfo.get_force_u16(39) as f32),
                   NAN])
      }
      let off = cam.wb_offset;
      let key: [u16;2] = if cam.find_hint("wb_mangle") {[0x410, 0x45f3]} else {[0,0]};
      return Ok([(cinfo.get_force_u16(off+1)^key[1]) as f32,
                 (cinfo.get_force_u16(off+0)^key[0]) as f32,
                 (cinfo.get_force_u16(off+2)^key[0]) as f32, NAN])
    }
    Ok([NAN,NAN,NAN,NAN])
  }

  fn create_hufftables(num: usize) -> [HuffTable;2] {
    [
      Self::create_hufftable(&CRW_FIRST_TREE[num]),
      Self::create_hufftable(&CRW_SECOND_TREE[num]),
    ]
  }

  fn create_hufftable(table: &[u8]) -> HuffTable {
    let mut htable = HuffTable::empty();

    for i in 0..16 {
      htable.bits[i+1] = table[i] as u32;
    }
    for i in 16..table.len() {
      htable.huffval[i-16] = table[i] as u32;
    }

    htable.disable_cache = true;
    htable.initialize().unwrap();
    htable
  }

  fn decode_compressed(&self, cam: &Camera, width: usize, height: usize, dummy: bool) -> Result<Vec<u16>,String> {
    let lowbits = !cam.find_hint("nolowbits");
    let dectable = fetch_tag!(self.ciff, CiffTag::DecoderTable).get_usize(0);
    if dectable > 2 {
      return Err(format!("CRW: Unknown decoder table {}", dectable).to_string())
    }
    Ok(Self::do_decode(&self.buffer, lowbits, dectable, width, height, dummy))
  }

  pub(crate) fn do_decode(buffer: &[u8], lowbits: bool, dectable: usize, width: usize, height: usize, dummy: bool) -> Vec<u16> {
    let mut out = alloc_image!(width, height, dummy);

    let htables = Self::create_hufftables(dectable);
    let offset = 540 + (lowbits as usize)*height*width/4;
    let mut pump = BitPumpJPEG::new(&buffer[offset..]);

    let mut carry: i32 = 0;
    let mut base = [0 as i32;2];
    let mut pnum = 0;
    for pixout in out.chunks_exact_mut(64) {
      // Decode a block of 64 differences
      let mut diffbuf = [0 as i32; 64];
      let mut i: usize = 0;
      while i < 64 {
        let ref tbl = htables[(i > 0) as usize];
        let leaf = tbl.huff_len(&mut pump).1 as u32;
        if leaf == 0 && i != 0 { break; }
        if leaf == 0xff { i+= 1; continue; }
        i += (leaf >> 4) as usize;
        let len = leaf & 0x0f;
        if len == 0 { i+= 1; continue; }
        let mut diff: i32 = pump.get_bits(len) as i32;
        if (diff & (1 << (len-1))) == 0 {
          diff -= (1 << len) - 1;
        }
        if i < 64 {
          diffbuf[i] = diff;
        }
        i += 1;
      }
      diffbuf[0] += carry;
      carry = diffbuf[0];

      // Save those differences to 64 pixels adjusting the predictor as we go
      for i in 0..64 {
        // At the start of lines reset the predictor to 512
        if pnum % width == 0 {
          base[0] = 512;
          base[1] = 512;
        }
        pnum += 1;
        base[i & 1] += diffbuf[i];
        pixout[i] = base[i & 1] as u16;
      }
    }

    if lowbits {
      // Add the uncompressed 2 low bits to the decoded 8 high bits
      for (i,o) in out.chunks_exact_mut(4).enumerate() {
        let c = buffer[26+i] as u16;
        o[0] = o[0] << 2 | (c     ) & 0x03;
        o[1] = o[1] << 2 | (c >> 2) & 0x03;
        o[2] = o[2] << 2 | (c >> 4) & 0x03;
        o[3] = o[3] << 2 | (c >> 6) & 0x03;
        if width == 2672 {
          // No idea why this is needed, probably some broken camera
          if o[0] < 512 { o[0] += 2}
          if o[1] < 512 { o[1] += 2}
          if o[2] < 512 { o[2] += 2}
          if o[3] < 512 { o[3] += 2}
        }
      }
    }
    out
  }
}
