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
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");
    let json: Value = serde_json::from_str(&buffer).expect("Unable to parse malformed JSON");
    let functions = json["functions"]
        .as_array()
        .expect("Expected `functions` to be a JSON array");
    for func in functions {
        let name = func["name"]
            .as_str()
            .expect("Expected `name` to be a string");
        println!("{}", name);
        let instrs = func["instrs"]
            .as_array()
            .expect("Expected `instrs` to be a JSON array");
        for instr in instrs {
            println!("{}", instr);
        }
    }
}
