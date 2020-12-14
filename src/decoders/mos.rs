use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use std::f32::NAN;

#[derive(Debug, Clone)]
pub struct MosDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> MosDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> MosDecoder<'a> {
    MosDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for MosDecoder<'a> {
  fn image(&self) -> Result<Image,String> {
    let make = self.xmp_tag("Make")?;
    let model = self.xmp_tag("Model")?;
    let camera = self.rawhide.check_supported_with_everything(&make, &model, "")?;

    let raw = fetch_ifd!(&self.tiff, Tag::TileOffsets);
    let width = fetch_tag!(raw, Tag::ImageWidth).get_u32(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_u32(0);
    let offset = fetch_tag!(raw, Tag::TileOffsets).get_u32(0) as usize;
    let src = &self.buffer[offset..];

    let image = decode_16be(src, width as usize, height as usize);
    ok_image(camera, width, height, self.get_wb()?, image)
  }
}

impl<'a> MosDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    Ok([NAN,NAN,NAN,NAN])
  }

  fn xmp_tag(&self, tag: &str) -> Result<String, String> {
    let xmp = fetch_tag!(self.tiff, Tag::Xmp).get_str();
    let error = format!("MOS: Couldn't find XMP tag {}", tag).to_string();
    let start = xmp.find(&format!("<tiff:{}>",tag)).ok_or(error.clone())?;
    let end   = xmp.find(&format!("</tiff:{}>",tag)).ok_or(error.clone())?;

    Ok(xmp[start+tag.len()+7..end].to_string())
  }
}