#[macro_use]
extern crate afl;
extern crate rawhide;

fn main() {
  let loader = rawhide::RawHide::new();

  fuzz!(|data: &[u8]| {
    // Remove the panic hook so we can actually catch panic
    std::panic::set_hook(Box::new(|_| {} ));

    loader.decode(&mut &data[..]).ok();
  });
}