use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use std::f32::NAN;

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
      c => return Err(format!("PEF: Don't know how to read compression {}", c).to_string()),
    };

    ok_image(camera, width, height, self.get_wb()?, image)
  }
}

impl<'a> PefDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    let levels = fetch_tag!(self.tiff, Tag::PefWB);
    Ok([levels.get_f32(0), levels.get_f32(1), levels.get_f32(3), NAN])
  }
}