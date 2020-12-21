use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use std::f32::NAN;

#[derive(Debug, Clone)]
pub struct DcsDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> DcsDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> DcsDecoder<'a> {
    DcsDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for DcsDecoder<'a> {
  fn image(&self) -> Result<RawImage,String> {
    let camera = self.rawhide.check_supported(&self.tiff)?;
    let data = self.tiff.find_ifds_with_tag(Tag::StripOffsets);
    let raw = data.iter().find(|&&ifd| {
      ifd.find_entry(Tag::ImageWidth).unwrap().get_u32(0) > 1000
    }).unwrap();
    let width = fetch_tag!(raw, Tag::ImageWidth).get_usize(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_usize(0);
    let offset = fetch_tag!(raw, Tag::StripOffsets).get_usize(0);
    let src = &self.buffer[offset..];
    let linearization = fetch_tag!(self.tiff, Tag::GrayResponse);
    let table = {
      let mut t: [u16;256] = [0;256];
      for i in 0..256 {
        t[i] = linearization.get_u32(i) as u16;
      }
      LookupTable::new(&t)
    };

    let image = decode_8bit_wtable(src, &table, width, height);
    ok_image(camera, width, height, [NAN,NAN,NAN,NAN], image)
  }
}