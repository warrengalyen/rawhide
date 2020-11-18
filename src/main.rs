use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
extern crate time;
extern crate toml;

mod decoders;

fn usage() {
    println!("rawhide <file>");
    std::process::exit(1);
}

fn error(err: &str) {
    println!("ERROR: {}", err);
    std::process::exit(2);
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        usage();
    }
    let file = &args[1];
    println!("Loading file \"{}\"", file);

    let rawhide = decoders::RawHide::new();
    let from_time = time::precise_time_ns();
    let image  = match rawhide.decode_safe(file)  {
        Ok(val) => val,
        Err(e) => {
            error(&e);
            unreachable!()
        }
    };
    let to_time = time::precise_time_ns();
    println!("Decoded in {} ms", (to_time - from_time)/1000000);

    println!("Found camera \"{}\" model \"{}\"", image.make, image.model);
    println!("Found canonical named camera \"{}\" model \"{}\"", image.canonical_make, image.canonical_model);
    println!("Image size is {}x{}", image.width, image.height);
    println!("WB coeffs are {:?}", image.wb_coeffs);
    println!("black levels are {:?}", image.blacklevels);
    println!("white levels are {:?}", image.whitelevels);
    println!("color matrix is {:?}", image.color_matrix);
    println!("dcraw filters is {:#x}", image.dcraw_filters);
    println!("crops are {:?}", image.crops);

    let mut sum: u64 = 0;
    for i in 0..(image.width * image.height) {
        sum += image.data[i as usize] as u64;
    }
    println!("Image sum: {}", sum);
    let count: u64 = (image.width as u64) * (image.height as u64);
    println!("Image avg: {}", sum / count);

    let uf = match File::create(format!("{}.ppm", file)) {
        Ok(val) => val,
        Err(e) => {
            error(e.description());
            unreachable!()
        }
    };
    let mut f = BufWriter::new(uf);
    let preamble = format!("P6 {} {} {}\n", image.width, image.height, image.whitelevels[0]).into_bytes();
    if let Err(err) = f.write_all(&preamble) {
        error(err.description());
    }
    for row in 0..image.height {
        let from: usize = (row as usize) * (image.width as usize);
        let to: usize = ((row + 1) as usize) * (image.width as usize);
        let imgline = &image.data[from..to];

        for pixel in imgline {
            let bytes = [
                (pixel >> 4) as u8,
                (pixel & 0x0f) as u8,
                (pixel >> 4) as u8,
                (pixel & 0x0f) as u8,
                (pixel >> 4) as u8,
                (pixel & 0x0f) as u8,
            ];
            if let Err(err) = f.write_all(&bytes) {
                error(err.description());
            }
        }
    }
}
