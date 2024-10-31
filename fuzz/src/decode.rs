#![no_main]

use mp4_atom::{Any, Decode};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut cursor = std::io::Cursor::new(data);
    let _ = Any::decode(&mut cursor);
});
