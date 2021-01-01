//! Library to extract the raw data and some metadata from digital camera
//! images. Given an image in a supported format and camera you will be able to get
//! everything needed to process the image
//!
//! # Example
//! ```rust,no_run
//! use std::env;
//! use std::fs::File;
//! use std::io::prelude::*;
//! use std::io::BufWriter;
//!
//! extern crate rawhide;
//!
//! fn main() {
//!   let args: Vec<_> = env::args().collect();
//!   if args.len() != 2 {
//!     println!("Usage: {} <file>", args[0]);
//!     std::process::exit(2);
//!   }
//!   let file = &args[1];
//!   let image = rawhide::decode(file).unwrap();
//!
//!   // Write out the image as a grayscale PPM
//!   let mut f = BufWriter::new(File::create(format!("{}.ppm",file)).unwrap());
//!   let preamble = format!("P6 {} {} {}\n", image.width, image.height, 65535).into_bytes();
//!   f.write_all(&preamble).unwrap();
//!   for pix in image.data {
//!     // Do an extremely crude "demosaic" by setting R=G=B
//!     let pixhigh = (pix>>8) as u8;
//!     let pixlow  = (pix&0x0f) as u8;
//!     f.write_all(&[pixhigh, pixlow, pixhigh, pixlow, pixhigh, pixlow]).unwrap()
//!   }
//! }
//! ```

#![deny(
  missing_docs,
  missing_debug_implementations,
  missing_copy_implementations,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications
)]

#[macro_use] extern crate enum_primitive;
extern crate num;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate itertools;

mod decoders;
pub use decoders::RawImage;
pub use decoders::Orientation;
pub use decoders::cfa::CFA;
#[doc(hidden)] pub use decoders::Buffer;
#[doc(hidden)] pub use decoders::RawHide;

lazy_static! {
  static ref LOADER: RawHide = decoders::RawHide::new();
  }

  use std::path::Path;
  use std::error::Error;
use std::fmt;

/// Error type for any reason for the decode to fail
#[derive(Debug)]
pub struct RawHideError {
  msg: String,
}

impl fmt::Display for RawHideError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RawLoaderError: \"{}\"", self.msg)
  }
}

impl Error for RawHideError {
}

impl RawHideError {
  fn new(msg: String) -> Self {
    Self {
      msg,
    }
  }
}
  
  /// Take a path to a raw file and return a decoded image or an error
  ///
  /// # Example
  /// ```rust,ignore
  /// let image = match rawhide::decode("path/to/your/file.RAW") {
  ///   Ok(val) => val,
  ///   Err(e) => ... some appropriate action when the file is unreadable ...
  /// };
  /// ```
  pub fn decode<P: AsRef<Path>>(path: P) -> Result<RawImage,RawHideError> {
    LOADER.decode(path.as_ref()).map_err(|err| RawHideError::new(err))
  }
