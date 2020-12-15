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
    let model_full = self.xmp_tag("Model")?.to_string();
    let model = model_full.split_terminator("(").next().unwrap();
    let camera = self.rawhide.check_supported_with_everything(&make, &model, "")?;

    let raw = fetch_ifd!(&self.tiff, Tag::TileOffsets);
    let width = fetch_tag!(raw, Tag::ImageWidth).get_usize(0);
    let height = fetch_tag!(raw, Tag::ImageLength).get_usize(0);
    let offset = fetch_tag!(raw, Tag::TileOffsets).get_usize(0);
    let src = &self.buffer[offset..];

    let image = match fetch_tag!(raw, Tag::Compression).get_usize(0) {
      1 => {
        if self.tiff.little_endian() {
          decode_16le(src, width, height)
        } else {
          decode_16be(src, width, height)
        }
      },
      x => return Err(format!("MOS: unsupported compression {}", x).to_string())
      };
      
    ok_image(camera, width, height, self.get_wb()?, image)
  }
}

impl<'a> MosDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    let meta = fetch_tag!(self.tiff, Tag::LeafMetadata).get_data();
    let mut pos = 0;
    // We need at least 16+45+10 bytes for the NeutObj_neutrals section itself
    while pos + 70 < meta.len() {
      if meta[pos..pos+16] == b"NeutObj_neutrals"[..] {
        let data = &meta[pos+44..];
        if let Some(endpos) = data.iter().position(|&x| x == 0) {
          let nums = String::from_utf8_lossy(&data[0..endpos])
                       .split_terminator("\n")
                       .map(|x| x.parse::<f32>().unwrap_or(NAN))
                       .collect::<Vec<f32>>();
          if nums.len() == 4 {
            return Ok([nums[0]/nums[1], nums[0]/nums[2], nums[0]/nums[3], NAN])
          }
        }
        break;
      }
      pos += 1;
    }
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