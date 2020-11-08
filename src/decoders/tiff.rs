use decoders::basics::*;
use std::collections::HashMap;
use std::str;

#[derive(Debug, Copy, Clone)]
pub enum Tag {
    MAKE = 0x010F,
    MODEL = 0x0110,
    SUBIFDS = 0x014A,
    EXIFIFDPOINTER = 0x8769,
}

// 0-1-2-3-4-5-6-7-8-9-10-11-12-13
const DATASHIFTS: [u8; 14] = [0, 0, 0, 1, 2, 3, 0, 0, 1, 2, 3, 2, 3, 2];

fn t(tag: Tag) -> u16 {
    tag as u16
}

pub struct TiffEntry<'a> {
    tag: u16,
    typ: u16,
    count: u32,
    data: &'a [u8],
    endian: Endian,
}

pub struct TiffIFD<'a> {
    entries: HashMap<u16, TiffEntry<'a>>,
    subifds: Vec<TiffIFD<'a>>,
    nextifd: usize,
    endian: Endian,
}

impl<'a> TiffIFD<'a> {
    pub fn new_root(buf: &'a [u8], offset: usize, depth: u32, e: Endian) -> TiffIFD<'a> {
        let mut subifds = Vec::new();
        let mut nextifd = e.ru32(buf, offset) as usize;

        for _ in 0..100 {
            // Never read more than 100 IFDs
            let ifd = TiffIFD::new(buf, nextifd, depth, e);
            nextifd = ifd.nextifd;
            subifds.push(ifd);
            if nextifd == 0 {
                break;
            }
        }

        TiffIFD {
            entries: HashMap::new(),
            subifds: subifds,
            nextifd: 0,
            endian: e,
        }
    }

    pub fn new(buf: &'a [u8], offset: usize, depth: u32, e: Endian) -> TiffIFD<'a> {
        let mut entries = HashMap::new();
        let mut subifds = Vec::new();

        let num = e.ru16(buf, offset); // Directory entries in this IFD
        for i in 0..num {
            let entry_offset: usize = offset + 2 + (i as usize) * 12;
            let entry = TiffEntry::new(buf, entry_offset, e);

            if entry.tag == t(Tag::SUBIFDS) || entry.tag == t(Tag::EXIFIFDPOINTER) {
                if depth < 10 {
                    // Avoid infinite looping IFDs
                    for i in 0..entry.count {
                        subifds.push(TiffIFD::new(buf, entry.get_u32(i) as usize, depth + 1, e));
                    }
                }
            } else {
                entries.insert(entry.tag, entry);
            }
        }

        TiffIFD {
            entries: entries,
            subifds: subifds,
            nextifd: e.ru32(buf, offset + (2 + num * 12) as usize) as usize,
            endian: e,
        }
    }

    pub fn find_entry(&self, tag: Tag) -> Option<&TiffEntry> {
        let utag: u16 = tag as u16;
        if self.entries.contains_key(&utag) {
            self.entries.get(&utag)
        } else {
            for ifd in &self.subifds {
                match ifd.find_entry(tag) {
                    Some(x) => return Some(x),
                    None => {}
                }
            }
            None
        }
    }
}

impl<'a> TiffEntry<'a> {
    pub fn new(buf: &'a [u8], offset: usize, e: Endian) -> TiffEntry<'a> {
        let tag = e.ru16(buf, offset);
        let mut typ = e.ru16(buf, offset + 2);
        let count = e.ru32(buf, offset + 4);

        // If we don't know the type assume byte data
        if typ == 0 || typ > 13 {
            typ = 1;
        }

        let bytesize: usize = (count as usize) << DATASHIFTS[typ as usize];
        let doffset: usize = if bytesize <= 4 {
            (offset + 8)
        } else {
            e.ru32(buf, offset + 8) as usize
        };

        TiffEntry {
            tag: tag,
            typ: typ,
            count: count,
            data: &buf[doffset..doffset + bytesize],
            endian: e,
        }
    }

    pub fn get_u32(&self, idx: u32) -> u32 {
        self.endian.ru32(self.data, (idx * 4) as usize)
    }

    pub fn get_str(&self) -> &str {
        match str::from_utf8(&self.data[0..self.data.len() - 1]) {
            Result::Ok(val) => val.trim(),
            Result::Err(err) => panic!(err),
        }
    }
}
