use std::io::{self, Read};

use crate::flatten;
use crate::unflatten;

/// Does a round trip from JSON -> flattened representation -> back to JSON
pub fn json_roundtrip() {
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
    let mut func_json_vec = vec![];
    for func in functions {
        let instr_store = flatten::flatten_instrs(func);
        let func_json = unflatten::unflatten_instrs(&instr_store);
        func_json_vec.push(func_json);
    }
    let prog_json = serde_json::json!({
        "functions": func_json_vec
    });
    println!("{:#}", prog_json);
}
