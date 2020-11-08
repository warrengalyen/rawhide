use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;

#[derive(Debug, Clone)]
pub struct ArwDecoder<'a> {
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> ArwDecoder<'a> {
  pub fn new(tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> ArwDecoder<'a> {
    ArwDecoder {
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for ArwDecoder<'a> {
  fn identify(&self) -> Result<&Camera, String> {
    let make = fetch_tag!(self.tiff, Tag::MAKE, "ARW: Couldn't find Make").get_str();
    let model = fetch_tag!(self.tiff, Tag::MODEL, "ARW: Couldn't find Model").get_str();
    self.rawhide.check_supported(make, model)
  }

  fn image(&self) -> Result<Image,String> {
    Err("ARW: Decoding not implemented yet!".to_string())
  }
}
