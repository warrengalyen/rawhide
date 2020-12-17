use decoders::*;
use decoders::tiff::*;
use decoders::ljpeg::*;
use std::f32::NAN;
use std::cmp;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Cr2Decoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> Cr2Decoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> Cr2Decoder<'a> {
    Cr2Decoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for Cr2Decoder<'a> {
  fn image(&self) -> Result<Image,String> {
    let camera = self.rawhide.check_supported(&self.tiff)?;
    let raw = fetch_ifd!(&self.tiff, Tag::Cr2Id);
    let offset = fetch_tag!(raw, Tag::StripOffsets).get_usize(0);
    let src = &self.buffer[offset..];

    let (width, height, image) = {
      let decompressor = LjpegDecompressor::new(src, true)?;
      let width = decompressor.width();
      let height = decompressor.height();
      let mut ljpegout = vec![0 as u16; width*height];
      decompressor.decode(&mut ljpegout, 0, width, width, height)?;

      // Take each of the vertical fields and put them into the right location
      // FIXME: Doing this at the decode would reduce about 10% in runtime but I haven't
      //        been able to do it without hairy code
      let mut out = vec![0 as u16; width*height];
      let canoncol = fetch_tag!(raw, Tag::Cr2StripeWidths).get_usize(1);
      for (field, fieldstart) in (0..width).step(canoncol).enumerate() {
        let fieldwidth = cmp::min((field+1)*canoncol,width) - field*canoncol;
        let fieldpos = field*canoncol*height;
        for row in 0..height {
          for col in 0..fieldwidth {
            out[row*width+fieldstart+col] = ljpegout[fieldpos+row*fieldwidth+col];
          }
        }
      }

      (width, height, out)
    };
    ok_image(camera, width, height, self.get_wb()?, image)
  }
}

impl<'a> Cr2Decoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    let levels = fetch_tag!(self.tiff, Tag::Cr2ColorData);
    let offset = 63;
    Ok([levels.get_force_u16(offset) as f32, levels.get_force_u16(offset+1) as f32,
        levels.get_force_u16(offset+3) as f32, NAN])
  }
}