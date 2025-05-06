use core::panic;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;
use strum_macros::EnumIter;
use zerocopy::IntoBytes;

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

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
/// A type that represents an "item" that we encounter in a Bril file,
/// where an item is either an instruction `Instr` or a `Label` (represented
/// by the start & end indices of the label in the global
/// labels byte sequence)
pub enum InstrOrLabel {
    Instr(Instr),
    Label((u32, u32)),
}

/// Struct representation of the pair `(i32, i32)`
/// (we need this b/c `zerocopy` doesn't work for tuples)
#[repr(C)]
#[derive(Debug, Clone, IntoBytes)]
pub struct I32Pair {
    pub first: i32,
    pub second: i32,
}

/// Flattened representation of an instruction, amenable to `zerocopy`
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FlatInstr {
    pub op: i32,
    pub dest: I32Pair,
    pub args: I32Pair,
    pub labels: I32Pair,
    pub funcs: I32Pair,
    pub ty: Type,
    pub value: BrilValue,
}

#[derive(Debug, Clone)]
pub enum InstrKind {
    Const,
    ValueOp,
    EffectOp,
}

/// Primitive types in core Bril are either `int` or `bool`
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Deserialize, IntoBytes)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
    Null,
}

/// Bril values are either 64-bit integers or bools.   
/// Note: We call this enum `BrilValue` to avoid namespace clashes
/// with `serde_json::Value`
/// - The `Null` constructor is only used for flattening
///   (to indicate the absence of a value)
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum BrilValue {
    IntVal(i64),
    BoolVal(bool),
    Null,
}

#[allow(dead_code)]
impl Instr {
    pub fn get_instr_kind(&self) -> InstrKind {
        use Opcode::*;
        let op = Opcode::u32_to_opcode(self.op);
        match op {
            Const => InstrKind::Const,
            Print | Jmp | Br | Ret => InstrKind::EffectOp,
            Call => {
                // Function calls can be both value op and effect op
                // depending on whether the `dest` field of the instr
                // is present
                if let Some((_, _)) = self.dest {
                    InstrKind::ValueOp
                } else {
                    InstrKind::EffectOp
                }
            }
            _ => InstrKind::ValueOp,
        }
    }
}

#[derive(
    Debug, PartialEq, Clone, Deserialize, Serialize, EnumIter, FromPrimitive,
)]
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
    /// Converts a `u32` value to the corresponding `Opcode`
    /// - Panics if the `u32` value can't be converted
    pub fn u32_to_opcode(v: u32) -> Self {
        let possible_op: Option<Opcode> =
            num_traits::FromPrimitive::from_u32(v);
        use Opcode::*;
        match possible_op {
            // Arithmetic
            Some(Add) => Add,
            Some(Mul) => Mul,
            Some(Sub) => Sub,
            Some(Div) => Div,

            // Comparison
            Some(Eq) => Eq,
            Some(Lt) => Lt,
            Some(Gt) => Gt,
            Some(Le) => Le,
            Some(Ge) => Ge,

            // Logic operations
            Some(Not) => Not,
            Some(And) => And,
            Some(Or) => Or,

            // Control flow
            Some(Jmp) => Jmp,
            Some(Br) => Br,
            Some(Call) => Call,
            Some(Ret) => Ret,

            // Misc operations
            Some(Id) => Id,
            Some(Print) => Print,
            Some(Nop) => Nop,
            Some(Const) => Const,

            None => panic!("Couldn't convert {} to an opcode", v),
        }
    }

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

/// Struct representing the two components of an argument to a Bril function:
/// 1. The argument name, represented by the start & end indexes in the
/// `var_store` vector of `InstrStore`
/// 2. The type of the argument
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FuncArg {
    pub arg_name_idxes: (u32, u32),
    pub arg_type: Type,
}

/// Struct that stores all the instrs and the args/dest/labels/funcs arrays
/// in the same place (note: we create one `InstrStore` per Bril function)
/// - The `func_name` field stores the name of the Bril function
///   corresponding to this `InstrStore`
/// - `func_args` is a list of function parameters
///    (arg type + indexes for the arg name)
/// - `func_ret_ty` is the return type of the function
///   (`None` means the function is void, i.e. has no return type)
/// - args_idxes_stores |-> var_store
/// - labels_idxes_store |-> labels_store
/// - there's only one function so `funcs_store` can just be Vec<u8>
/// - `instrs_and_labels` is a vector containing the instructions/labels in
///    the order they appear in the source Bril file
#[derive(Debug, Clone)]
pub struct InstrStore {
    pub func_name: Vec<u8>,
    pub func_args: Vec<FuncArg>,
    pub func_ret_ty: Option<Type>,
    pub var_store: Vec<u8>,
    pub args_idxes_store: Vec<(u32, u32)>,
    pub labels_idxes_store: Vec<(u32, u32)>,
    pub labels_store: Vec<u8>,
    pub funcs_store: Vec<u8>,
    pub instrs_and_labels: Vec<InstrOrLabel>,
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::Null => write!(f, "null"),
        }
    }
}

impl Type {
    /// Converts a `Type` to its string representation
    pub fn as_str(&self) -> &str {
        match self {
            Type::Int => "int",
            Type::Bool => "bool",
            Type::Null => "null",
        }
    }
}

impl fmt::Display for BrilValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrilValue::IntVal(n) => write!(f, "{}", n),
            BrilValue::BoolVal(b) => write!(f, "{}", b),
            BrilValue::Null => write!(f, "null"),
        }
    }
}
