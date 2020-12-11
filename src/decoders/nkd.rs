use decoders::*;
use decoders::basics::*;
use std::f32::NAN;

#[derive(Debug, Clone)]
pub struct NakedDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  camera: &'a Camera,
}

impl<'a> NakedDecoder<'a> {
  pub fn new(buf: &'a [u8], cam: &'a Camera, rawhide: &'a RawHide) -> NakedDecoder<'a> {
    NakedDecoder {
      buffer: buf,
      camera: cam,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for NakedDecoder<'a> {
  fn image(&self) -> Result<Image,String> {
    let width = self.camera.raw_width;
    let height = self.camera.raw_height;

    let image = decode_10le_lsb16(self.buffer, width, height);
    ok_image(self.camera, width as u32, height as u32, [NAN,NAN,NAN,NAN], image)
  }
}