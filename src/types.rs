use serde::{Deserialize, Serialize};
use std::fmt;
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
/// - We can store the actual `type` and `value` inline in the `Instr` struct
///   (since they're either an int or a bool,
///   i.e. they don't need to be heap-allocated)
/// - `dest` stores the start & end indices (inclusive) of the byte representation
///    of the string in the `all_vars` byte vector (see `flatten.rs`)
/// - `args` and `labels` contains the start & end indices (inclusive)
///   in their index vectors (see `all_args_idxes` & `all_labels_idxes` in `flatten.rs`)
/// - For `args` and `labels` we have 2 layers of indirection since
///   an instruction can have multiple args/labels, so
///   `(start, end) = instr.arg ==> all_args_idxes[start..=end] ==> all_vars[...]`
/// - (Well-formedness condition: we must have end_idx >= start_idx always)
pub struct Instr {
    pub op: u32,
    pub dest: Option<(u32, u32)>,
    pub ty: Option<Type>,
    pub value: Option<BrilValue>,
    pub args: Option<(u32, u32)>,
    pub labels: Option<(u32, u32)>,
    pub funcs: Option<(u32, u32)>,
}

/// Primitive types in core Bril are either `int` or `bool`
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
}

/// Bril values are either 64-bit integers or bools.   
/// Note: We call this enum `BrilValue` to avoid namespace clashes
/// with `serde_json::Value`
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum BrilValue {
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
    pub fn get_buffer_start_end_indexes(&self) -> (usize, usize) {
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
    pub fn as_str(&self) -> &str {
        let (start_idx, end_idx) = self.get_buffer_start_end_indexes();
        &OPCODE_BUFFER[start_idx..=end_idx]
    }

    /// Converts an opcode's index in `OPCODE_IDX` to a `String` representation
    /// of an opcode
    pub fn op_idx_to_op_str(op_idx: usize) -> String {
        let (start_idx, end_idx) = OPCODE_IDX[op_idx];
        let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
        op_str.to_string()
    }
}

/// Struct that stores all the instrs and the args/dest/labels/funcs arrays
/// in the same place (note: we create one `InstrStore` per Bril function)
/// - The `func_name` field stores the name of the Bril function
///   corresponding to this `InstrStore`
/// - args_idxes_stores |-> var_store
/// - labels_idxes_store |-> labels_store
/// - there's only one function so funcs_store can just be Vec<u8>
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InstrStore {
    pub func_name: Vec<u8>,
    pub var_store: Vec<u8>,
    pub args_idxes_store: Vec<(u32, u32)>,
    pub labels_idxes_store: Vec<(u32, u32)>,
    pub labels_store: Vec<u8>,
    pub funcs_store: Vec<u8>,
    pub instrs: Vec<Instr>,
}

/* -------------------------------------------------------------------------- */
/*                                  Constants                                 */
/* -------------------------------------------------------------------------- */

/// A string literal storing all distinct opcodes in core Bril
#[allow(dead_code)]
pub const OPCODE_BUFFER: &str =
    "addmulsubdiveqltgtlegenotandorjmpbrcallretidprintnopconst";

/// There are 20 distinct opcodes in core Bril
pub const NUM_OPCODES: usize = 20;

/// Default length of the args array
/// (Rust `Vec`s are initialized with a capacity that is a power of 2,
/// we pick 64 since that seems like a reasonable upper bound for the no. of
/// variables in a Bril function)
pub const NUM_ARGS: usize = 64;

/// Variables are just a way to interpret dests/args, we assume there are 128 of them
pub const NUM_VARS: usize = 128;

/// Similarly, we assume that Bril programs contain at most 128 dests/labels/instrs
pub const NUM_LABELS: usize = 128;
pub const NUM_INSTRS: usize = 128;

/// The only core Bril instruction with a `funcs` field is `call`,
/// whose `funcs` field is just a length-1 list, so we can get away with making
/// `NUM_FUNCS` a really small power of 2, like 8.
pub const NUM_FUNCS: usize = 8;

/// Each pair contains the `(start idx, end idx)` of the opcode in `OPCODES`.     
/// Note that both start and indexes are inclusive.
pub const OPCODE_IDX: [(usize, usize); NUM_OPCODES] = [
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
/*                               Pretty-Printing                              */
/* -------------------------------------------------------------------------- */

// /// Pretty-prints an `Instr`, using the indexes in the `Instr` struct to fetch
// /// the appropriate elements in the other argument slices.
// /// Note: via the magic of deref coercion, we can just pass in references to
// /// vectors (i.e. `&Vec<&str>`'s) as arguments to this function,
// /// and Rust will automatically convert them to `&[&str]`!
// pub fn print_instr(
//     instr: &Instr,
//     args: &[&str],
//     dests: &[&str],
//     labels: &[&str],
//     funcs: &[&str],
// ) {
//     // Look up the actual Opcode corresponding to the op index in the struct
//     let (start_idx, end_idx) = OPCODE_IDX[instr.op as usize];
//     let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
//     print!("\top: {:5}\t", op_str);

//     if let Some(dest_idx) = &instr.dest {
//         print!("\tdest: {:2}\t", dests[*dest_idx as usize]);
//     }

//     if let Some(ty) = &instr.ty {
//         print!("\ttype: {:5}\t", ty);
//     }
//     if let Some(value) = &instr.value {
//         print!("\tvalue: {:5}\t", value);
//     }

//     if let Some((args_start, args_end)) = &instr.args {
//         let args_start = *args_start as usize;
//         let args_end = *args_end as usize;
//         print!("\targs: {:?}\t", &args[args_start..=args_end]);
//     }
//     if let Some((labels_start, labels_end)) = &instr.labels {
//         let labels_start = *labels_start as usize;
//         let labels_end = *labels_end as usize;
//         print!("\tlabels: {:?}", &labels[labels_start..=labels_end]);
//     }
//     if let Some((funcs_start, funcs_end)) = &instr.funcs {
//         let funcs_start = *funcs_start as usize;
//         let funcs_end = *funcs_end as usize;
//         print!("\tfuncs: {:?}", &funcs[funcs_start..=funcs_end]);
//     }

//     println!();
// }

/// Note: prefer the `print_instr` function above over the implementation of
/// the `Display` trait for `Instr`, since the former actually prints out
/// what the concrete values are for each field in the `Instr`
/// (the `Display` trait just prints out the indexes, not the actual values)
// impl fmt::Display for Instr {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // Look up the actual Opcode corresponding to the op index in the struct
//         let (start_idx, end_idx) = OPCODE_IDX[self.op as usize];
//         let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
//         write!(f, "op: {:5}\t", op_str)?;
//         if let Some(dest) = &self.dest {
//             write!(f, "dest: {:2}\t", dest)?;
//         }
//         if let Some(ty) = &self.ty {
//             write!(f, "type: {:5}\t", ty)?;
//         }
//         if let Some(value) = &self.value {
//             write!(f, "value: {:5}\t", value)?;
//         }
//         if let Some(args) = &self.args {
//             write!(f, "args: {:?}\t", args)?;
//         }
//         if let Some(labels) = &self.labels {
//             write!(f, "labels: {:?}", labels)?;
//         }
//         Ok(())
//     }
// }

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
