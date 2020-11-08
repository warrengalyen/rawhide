use decoders::basics::*;
use decoders::tiff::*;
use decoders::*;

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
        let make = try!(self
            .tiff
            .find_entry(Tag::MAKE)
            .ok_or("ARW: Couldn't find Make".to_string()))
        .get_str();
        let model = try!(self
            .tiff
            .find_entry(Tag::MODEL)
            .ok_or("ARW: Couldn't find Model".to_string()))
        .get_str();
        self.rawhide.check_supported(make, model)
    }

    fn image(&self) -> Result<Image, String> {
        Err("ARW: Decoding not implemented yet!".to_string())
    }
}
