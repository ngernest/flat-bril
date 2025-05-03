use std::io::{self, Read};

mod flatten;
mod types;
mod unflatten;

fn main() {
    // Enable stack backtrace for debugging
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Read in the JSON representation of a Bril file from stdin
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");

    // Parse the JSON into serde_json's `Value` datatype
    let json: serde_json::Value =
        serde_json::from_str(&buffer).expect("Unable to parse malformed JSON");
    let functions = json["functions"]
        .as_array()
        .expect("Expected `functions` to be a JSON array");
    for func in functions {
        let _instrs = flatten::create_instrs(func);
        // TODO: figure out what to do with _instrs
    }
}
