#[macro_use]
extern crate afl;
extern crate rawhide;

fn main() {
    rawhide::force_initialization();

  fuzz_nohook!(|data: &[u8]| {
    rawhide::decode_unwrapped(&mut &data[..]).ok();
  });
}