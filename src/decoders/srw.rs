use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use std::f32::NAN;
use itertools::Itertools;
use std::cmp;

#[derive(Debug, Clone)]
pub struct SrwDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> SrwDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> SrwDecoder<'a> {
    SrwDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for SrwDecoder<'a> {
  fn identify(&self) -> Result<&Camera, String> {
    let make = fetch_tag!(self.tiff, Tag::Make, "SRW: Couldn't find Make").get_str();
    let model = fetch_tag!(self.tiff, Tag::Model, "SRW: Couldn't find Model").get_str();
    self.rawhide.check_supported(make, model)
  }

  fn image(&self) -> Result<Image,String> {
    let camera = try!(self.identify());
    let data = self.tiff.find_ifds_with_tag(Tag::StripOffsets);
    let raw = data[0];
    let width = fetch_tag!(raw, Tag::ImageWidth, "SRW: Couldn't find width").get_u32(0);
    let height = fetch_tag!(raw, Tag::ImageLength, "SRW: Couldn't find height").get_u32(0);
    let offset = fetch_tag!(raw, Tag::StripOffsets, "SRW: Couldn't find offset").get_u32(0) as usize;
    let compression = fetch_tag!(raw, Tag::Compression, "SRW: Couldn't find compression").get_u32(0);
    let bits = fetch_tag!(raw, Tag::BitsPerSample, "SRW: Couldn't find bps").get_u32(0);
    let src = &self.buffer[offset..];

    let image = match compression {
      32770 => {
        match raw.find_entry(Tag::SrwSensorAreas) {
          None => match bits {
            12 => decode_12be(src, width as usize, height as usize),
            14 => decode_14le_unpacked(src, width as usize, height as usize),
             x => return Err(format!("SRW: Don't know how to handle bps {}", x).to_string()),
            },
            Some(x) => {
              let coffset = x.get_u32(0) as usize;
              let loffsets = &self.buffer[coffset..];
              SrwDecoder::decode_srw1(src, loffsets, width as usize, height as usize)
          }
        }
      }
      x => return Err(format!("SRW: Don't know how to handle compression {}", x).to_string()),
    };

    ok_image(camera, width, height, try!(self.get_wb()), image)
  }
}

impl<'a> SrwDecoder<'a> {
  pub fn decode_srw1(buf: &[u8], loffsets: &[u8], width: usize, height: usize) -> Vec<u16> {
    let mut out: Vec<u16> = vec![0; (width*height) as usize];

    for row in 0..height {
      let mut len: [u32; 4] = [if row < 2 {7} else {4}; 4];
      let loffset = LEu32(loffsets, row*4) as usize;
      let mut pump = BitPumpMSB32::new(&buf[loffset..]);

      let img      = width*row;
      let img_up   = width*(cmp::max(1, row)-1);
      let img_up2  = width*(cmp::max(2, row)-2);

      // Image is arranged in groups of 16 pixels horizontally
      for col in (0..width).step(16) {
        let dir = pump.get_bits(1) == 1;

        let ops = [pump.get_bits(2), pump.get_bits(2), pump.get_bits(2), pump.get_bits(2)];
        for (i, op) in ops.iter().enumerate() {
          match *op {
            3 => {len[i] = pump.get_bits(4);},
            2 => {len[i] -= 1;},
            1 => {len[i] += 1;},
            _ => {},
          }
        }

        // First decode even pixels
        for c in (0..16).step(2) {
          let l = len[(c >> 3)];
          let adj = pump.get_ibits_sextended(l);
          let predictor = if dir { // Upward prediction
              out[img_up+col+c]
          } else { // Left to right prediction
              if col == 0 { 128 } else { out[img+col-2] }
          };
          if col+c < width { // No point in decoding pixels outside the image
            out[img+col+c] = ((predictor as i32) + adj) as u16;
          }
        }
        // Now decode odd pixels
        for c in (1..16).step(2) {
          let l = len[2 | (c >> 3)];
          let adj = pump.get_ibits_sextended(l);
          let predictor = if dir { // Upward prediction
              out[img_up2+col+c]
          } else { // Left to right prediction
              if col == 0 { 128 } else { out[img+col-1] }
          };
          if col+c < width { // No point in decoding pixels outside the image
            out[img+col+c] = ((predictor as i32) + adj) as u16;
          }
        }
      }
    }

    // SRW1 apparently has red and blue swapped, just changing the CFA pattern to
    // match causes color fringing in high contrast areas because the actual pixel
    // locations would not match the CFA pattern
    for row in (0..height).step(2) {
      for col in (0..width).step(2) {
        out.swap(row*width+col+1, (row+1)*width+col);
      }
    }

    out
  }

  fn get_wb(&self) -> Result<[f32;4], String> {
    let rggb_levels = fetch_tag!(self.tiff, Tag::SrwRGGBLevels, "SRW: No RGGB Levels");
    let rggb_blacks = fetch_tag!(self.tiff, Tag::SrwRGGBBlacks, "SRW: No RGGB Blacks");
    if rggb_levels.count() != 4 || rggb_blacks.count() != 4 {
      Err("SRW: RGGB Levels and Blacks don't have 4 elements".to_string())
    } else {
      let nlevels = &rggb_levels.copy_offset_from_parent(&self.buffer);
      let nblacks = &rggb_blacks.copy_offset_from_parent(&self.buffer);
      Ok([nlevels.get_u32(0) as f32 - nblacks.get_u32(0) as f32,
          nlevels.get_u32(1) as f32 - nblacks.get_u32(1) as f32,
          nlevels.get_u32(3) as f32 - nblacks.get_u32(3) as f32,
          NAN])
    }
  }
}