use std::env;
use std::error::Error;
use std::fs::File;
extern crate rawhide;
extern crate time;
use rawhide::decoders;

fn usage() {
    println!("benchmark <file>");
    std::process::exit(1);
}

static STEP_ITERATIONS: u32 = 10;
static MIN_ITERATIONS: u32 = 25;
static MIN_TIME: u64 = 5000000000;

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

    let mut iterations = 0;
    let mut f = match File::open(file) {
        Ok(val) => val,
        Err(e) => {
            error(e.description());
            return;
        }
    };
    let buffer = match decoders::Buffer::new(&mut f) {
        Ok(val) => val,
        Err(e) => {
            error(&e);
            return;
        }
    };
    let rawhide = decoders::RawHide::new();
    let from_time = time::precise_time_ns();
    loop {
        for _ in 0..STEP_ITERATIONS {
            let decoder = match rawhide.get_decoder(&buffer) {
                Ok(val) => val,
                Err(e) => {
                    error(&e);
                    return;
                }
            };
            match decoder.image() {
                Ok(_) => {}
                Err(e) => error(&e),
            }
        }
        iterations += STEP_ITERATIONS;
        let to_time = time::precise_time_ns();
        if iterations >= MIN_ITERATIONS && (to_time - from_time) >= MIN_TIME {
            println!(
                "Average decode time: {} ms ({} iterations)",
                (to_time - from_time) / (iterations as u64) / 1000000,
                iterations
            );
            break;
        }
    }
}
