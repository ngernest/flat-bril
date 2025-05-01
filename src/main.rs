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
/// - `args` and `labels` contain the start and end indices (inclusive)
///   of the relevant elements in their respective arrays
///   (Well-formedness condition: we must have end_idx >= start_idx always
///   for the `args` and `labels` fields)
struct Instr {
    op: u32,
    dest: Option<u32>,
    ty: Option<Type>,
    value: Option<BrilValue>,
    args: Option<(u32, u32)>,
    labels: Option<(u32, u32)>,
    funcs: Option<(u32, u32)>,
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

#[allow(dead_code)]
impl Instr {
    /// Creates a new `Instr` struct with the `op` field set to `opcode_idx`,
    /// and all other fields initialized to `None`
    pub fn init(opcode_idx: u32) -> Self {
        Instr {
            op: opcode_idx,
            dest: None,
            ty: None,
            value: None,
            args: None,
            labels: None,
            funcs: None,
        }
    }

    /// Creates a new `Instr` struct with all fields populated according
    /// to the arguments supplied to this function
    pub fn new(
        op: u32,
        dest: Option<u32>,
        ty: Option<Type>,
        value: Option<BrilValue>,
        args: Option<(u32, u32)>,
        labels: Option<(u32, u32)>,
        funcs: Option<(u32, u32)>,
    ) -> Self {
        Instr {
            op,
            dest,
            ty,
            value,
            args,
            labels,
            funcs,
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

    /// Converts an `Opcode` to a `&str` using the `(start_idx, end_idx)`
    /// obtained from `Instr::get_buffer_start_end_indexes`.
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

/// Similarly, we assume that Bril programs contain at most 128 dests/labels/instrs
const NUM_DESTS: usize = 128;
const NUM_LABELS: usize = 128;
const NUM_INSTRS: usize = 128;

/// The only core Bril instruction with a `funcs` field is `call`,
/// whose `funcs` field is just a length-1 list, so we can get away with making
/// `NUM_FUNCS` a really small power of 2, like 8.
const NUM_FUNCS: usize = 8;

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

/// Takes in a JSON function representing one single Bril function,
/// and returns a vector containing the flattened instructions in the function
/// (in the same order)
fn create_instrs(func_json: serde_json::Value) -> Vec<Instr> {
    // We reserve a buffer of size `NUM_ARGS` that contains
    // all the variables used in this function
    // (Note: this vec is heap-allocated for now, but later on we will convert
    // it to a slice)
    // We also do the same for dests, labels and funcs

    // TODO: maybe use the `smallvec` or `arrayvec` libraries to create
    // these vectors in the future?
    // (these are specialized short vectors which minimize heap allocations)
    // ^^ we should only switch to these after benchmarking the current impl tho
    let mut all_args: Vec<&str> = Vec::with_capacity(NUM_ARGS);
    let mut all_dests: Vec<&str> = Vec::with_capacity(NUM_DESTS);
    let mut all_labels: Vec<&str> = Vec::with_capacity(NUM_LABELS);
    let mut all_funcs: Vec<&str> = Vec::with_capacity(NUM_FUNCS);

    let func_name = func_json["name"]
        .as_str()
        .expect("Expected `name` to be a string");
    println!("@{func_name}");
    let instrs = func_json["instrs"]
        .as_array()
        .expect("Expected `instrs` to be a JSON array");

    // `all_instrs` is a temporary vec that stores all the `Instr` structs
    // that we create (we'll convert this vec to a slice after the loop below)
    let mut all_instrs: Vec<Instr> = Vec::with_capacity(NUM_INSTRS);

    for instr in instrs {
        if let Some(label) = instr["label"].as_str() {
            // Instruction is a label, doesn't have an opcode
            println!(".{label}");
            continue;
        } else {
            let opcode: Opcode = serde_json::from_value(instr["op"].clone())
                .expect("Invalid opcode");
            let opcode_idx = opcode.get_index() as u32;

            // Obtain the start/end indexes of the args,
            // (used to populate the `args` field of the `Instr` struct)
            let mut arg_idxes = None;
            if let Some(args_json_vec) = instr["args"].as_array() {
                let args_vec: Vec<&str> =
                    args_json_vec.iter().map(|v| v.as_str().unwrap()).collect();
                let args_slice = args_vec.as_slice();
                let start_idx = all_args.len();
                all_args.extend_from_slice(args_slice);
                let end_idx = all_args.len() - 1;
                arg_idxes = Some((start_idx as u32, end_idx as u32));
            }

            // Populate the `dest` field of the `Instr` struct
            let mut dest_idx = None;
            if let Some(dest) = instr["dest"].as_str() {
                dest_idx = Some(all_dests.len() as u32);
                all_dests.push(dest);
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
                let start_idx = all_labels.len();
                all_labels.extend(labels_vec);
                let end_idx = all_labels.len() - 1;
                labels_idxes = Some((start_idx as u32, end_idx as u32));
            }

            // Handle `func` field in `Instr` struct
            let mut funcs_idxes = None;
            if let Some(funcs_json_vec) = instr["funcs"].as_array() {
                let funcs_vec: Vec<&str> = funcs_json_vec
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect();
                let start_idx = all_funcs.len();
                all_funcs.extend(funcs_vec);
                let end_idx = all_funcs.len() - 1;
                funcs_idxes = Some((start_idx as u32, end_idx as u32));
            }

            let instr = Instr {
                op: opcode_idx,
                args: arg_idxes,
                dest: dest_idx,
                ty,
                labels: labels_idxes,
                value,
                funcs: funcs_idxes,
            };
            print_instr(&instr, &all_args, &all_dests, &all_labels, &all_funcs);
            all_instrs.push(instr);
        }
    }
    all_instrs
}

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
        let _instrs = create_instrs(func.clone());
        // TODO: figure out what to do with _instrs
    }
}

/* -------------------------------------------------------------------------- */
/*                               Pretty-Printing                              */
/* -------------------------------------------------------------------------- */

/// Pretty-prints an `Instr`, using the indexes in the `Instr` struct to fetch
/// the appropriate elements in the other argument slices
fn print_instr(
    instr: &Instr,
    args_vec: &Vec<&str>,
    dests_vec: &Vec<&str>,
    labels_vec: &Vec<&str>,
    funcs_vec: &Vec<&str>,
) {
    // Look up the actual Opcode corresponding to the op index in the struct
    let (start_idx, end_idx) = OPCODE_IDX[instr.op as usize];
    let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
    print!("\top: {:5}\t", op_str);

    if let Some(dest_idx) = &instr.dest {
        print!("\tdest: {:2}\t", dests_vec[*dest_idx as usize]);
    }

    if let Some(ty) = &instr.ty {
        print!("\ttype: {:5}\t", ty);
    }
    if let Some(value) = &instr.value {
        print!("\tvalue: {:5}\t", value);
    }

    if let Some((args_start, args_end)) = &instr.args {
        let args_start = *args_start as usize;
        let args_end = *args_end as usize;
        print!("\targs: {:?}\t", &args_vec[args_start..=args_end]);
    }
    if let Some((labels_start, labels_end)) = &instr.labels {
        let labels_start = *labels_start as usize;
        let labels_end = *labels_end as usize;
        print!("\tlabels: {:?}", &labels_vec[labels_start..=labels_end]);
    }
    if let Some((funcs_start, funcs_end)) = &instr.funcs {
        let funcs_start = *funcs_start as usize;
        let funcs_end = *funcs_end as usize;
        print!("\tfuncs: {:?}", &funcs_vec[funcs_start..=funcs_end]);
    }

    println!();
}

/// Note: prefer the `print_instr` function above over the implementation of
/// the `Display` trait for `Instr`, since the former actually prints out
/// what the concrete values are for each field in the `Instr`
/// (the `Display` trait just prints out the indexes, not the actual values)
impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Look up the actual Opcode corresponding to the op index in the struct
        let (start_idx, end_idx) = OPCODE_IDX[self.op as usize];
        let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
        write!(f, "op: {:5}\t", op_str)?;
        if let Some(dest) = &self.dest {
            write!(f, "dest: {:2}\t", dest)?;
        }
        if let Some(ty) = &self.ty {
            write!(f, "type: {:5}\t", ty)?;
        }
        if let Some(value) = &self.value {
            write!(f, "value: {:5}\t", value)?;
        }
        if let Some(args) = &self.args {
            write!(f, "args: {:?}\t", args)?;
        }
        if let Some(labels) = &self.labels {
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
    use std::{fs::File, io::BufReader, path::Path};

    use super::*;

    // We use `strum` to iterate over every variant in the `Opcode` enum easily
    use strum::IntoEnumIterator;

    /// Test that opcode serialization is correct
    /// (what this test does is it converts the opcode to a string using `serde`,
    /// and checks that the corresponding substring when we index into `OPCODES`
    /// is the same)
    #[test]
    fn test_opcode_serialization_round_trip() {
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

    /// Test that all the flattened instructions corresponding to `test/add.bril`
    /// are well-formed (i.e. for pairs of indices, the end index is always
    /// >= the start index)
    #[test]
    fn test_add_bril_instrs_wf() {
        let path = Path::new("test/add.json");
        let file = File::open(path).expect("Unable to open file");
        let reader = BufReader::new(file);

        let json: serde_json::Value =
            serde_json::from_reader(reader).expect("Unable to parse JSON");
        let functions = json["functions"]
            .as_array()
            .expect("Expected `functions` to be a JSON array");
        let instrs: Vec<Instr> = create_instrs(functions[0].clone());
        for instr in instrs {
            if let Some((args_start, args_end)) = instr.args {
                assert!(
                    args_end >= args_start,
                    "{} >= {} is false",
                    args_end,
                    args_start
                );
            }
            if let Some((labels_start, labels_end)) = instr.labels {
                assert!(
                    labels_end >= labels_start,
                    "{} >= {} is false",
                    labels_end,
                    labels_start
                );
            }
        }
    }
}
