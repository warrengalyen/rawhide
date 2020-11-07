use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
extern crate glob;
use self::glob::glob;

extern crate toml;
mod basics;
mod mrw;
mod tiff;

pub static CAMERAS_TOML: &'static str =
    include_str!("../../data/cameras/all.toml");

pub trait Decoder {
    fn identify(&self) -> Result<&Camera, String>;
    fn image(&self) -> Result<Image, String>;
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub wb_coeffs: [f32; 4],
    pub data: Box<[u16]>,
    pub whitelevels: [i64; 4],
    pub blacklevels: [i64; 4],
    pub color_matrix: [i64; 12],
    pub dcraw_filters: u32,
}

#[derive(Debug)]
pub struct Camera {
    pub make: String,
    pub model: String,
    pub canonical_make: String,
    pub canonical_model: String,
    whitelevels: [i64; 4],
    blacklevels: [i64; 4],
    color_matrix: [i64; 12],
    dcraw_filters: u32,
}

#[derive(Debug)]
pub struct RawHide {
    pub cameras: HashMap<(String, String), Camera>,
}

impl RawHide {
    pub fn new() -> RawHide {
        let mut map = HashMap::new();

        let mut parser = toml::Parser::new(&CAMERAS_TOML);
        let toml = match parser.parse() {
            Some(val) => val,
            None => {
                panic!(format!("Error parsing all.toml: {:?}", parser.errors))
            }
        };
        let cameras = toml.get("cameras").unwrap().as_table().unwrap();
        for (_, c) in cameras {
            let ct = c.as_table().unwrap();
            let make = ct.get("make").unwrap().as_str().unwrap().to_string();
            let model = ct.get("model").unwrap().as_str().unwrap().to_string();
            let canonical_make = ct
                .get("canonical_make")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let canonical_model = ct
                .get("canonical_model")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let white = ct.get("whitepoint").unwrap().as_integer().unwrap();
            let black = ct.get("blackpoint").unwrap().as_integer().unwrap();
            let matrix = ct.get("color_matrix").unwrap().as_slice().unwrap();
            let mut cmatrix: [i64; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            for (i, val) in matrix.into_iter().enumerate() {
                cmatrix[i] = val.as_integer().unwrap();
            }
            let color_pattern = ct
                .get("color_pattern")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let cam = Camera {
                make: make.clone(),
                model: model.clone(),
                canonical_make: canonical_make.clone(),
                canonical_model: canonical_model.clone(),
                whitelevels: [white, white, white, white],
                blacklevels: [black, black, black, black],
                color_matrix: cmatrix,
                dcraw_filters: RawHide::dcraw_filters(&color_pattern),
            };
            map.insert((make.clone(), model.clone()), cam);
        }

        RawHide { cameras: map }
    }

    pub fn get_decoder<'b>(
        &'b self,
        buffer: &'b [u8],
    ) -> Result<Box<Decoder + 'b>, String> {
        if mrw::is_mrw(buffer) {
            let dec = Box::new(mrw::MrwDecoder::new(buffer, &self));
            return Ok(dec as Box<Decoder>);
        }
        Err("Couldn't find a decoder!".to_string())
    }

    pub fn check_supported<'a>(
        &'a self,
        make: &'a str,
        model: &'a str,
    ) -> Result<&Camera, String> {
        match self.cameras.get(&(make.to_string(), model.to_string())) {
            Some(cam) => Ok(cam),
            None => {
                Err(format!("Couldn't find camera \"{}\" \"{}\"", make, model))
            }
        }
    }

    fn dcraw_filters(pattern: &str) -> u32 {
        match pattern {
            "BGGR" => 0x16161616,
            "GRBG" => 0x61616161,
            "GBRG" => 0x49494949,
            "RGGB" => 0x94949494,
            _ => 0,
        }
    }
}
