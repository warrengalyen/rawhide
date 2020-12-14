use decoders::*;
use decoders::tiff::*;
use decoders::basics::*;
use std::f32::NAN;

#[derive(Debug, Clone)]
pub struct RafDecoder<'a> {
  buffer: &'a [u8],
  rawhide: &'a RawHide,
  tiff: TiffIFD<'a>,
}

impl<'a> RafDecoder<'a> {
  pub fn new(buf: &'a [u8], tiff: TiffIFD<'a>, rawhide: &'a RawHide) -> RafDecoder<'a> {
    RafDecoder {
      buffer: buf,
      tiff: tiff,
      rawhide: rawhide,
    }
  }
}

impl<'a> Decoder for RafDecoder<'a> {
  fn image(&self) -> Result<Image,String> {
    let camera = self.rawhide.check_supported(&self.tiff)?;
    let raw = fetch_ifd!(&self.tiff, Tag::RafOffsets);
    let (width,height) = if raw.has_entry(Tag::RafImageWidth) {
      (fetch_tag!(raw, Tag::RafImageWidth).get_usize(0),
       fetch_tag!(raw, Tag::RafImageLength).get_usize(0))
    } else {
      let sizes = fetch_tag!(raw, Tag::ImageWidth);
      (sizes.get_usize(1), sizes.get_usize(0))
    };
    let offset = fetch_tag!(raw, Tag::RafOffsets).get_usize(0) + raw.start_offset();
    let bps = match raw.find_entry(Tag::RafBitsPerSample) {
      Some(val) => val.get_u32(0),
      None      => 16,
    };
    let src = &self.buffer[offset..];

    let image = if camera.find_hint("double_width") {
      // Some fuji SuperCCD cameras include a second raw image next to the first one
      // that is identical but darker to the first. The two combined can produce
      // a higher dynamic range image. Right now we're ignoring it.
      decode_16le_skiplines(src, width, height)
    } else if camera.find_hint("jpeg32") {
      decode_12be_msb32(src, width, height)
    } else {
      match bps {
        12 => decode_12le(src, width, height),
        14 => decode_14le_unpacked(src, width, height),
        16 => {
          if self.tiff.little_endian() {
            decode_16le(src, width, height)
          } else {
            decode_16be(src, width, height)
          }
        },
        _ => {return Err(format!("RAF: Don't know how to decode bps {}", bps).to_string());},
      }
    };

    if camera.find_hint("fuji_rotation") || camera.find_hint("fuji_rotation_alt") {
      let (width, height, image) = RafDecoder::rotate_image(&image, camera, width, height);
      Ok(Image {
        make: camera.make.clone(),
        model: camera.model.clone(),
        canonical_make: camera.canonical_make.clone(),
        canonical_model: camera.canonical_model.clone(),
        width: width,
        height: height,
        wb_coeffs: self.get_wb()?,
        data: image.into_boxed_slice(),
        blacklevels: camera.blacklevels,
        whitelevels: camera.whitelevels,
        xyz_to_cam: camera.xyz_to_cam,
        cfa: camera.cfa.clone(),
        crops: [0,0,0,0],
      })
    } else {
      ok_image(camera, width, height, self.get_wb()?, image)
    }
  }
}

impl<'a> RafDecoder<'a> {
  fn get_wb(&self) -> Result<[f32;4], String> {
    match self.tiff.find_entry(Tag::RafWBGRB) {
      Some(levels) => Ok([levels.get_f32(1), levels.get_f32(0), levels.get_f32(2), NAN]),
      None => {
        let levels = fetch_tag!(self.tiff, Tag::RafOldWB);
        Ok([levels.get_f32(1), levels.get_f32(0), levels.get_f32(3), NAN])
      },
    }
  }

  fn rotate_image(src: &[u16], camera: &Camera, width: usize, height: usize) -> (usize, usize, Vec<u16>) {
    let x = camera.crops[3];
    let y = camera.crops[0];
    let cropwidth = width - camera.crops[1] - x;
    let cropheight = height - camera.crops[2] - y;

    if camera.find_hint("fuji_rotation_alt") {
      let rotatedwidth = cropheight + cropwidth/2;
      let rotatedheight = rotatedwidth-1;

      let mut out: Vec<u16> = vec![0; rotatedwidth * rotatedheight];
      for row in 0..cropheight {
        let inb = &src[(row+y)*width+x..];
        for col in 0..cropwidth {
          let out_row = rotatedwidth - (cropheight + 1 - row + (col >> 1));
          let out_col = ((col+1) >> 1) + row;
          out[out_row*rotatedwidth+out_col] = inb[col];
        }
      }

      (rotatedwidth, rotatedheight, out)
    } else {
      let rotatedwidth = cropwidth + cropheight/2;
      let rotatedheight = rotatedwidth-1;

      let mut out: Vec<u16> = vec![0; rotatedwidth * rotatedheight];
      for row in 0..cropheight {
        let inb = &src[(row+y)*width+x..];
        for col in 0..cropwidth {
          let out_row = cropwidth - 1 - col + (row>>1);
          let out_col = ((row+1) >> 1) + col;
          out[out_row*rotatedwidth+out_col] = inb[col];
        }
      }

      (rotatedwidth, rotatedheight, out)
    }
  }
}
