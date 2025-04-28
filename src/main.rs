use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{self, Read};

use strum_macros::EnumIter;

/* -------------------------------------------------------------------------- */
/*                                    Types                                   */
/* -------------------------------------------------------------------------- */

#[allow(dead_code, unused_variables)]
#[derive(Debug, PartialEq, Clone)]
/// Flattened type for Bril instructions.   
/// - The `op` field stores an index `i` into `OPCODE_IDX`, where
///   `OPCODE_IDX[i] = (start, end)`, such that `OPCODE_BUFFER[start..=end]`
///   is the serialized version of the opcode
/// - Similarly, `dest` field is an index into the `all_dests` array
/// - We can store the actual `type` and `value` inline in the `Instr` struct
///   (since they're either an int or a bool,
///   i.e. they don't need to be heap-allocated)
struct Instr {
    op: usize,
    dest: Option<usize>,
    ty: Option<Type>,
    args: Option<(usize, usize)>,
    labels: Option<(usize, usize)>,
    value: Option<BrilValue>,
}

/// Primitive types in core Bril are either `int` or `bool`
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Type {
    Int,
    Bool,
}

/// Bril values are either 64-bit integers or bools.   
/// Note: We call this enum `BrilValue` to avoid namespace clashes
/// with `serde_json::Value`
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
enum BrilValue {
    IntVal(i64),
    BoolVal(bool),
}

impl Instr {
    /// Creates a new `Instr` struct with the `op` field set to `opcode_idx`,
    /// and all other fields initialized to `None`
    #[allow(dead_code)]
    pub fn new(opcode_idx: usize) -> Self {
        Instr {
            op: opcode_idx,
            dest: None,
            ty: None,
            args: None,
            labels: None,
            value: None,
        }
    }
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
    fn get_buffer_start_end_indexes(&self) -> (usize, usize) {
        let opcode = self.clone();
        OPCODE_IDX[opcode as usize]
    }

    /// Extracts the index of the opcode's `(start_idx, end_idx)` pair
    /// in `OPCODE_IDX`
    pub fn get_index(&self) -> usize {
        let (start_idx, end_idx) = self.get_buffer_start_end_indexes();
        OPCODE_IDX
            .iter()
            .position(|&x| x.0 == start_idx && x.1 == end_idx)
            .expect("Opcode not present in `OPCODE_IDX`")
    }

    /// Converts an `Opcode` to a `&str`
    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        let (start_idx, end_idx) = self.get_buffer_start_end_indexes();
        &OPCODE_BUFFER[start_idx..=end_idx]
    }
}

/* -------------------------------------------------------------------------- */
/*                                  Constants                                 */
/* -------------------------------------------------------------------------- */

/// A string literal storing all distinct opcodes in core Bril
#[allow(dead_code)]
const OPCODE_BUFFER: &str =
    "addmulsubdiveqltgtlegenotandorjmpbrcallretidprintnopconst";

/// There are 20 distinct opcodes in core Bril
const NUM_OPCODES: usize = 20;

/// Default length of the args array
/// (Rust `Vec`s are initialized with a capacity that is a power of 2,
/// we pick 64 since that seems like a reasonable upper bound for the no. of
/// variables in a Bril function)
const NUM_ARGS: usize = 64;

/// SImilarly, we assume that Bril programs contain at most 128 dests/labels/instrs
const NUM_DESTS: usize = 128;
const NUM_LABELS: usize = 128;
const NUM_INSTRS: usize = 128;

/// Each pair contains the `(start idx, end idx)` of the opcode in `OPCODES`.     
/// Note that both start and indexes are inclusive.
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

