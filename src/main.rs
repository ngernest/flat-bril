#[allow(unused_imports)]
use serde_json::{json, Error, Value};
use std::{io, io::Read};

#[allow(dead_code)]
/// Flattened type for Bril instructions
struct Inst {
    op: usize,
    dest: usize,
    ty: usize,
    args: (usize, usize),
    label: (usize, usize),
}

fn main() {
    // Enable stack backtrace for debugging
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");
    let json: Value =
        serde_json::from_str(&buffer).expect("Unable to parse malformed JSON");
    println!("{}", json);
}
