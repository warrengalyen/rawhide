use decoders::ljpeg::*;
use decoders::tiff::*;
use decoders::*;
use std::f32::NAN;

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
    fn image(&self) -> Result<Image, String> {
        let camera = self.rawhide.check_supported(&self.tiff)?;
        let raw = fetch_ifd!(&self.tiff, Tag::Cr2Id);
        let offset = fetch_tag!(raw, Tag::StripOffsets).get_usize(0);
        let src = &self.buffer[offset..];

        let (width, height, image) = {
            let decompressor = LjpegDecompressor::new(src, true)?;
            let width = decompressor.width();
            let height = decompressor.height();
            let mut ljpegout = vec![0 as u16; width * height];
            decompressor.decode(&mut ljpegout, 0, width, width, height)?;

            // Take each of the vertical fields and put them into the right location
            // FIXME: Doing this at the decode would reduce about 5% in runtime but I haven't
            //        been able to do it without hairy code
            let canoncol = fetch_tag!(raw, Tag::Cr2StripeWidths);
            if canoncol.get_usize(0) == 0 {
                (width, height, ljpegout)
            } else {
                let mut out = vec![0 as u16; width * height];
                let mut fieldwidths = Vec::new();
                for _ in 0..canoncol.get_usize(0) {
                    fieldwidths.push(canoncol.get_usize(1));
                }
                fieldwidths.push(canoncol.get_usize(2));

                let mut fieldstart = 0;
                let mut fieldpos = 0;
                for fieldwidth in fieldwidths {
                    for row in 0..height {
                        let outpos = row * width + fieldstart;
                        let inpos = fieldpos + row * fieldwidth;
                        let outb = &mut out[outpos..outpos + fieldwidth];
                        let inb = &ljpegout[inpos..inpos + fieldwidth];
                        outb.copy_from_slice(inb);
                    }
                    fieldstart += fieldwidth;
                    fieldpos += fieldwidth * height;
                }

                (width, height, out)
            }
        };
        ok_image(camera, width, height, self.get_wb(camera)?, image)
    }
}

impl<'a> Cr2Decoder<'a> {
    fn get_wb(&self, cam: &Camera) -> Result<[f32; 4], String> {
        if let Some(levels) = self.tiff.find_entry(Tag::Cr2ColorData) {
            let offset = if cam.wb_offset != 0 {cam.wb_offset} else {63};
            Ok([levels.get_force_u16(offset) as f32, levels.get_force_u16(offset+1) as f32,
                levels.get_force_u16(offset+3) as f32, NAN])
          } else if let Some(levels) = self.tiff.find_entry(Tag::Cr2PowerShotWB) {
            Ok([levels.get_force_u32(3) as f32, levels.get_force_u32(2) as f32,
                levels.get_force_u32(4) as f32, NAN])
          } else {
            Err("CR2: Couldn't find WB".to_string())
          }
    }
}
