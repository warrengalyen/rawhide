# RawHide

This is a rust library to decode image data and some metadata from digital camera images. Given an image in a supported format you will be able to retrieve everthing needed to process the image:

  * Identification of the camera that produced the image (both the EXIF name and a clean/canonical name)
  * The raw pixels themselves, exactly as encoded by the camera
  * The number of pixels to crop on the top, right, bottom, left of the image to only use the actual image area
  * The black and white points of each of the color channels
  * The multipliers to apply to the color channels for the white balance
  * A conversion matrix between the camera color space and XYZ
  * The description of the bayer pattern itself so you'll know which pixels are which color

Additionally it includes a simple raw processing pipeline that does the following:

  * Demosaic
  * Black and whitelevel application
  * Whitebalance
  * Convert from camera space to Lab
  * Apply a contrast curve to the L channel
  * Convert from Lab to Rec709
  * Apply sRGB gamma for output

  Current State
  -------------

The library is still a work in process with the following formats already implemented:
  * Minolta MRW
  * Sony ARW, SRF and SR2
  * Mamiya MEF
  * Olympus ORF
  * Samsung SRW
  * Epson ERF
  * Kodak KDC
  * Kodak DCS
  * Panasonic RW2
  * Fuji RAF
  * Kodak DCR
  * Adobe DNG (the "good parts"<sup>1</sup>)
  * Pentax PEF
  * Canon CRW

<sup>1</sup> DNG is a 101 page overambitions spec that tries to be an interchange format for processed images, complete with image transformation operations. We just implement enough of the spec so that actual raw files from DNG producing cameras or the Adobe DNG converter can be read.
  
Usage
-----

Here's a simple sample program that uses this library:

```rust
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

extern crate rawhide;
use rawhide::decoders;

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() != 2 {
    println!("Usage: {} <file>", args[0]);
    std::process::exit(2);
  }
  let file = &args[1];
  println!("Loading file \"{}\"", file);

  let rawhide = decoders::RawHide::new();
  let image = rawhide.decode_safe(file).unwrap();
  println!("Found camera \"{}\" model \"{}\"", image.make, image.model);
  println!("Found canonical named camera \"{}\" model \"{}\"", image.canonical_make, image.canonical_model);

  println!("Image size is {}x{}", image.width, image.height);
  println!("WB coeffs are {:?}", image.wb_coeffs);
  println!("black levels are {:?}", image.blacklevels);
  println!("white levels are {:?}", image.whitelevels);
  println!("color matrix is {:?}", image.color_matrix);
  println!("CFA is {:?}", image.cfa);
  println!("crops are {:?}", image.crops);

  // Write out the image as a grayscale PPM
  let mut f = BufWriter::new(File::create(format!("{}.ppm",file)).unwrap());
  let preamble = format!("P6 {} {} {}\n", image.width, image.height, image.whitelevels[0]).into_bytes();
  f.write_all(&preamble).unwrap();
  for row in 0..image.height {
    let from: usize = (row as usize) * (image.width as usize);
    let to: usize = ((row+1) as usize) * (image.width as usize);
    let imgline = &image.data[from .. to];

    for pixel in imgline {
      // Do an extremely crude "demosaic" by setting R=G=B
      let bytes = [(pixel>>4) as u8, (pixel&0x0f) as u8, (pixel>>4) as u8, (pixel&0x0f) as u8, (pixel>>4) as u8, (pixel&0x0f) as u8];
      f.write_all(&bytes).unwrap();
    }
  }
}
```

To do the image decoding decode the image the same way but then do:

```rust
  // Decode to the largest image that fits in 1080p size. If the original image is
  // smaller this will not scale up but otherwise you will get an image that is either
  // 1920 pixels wide or 1080 pixels tall and maintains the image ratio.
  let decoded = imageops::simple_decode(&image, 1920, 1080);
  let mut f = BufWriter::new(File::create(format!("{}.ppm",file)).unwrap());
  let preamble = format!("P6 {} {} {}\n", decoded.width, decoded.height, 255).into_bytes();
  f.write_all(&preamble).unwrap();
  for pix in decoded.data {
    let pixel = ((pix.max(0.0)*255.0).min(255.0)) as u8;
    f.write_all(&[pixel]).unwrap();
  }
```

And this will write out an 8bit RGB image. Reducing the decode size makes the processing much faster by doing the scaling extremely early in the processing (at the demosaic). So this can be used quite directly to do fast thumbnailing of raw images. A 24MP raw file can be turned into a 500x500 thumbnail in 200-300ms on normal laptop hardware.

Contributing
------------

Bug reports and pull requests welcome at https://github.com/warrengalyen/rawhide