use std::f32::NAN;

use crate::decoders::*;
use crate::decoders::tiff::*;
use crate::decoders::basics::*;

#[derive(Debug, Clone)]
pub struct MefDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> MefDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> MefDecoder<'a> {
    MefDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for MefDecoder<'a> {
  fn image(&self, dummy: bool) -> Result<RawImage,String> {
    let camera = self.rawhide.check_supported(&self.tiff)?;
    let raw = fetch_ifd!(&self.tiff, Tag::CFAPattern);
    let width = fetch_tag!(raw, Tag::ImageWidth).get_usize(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_usize(0);
    let offset = fetch_tag!(raw, Tag::StripOffsets).get_usize(0);
    let src = &self.buffer[offset..];

    let image = decode_12be(src, width, height, dummy);
    ok_image(camera, width, height, [NAN,NAN,NAN,NAN], image)
  }
}