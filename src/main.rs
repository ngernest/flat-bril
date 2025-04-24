use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{io, io::Read};

use strum_macros::EnumIter;

#[allow(dead_code)]
/// Flattened type for Bril instructions
struct Instr {
    op: usize,
    dest: usize,
    ty: usize,
    args: (usize, usize),
    label: (usize, usize),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum Opcode {
    // Arithmetic
    Add = 0,
    Mul = 1,
    Sub = 2,
    Div = 3,

    // Comparison
    Eq = 4,
    Lt = 5,
    Gt = 6,
    Le = 7,
    Ge = 8,

    // Logic operations
    Not = 9,
    And = 10,
    Or = 11,

    // Control flow
    Jmp = 12,
    Br = 13,
    Call = 14,
    Ret = 15,

    // Misc operations
    Id = 16,
    Print = 17,
    Nop = 18,
    Const = 19,
}

impl Opcode {
    /// Returns the `(start idx, end idx)` of the opcode in the `OPCODES` buffer
    fn get_indexes(&self) -> (usize, usize) {
        let opcode = self.clone();
        OPCODE_IDX[opcode as usize]
    }

    /// Converts an `Opcode` to a `&str`
    fn as_str(&self) -> &str {
        let (start_idx, end_idx) = self.get_indexes();
        &OPCODES[start_idx..=end_idx]
    }
}

const OPCODES: &str =
    "addmulsubdiveqltgtlegenotandorjmpbrcallretidprintnopconst";
const NUM_OPCODES: usize = 20;

/// Each pair contains the `(start idx, end idx)` of the opcode in `OPCODES`
const OPCODE_IDX: [(usize, usize); NUM_OPCODES] = [
    (0, 2),   // Add
    (3, 5),   // Mul
    (6, 8),   // Sub
    (9, 11),  // Div
    (12, 13), // Eq
    (14, 15), // Lt
    (16, 17), // Gt
    (18, 19), // Le
    (20, 21), // Ge
    (22, 24), // Not
    (25, 27), // And
    (28, 29), // Or
    (30, 32), // Jmp
    (33, 34), // Br
    (35, 38), // Call
    (39, 41), // Ret
    (42, 43), // Id
    (44, 48), // Print
    (49, 51), // Nop
    (52, 56), // Const
];

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
    let json: Value =
        serde_json::from_str(&buffer).expect("Unable to parse malformed JSON");
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
            println!("instr = {}", instr);
            let op_str: &str =
                instr["op"].as_str().expect("Expected `op` to be a string");
            println!("op_str = {}", op_str);
            let opcode: Opcode = serde_json::from_value(instr["op"].clone())
                .expect("Invalid opcode");

            println!("opcode as usize = {}", opcode.clone() as usize);

            let new_op_str = opcode.as_str();

            assert_eq!(op_str, new_op_str, "{} != {}", op_str, new_op_str);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // We use `strum` to iterate over every variant in the `Opcode` enum easily
    use strum::IntoEnumIterator;

    /// Checks that for all opcodes, their start/end indexes in `OPCODE_IDX` correct
    /// (what this test does is it converts the opcode to a string using `serde`,
    /// and checks that the corresponding substring when we index into `OPCODES`
    /// is the same)
    #[test]
    fn test_opcode() {
        for opcode in Opcode::iter() {
            let json: Value = serde_json::json!(opcode);
            let deserialized_op: Value =
                serde_json::from_value(json).expect("trouble deserializing");
            let serde_op_str = deserialized_op.as_str().unwrap();
            let op_str = opcode.as_str();
            assert_eq!(
                serde_op_str, op_str,
                "{:?} != {:?}",
                serde_op_str, op_str
            );
        }
    }
}
