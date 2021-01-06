#[macro_use]
extern crate afl;
extern crate rawhide;

fn main() {
    rawhide::force_initialization();

    fuzz_nohook!(|data: &[u8]| {
        // Remove the panic hook so we can actually catch panic
        std::panic::set_hook(Box::new(|_| {}));

        rawhide::decode_dummy(&mut &data[..]).ok();
    });
}
