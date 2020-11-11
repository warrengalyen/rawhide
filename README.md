# RawHide

This is a rust library to decode image data and some metadata from digital camera images. Given an image in a supported format you will be able to retrieve everthing needed to process the image:

  * Identification of the camera that produced the image (both the EXIF name and a clean/canonical name)
  * The raw pixels themselves, exactly as encoded by the camera
  * The number of pixels to crop on the top, right, bottom, left of the image to only use the actual image area
  * The black and white points of each of the color channels
  * The multipliers to apply to the color channels for the white balance
  * A conversion matrix between the camera color space and XYZ
  * The description of the bayer pattern itself so you'll know which pixels are which color

  Current State
  -------------

The library is still in a very early stage with only the simple Minolta MRW format implemented. 

Usage
-----

Here's a simple sample program that uses this library:

```rust
use std::env;
use std::fs::File;
use std::io::prelude::*;

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
  println!("dcraw filters is {:#x}", image.dcraw_filters);
  println!("crops are {:?}", image.crops);

  // Write out the image as a grayscale PPM in an extremely inefficient way
  let mut f = File::create(format!("{}.ppm",file)).unwrap();
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

Contributing
------------

Bug reports and pull requests welcome at https://github.com/warrengalyen/rawhide