/* -------------------------------------------------------------------------- */
/*                                 Actual code                                */
/* -------------------------------------------------------------------------- */

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
        // We reserve a buffer of size `NUM_ARGS` that contains
        // all the variables used in this function
        // (Note: this vec is heap-allocated for now, but later on we will convert
        // it to a slice)
        // We also do the same for dests and labels
        let mut all_args: Vec<&str> = Vec::with_capacity(NUM_ARGS);
        let mut all_dests: Vec<&str> = Vec::with_capacity(NUM_DESTS);
        let mut all_labels: Vec<&str> = Vec::with_capacity(NUM_LABELS);

        let name = func["name"]
            .as_str()
            .expect("Expected `name` to be a string");
        println!("{}", name);
        let instrs = func["instrs"]
            .as_array()
            .expect("Expected `instrs` to be a JSON array");

        // `all_instrs` is a temporary vec that stores all the `Instr` structs
        // that we create (we'll convert this vec to a slice after the loop below)
        let mut all_instrs: Vec<Instr> = Vec::with_capacity(NUM_INSTRS);

        for instr in instrs {
            if let Some(label) = instr["label"].as_str() {
                // Instruction is a label, doesn't have an opcode
                // TODO: figure out how to handle labels
                println!("Encountered label {}", label);
                continue;
            } else {
                let opcode: Opcode =
                    serde_json::from_value(instr["op"].clone())
                        .expect("Invalid opcode");
                let opcode_idx = opcode.get_index();

                // Obtain the start/end indexes of the args,
                // (used to populate the `args` field of the `Instr` struct)
                let mut arg_idxes = None;
                if let Some(args_json_vec) = instr["args"].as_array() {
                    let args_vec: Vec<&str> = args_json_vec
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect();
                    let args_slice = args_vec.as_slice();
                    let start_idx = all_args.len();
                    all_args.extend_from_slice(args_slice);
                    let end_idx = all_args.len() - 1;
                    arg_idxes = Some((start_idx, end_idx));
                    // let args_copy = &all_args.as_slice()[start_idx..=end_idx];
                    // println!("args = {:?}", args_copy);
                }

                // Populate the `dest` field of the `Instr` struct
                let mut dest_idx = None;
                if let Some(dest) = instr["dest"].as_str() {
                    dest_idx = Some(all_dests.len() as usize);
                    all_dests.push(dest);
                    // let dest_copy = all_dests.as_slice()[dest_idx.unwrap()];
                    // println!("dest = {:?}", dest_copy);
                }

                // Populate the `ty` field of the `Instr` struct
                let mut ty = None;
                if let Ok(instr_ty) =
                    serde_json::from_value::<Type>(instr["type"].clone())
                {
                    ty = Some(instr_ty);
                }

                // Populate the `value` field of the `Instr` struct
                let mut value = None;
                if let Some(int_value) = instr["value"].as_i64() {
                    value = Some(BrilValue::IntVal(int_value));
                } else if let Some(b) = instr["value"].as_bool() {
                    value = Some(BrilValue::BoolVal(b));
                }

                // Populate the `labels` field of the `Instr` struct
                let mut labels_idxes = None;
                if let Some(labels_json_vec) = instr["labels"].as_array() {
                    let labels_vec: Vec<&str> = labels_json_vec
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect();
                    let labels_slice = labels_vec.as_slice();
                    let start_idx = all_labels.len();
                    all_labels.extend_from_slice(labels_slice);
                    let end_idx = all_labels.len() - 1;
                    labels_idxes = Some((start_idx, end_idx));
                    // let labels_copy =
                    //     &all_labels.as_slice()[start_idx..=end_idx];
                    // println!("labels = {:?}", labels_copy);
                }

                let instr = Instr {
                    op: opcode_idx,
                    args: arg_idxes,
                    dest: dest_idx,
                    ty,
                    labels: labels_idxes,
                    value,
                };
                all_instrs.push(instr);
            }
        }
        // Convert the args/dest/labels/instrs vecs into slices
        let _args_slice: &[&str] = all_args.as_slice();
        let _dest_slice: &[&str] = &all_dests.as_slice();
        let _labels_slice: &[&str] = &all_labels.as_slice();

        let all_instrs_slice = &all_instrs.as_slice();
        for instr in *all_instrs_slice {
            println!("{}", instr);
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                               Pretty-Printing                              */
/* -------------------------------------------------------------------------- */

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Look up the actual Opcode corresponding to the op index in the struct
        let (start_idx, end_idx) = OPCODE_IDX[self.op];
        let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
        write!(f, "op: {:5}\t", op_str)?;
        if let Some(dest) = &self.dest {
            // TODO: use the index to look up into the dest array and display
            // the actual dest instead
            write!(f, "dest: {:2}\t", dest)?;
        }
        if let Some(ty) = &self.ty {
            write!(f, "type: {:5}\t", ty)?;
        }
        if let Some(value) = &self.value {
            write!(f, "value: {:5}\t", value)?;
        }
        if let Some(args) = &self.args {
            // TODO: use the index to look up into the args array and display
            // the actual dest instead
            write!(f, "args: {:?}\t", args)?;
        }
        if let Some(labels) = &self.labels {
            // TODO: use the index to look up into the displays array and display
            // the actual dest instead
            write!(f, "labels: {:?}", labels)?;
        }
        Ok(())
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
        }
    }
}

impl Type {
    /// Converts a `Type` to its string representation
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Type::Int => "int",
            Type::Bool => "bool",
        }
    }
}

impl fmt::Display for BrilValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrilValue::IntVal(n) => write!(f, "{}", n),
            BrilValue::BoolVal(b) => write!(f, "{}", b),
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                                    Tests                                   */
/* -------------------------------------------------------------------------- */
#[cfg(test)]
mod tests {
    use super::*;

    // We use `strum` to iterate over every variant in the `Opcode` enum easily
    use strum::IntoEnumIterator;

    /// Test that opcode serialization is correct
    /// (what this test does is it converts the opcode to a string using `serde`,
    /// and checks that the corresponding substring when we index into `OPCODES`
    /// is the same)
    #[test]
    fn test_opcode_serialization_correct() {
        for opcode in Opcode::iter() {
            let json: serde_json::Value = serde_json::json!(opcode);
            let deserialized_op: serde_json::Value =
                serde_json::from_value(json).expect("trouble deserializing");
            let serde_op_str = deserialized_op.as_str().unwrap();
            let op_str = opcode.as_str();
            assert_eq!(serde_op_str, op_str);
        }
    }

    /// Checks that for all opcodes, their start/end indexes in `OPCODE_IDX` are correct
    #[test]
    fn test_opcode_indexes_correct() {
        for opcode in Opcode::iter() {
            let idx = opcode.get_index();
            let (start_idx, end_idx) = OPCODE_IDX[idx];
            let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
            assert_eq!(opcode.as_str(), op_str);
        }
    }
}
