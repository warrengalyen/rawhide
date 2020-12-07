use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use std::f32::NAN;

#[derive(Debug, Clone)]
pub struct DngDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> DngDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> DngDecoder<'a> {
    DngDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for DngDecoder<'a> {
  fn image(&self) -> Result<Image,String> {
    let camera = self.rawhide.check_supported(&self.tiff)?;
    let ifds = self.tiff.find_ifds_with_tag(Tag::Compression).into_iter().filter(|ifd| {
      let compression = (**ifd).find_entry(Tag::Compression).unwrap().get_u32(0);
      let subsampled = match (**ifd).find_entry(Tag::NewSubFileType) {
        Some(e) => e.get_u32(0) & 1 != 0,
        None => false,
      };
      !subsampled && (compression == 7 || compression == 1 || compression == 0x884c)
    }).collect::<Vec<&TiffIFD>>();
    let raw = ifds[0];
    let width = fetch_tag!(raw, Tag::ImageWidth).get_u32(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_u32(0);

    let image = match fetch_tag!(raw, Tag::Compression).get_u32(0) {
      1 => self.decode_uncompressed(raw, width as usize, height as usize)?,
      c => return Err(format!("Don't know how to read DNGs with compression {}", c).to_string()),
    };

    ok_image(camera, width, height, self.get_wb()?, image)
  }
}

impl<'a> DngDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    let levels = fetch_tag!(self.tiff, Tag::AsShotNeutral);
    Ok([1.0/levels.get_f32(0),1.0/levels.get_f32(1),1.0/levels.get_f32(2),NAN])
  }

  pub fn decode_uncompressed(&self, raw: &TiffIFD, width: usize, height: usize) -> Result<Vec<u16>,String> {
    let offset = fetch_tag!(raw, Tag::StripOffsets).get_u32(0) as usize;
    let src = &self.buffer[offset..];

    match fetch_tag!(raw, Tag::BitsPerSample).get_u32(0) {
      16  => Ok(decode_16le(src, width, height)),
      bps => Err(format!("DNG: Don't know about {} bps images", bps).to_string()),
    }
  }
}