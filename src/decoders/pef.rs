use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use decoders::ljpeg::huffman::*;
use std::f32::NAN;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct PefDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> PefDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> PefDecoder<'a> {
    PefDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for PefDecoder<'a> {
  fn image(&self) -> Result<Image,String> {
    let camera = self.rawhide.check_supported(&self.tiff)?;
    let raw = fetch_ifd!(&self.tiff, Tag::StripOffsets);
    let width = fetch_tag!(raw, Tag::ImageWidth).get_u32(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_u32(0);
    let offset = fetch_tag!(raw, Tag::StripOffsets).get_u32(0) as usize;
    let src = &self.buffer[offset .. self.buffer.len()];

    let image = match fetch_tag!(raw, Tag::Compression).get_u32(0) {
      1 => decode_16be(src, width as usize, height as usize),
      32773 => decode_12be(src, width as usize, height as usize),
      65535 => self.decode_compressed(src, width as usize, height as usize)?,
      c => return Err(format!("PEF: Don't know how to read compression {}", c).to_string()),
    };

    let blacklevels = self.get_blacklevels().unwrap_or(camera.blacklevels);
    ok_image_with_blacklevels(camera, width, height, self.get_wb()?, blacklevels, image)
  }
}

impl<'a> PefDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    let levels = fetch_tag!(self.tiff, Tag::PefWB);
    Ok([levels.get_f32(0), levels.get_f32(1), levels.get_f32(3), NAN])
  }

  fn get_blacklevels(&self) -> Option<[u16;4]> {
    match self.tiff.find_entry(Tag::PefBlackLevels) {
      Some(levels) => {
        Some([levels.get_f32(0) as u16,levels.get_f32(1) as u16,
             levels.get_f32(2) as u16,levels.get_f32(3) as u16])
      },
      None => None,
    }
  }

  fn decode_compressed(&self, src: &[u8], width: usize, height: usize) -> Result<Vec<u16>,String> {
    let mut htable = HuffTable::empty(16);
    htable.dng_compatible = false;

    /* Attempt to read huffman table, if found in makernote */
    if self.tiff.has_entry(Tag::PefHuffman) {
      let huff = fetch_tag!(self.tiff, Tag::PefHuffman);
      let mut stream = ByteStream::new(huff.get_data(), BIG_ENDIAN);

      let depth: usize = (stream.get_u16() as usize + 12) & 0xf;
      stream.consume_bytes(12);

      let mut v0: [u32;16] = [0;16];
      for i in 0..depth {
        v0[i] = stream.get_u16() as u32;
      }

      let mut v1: [u32;16] = [0;16];
      for i in 0..depth {
        v1[i] = stream.get_u8() as u32;
      }

      // Calculate codes and store bitcounts
      let mut v2: [u32;16] = [0;16];
      for c in 0..depth {
        v2[c] = v0[c] >> (12 - v1[c]);
        htable.bits[v1[c] as usize] += 1;
      }

      // Find smallest
      for i in 0..depth {
        let mut sm_val: u32 = 0xfffffff;
        let mut sm_num: u32 = 0xff;
        for j in 0..depth {
          if v2[j] <= sm_val {
            sm_num = j as u32;
            sm_val = v2[j];
          }
        }
        htable.huffval[i] = sm_num;
        v2[sm_num as usize]=0xffffffff;
      }
    } else {
      // Initialize with legacy data
      let pentax_tree: [u8; 29] = [ 0, 2, 3, 1, 1, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0, 0,
                                    3, 4, 2, 5, 1, 6, 0, 7, 8, 9, 10, 11, 12 ];
      let mut acc: usize = 0;
      for i in 0..16 {
        htable.bits[i+1] = pentax_tree[i] as u32;
        acc += htable.bits[i+1] as usize;
      }
      for i in 0..acc {
        htable.huffval[i] = pentax_tree[i+16] as u32;
      }
    }

    htable.initialize(true)?;

    let mut pump = BitPumpMSB::new(src);
    let mut pred_up1: [i32;2] = [0, 0];
    let mut pred_up2: [i32;2] = [0, 0];
    let mut pred_left1: i32;
    let mut pred_left2: i32;

    let mut out = vec![0 as u16; width*height];
    for row in 0..height {
      pred_up1[row & 1] += htable.huff_decode(&mut pump)?;
      pred_up2[row & 1] += htable.huff_decode(&mut pump)?;
      pred_left1 = pred_up1[row & 1];
      pred_left2 = pred_up2[row & 1];
      out[row*width+0] = pred_left1 as u16;
      out[row*width+1] = pred_left2 as u16;
      for col in (2..width).step(2) {
        pred_left1 += htable.huff_decode(&mut pump)?;
        pred_left2 += htable.huff_decode(&mut pump)?;
        out[row*width+col+0] = pred_left1 as u16;
        out[row*width+col+1] = pred_left2 as u16;
      }
    }
    Ok(out)
  }
}