#![allow(unused_variables)]
use std::collections::HashMap;
use std::str;

use crate::types::*;

// An environment maps variable names (`&str`s) to values
pub type Environment<'a> = HashMap<&'a str, BrilValue>;

/// Extracts the variable name (string) that occupies `start_idx` to `end_idx`
/// (inclusive) in `instr_view.var_store`
pub fn get_var<'a>(
    instr_view: &'a InstrView,
    start_idx: u32,
    end_idx: u32,
) -> &'a str {
    let start_idx = start_idx as usize;
    let end_idx = end_idx as usize;
    str::from_utf8(&instr_view.var_store[start_idx..=end_idx])
        .expect("invalid utf-8")
}

/// Extracts a vec of args (variable name strings) that correspond to the
/// `args_start` to `args_end` indices (inclusive) in `instr_view.arg_idxes_store`
pub fn get_args<'a>(
    instr_view: &'a InstrView,
    args_start: u32,
    args_end: u32,
) -> Vec<&'a str> {
    let arg_start = args_start as usize;
    let arg_end = args_end as usize;
    let args_idxes_slice = &instr_view.arg_idxes_store[arg_start..=arg_end];
    args_idxes_slice
        .iter()
        .map(|i32pair| {
            let (start_idx, end_idx) = <(u32, u32)>::from(*i32pair);
            get_var(instr_view, start_idx, end_idx)
        })
        .collect()
}

/// Extracts the label name (string) that occupies `start_idx` to `end_idx`
/// (inclusive) in `instr_view.labels_store`
pub fn get_label_name<'a>(
    instr_view: &'a InstrView,
    start_idx: u32,
    end_idx: u32,
) -> &'a str {
    let start_idx = start_idx as usize;
    let end_idx = end_idx as usize;
    str::from_utf8(&instr_view.labels_store[start_idx..=end_idx])
        .expect("invalid utf-8")
}

/// Extracts a vec of labels that correspond to the
/// `labels_start` to `labels_end` indices (inclusive) in `instr_view.labels_idxes_store`
pub fn get_labels_vec<'a>(
    instr_view: &'a InstrView,
    labels_start: u32,
    labels_end: u32,
) -> Vec<&'a str> {
    let label_start = labels_start as usize;
    let label_end = labels_end as usize;
    let labels_idxes_slice =
        &instr_view.labels_idxes_store[label_start..=label_end];
    labels_idxes_slice
        .iter()
        .map(|i32pair| {
            let (start_idx, end_idx) = <(u32, u32)>::from(*i32pair);
            get_label_name(instr_view, start_idx, end_idx)
        })
        .collect()
}

pub fn get_label_idxes<'a>(
    instr_view: &'a InstrView,
    labels_start: u32,
    labels_end: u32,
) -> &'a [I32Pair] {
    let label_start = labels_start as usize;
    let label_end = labels_end as usize;
    &instr_view.labels_idxes_store[label_start..=label_end]
}

/// Interprets a unary value operation (`not` and `id`)
/// (panics if `op` is not an unop)
pub fn interp_unop<'a>(
    instr_view: &'a InstrView,
    op: Opcode,
    instr: &FlatInstr,
    env: &mut Environment<'a>,
) {
    if !op.is_unop() {
        panic!("interp_unop called on a non-unary value operation");
    }

    let (dest_start, dest_end): (u32, u32) = instr.dest.into();
    let dest = get_var(instr_view, dest_start, dest_end);
    let (args_start, args_end): (u32, u32) = instr.args.into();
    let args = get_args(instr_view, args_start, args_end);
    assert!(
        args.len() == 1,
        "unary instruction is malformed (no. of args != 1)"
    );

    let arg = args[0];
    let value = env.get(arg).expect("arg missing from env");
    let result = match (op, value) {
        (Opcode::Not, BrilValue::BoolVal(b)) => {
            let b = bool::from(*b);
            BrilValue::BoolVal((!b).into())
        }
        (Opcode::Id, _) => *value,
        _ => {
            panic!("argument to unary instruction is ill-typed");
        }
    };
    env.insert(dest, result);
}

/// Interprets a binary value operation (panics if `op` is not a binop)
pub fn interp_binop<'a>(
    instr_view: &'a InstrView,
    op: Opcode,
    instr: &FlatInstr,
    env: &mut Environment<'a>,
) {
    use BrilValue::*;
    use Opcode::*;

    if !op.is_binop() {
        panic!("interp_binop called on a non-binary value operation");
    }

    let (dest_start, dest_end): (u32, u32) = instr.dest.into();
    let dest = get_var(instr_view, dest_start, dest_end);

    let (args_start, args_end): (u32, u32) = instr.args.into();
    let args = get_args(instr_view, args_start, args_end);
    assert!(args.len() == 2, "no. of args to arithmetic op != 2");

    let x = env.get(args[0]).expect("left operand missing from env");
    let y = env.get(args[1]).expect("right operand missing from env");

    match (x, y) {
        (IntVal(v1), IntVal(v2)) => {
            let value = match op {
                // Arithmetic
                Add => IntVal(v1.wrapping_add(*v2)),
                Sub => IntVal(v1.wrapping_sub(*v2)),
                Mul => IntVal(v1.wrapping_mul(*v2)),
                Div => IntVal(v1.wrapping_div(*v2)),
                // Comparison
                Eq => BoolVal((v1 == v2).into()),
                Ge => BoolVal((v1 >= v2).into()),
                Gt => BoolVal((v1 > v2).into()),
                Le => BoolVal((v1 <= v2).into()),
                Lt => BoolVal((v1 < v2).into()),
                _ => unreachable!(),
            };

            env.insert(dest, value);
        }
        (BoolVal(b1), BoolVal(b2)) => {
            let b1 = bool::from(*b1);
            let b2 = bool::from(*b2);
            // Logic
            let value = match op {
                And => BoolVal((b1 && b2).into()),
                Or => BoolVal((b1 || b2).into()),
                _ => unreachable!(),
            };
            env.insert(dest, value);
        }
        (_, _) => {
            panic!("operands to binop are ill-typed")
        }
    }
}

/// Interprets all the instructions in `instr_view` using the supplied `env`
pub fn interp_instr_view<'a>(
    instr_view: &'a InstrView,
    env: &mut Environment<'a>,
) -> Result<(), String> {
    let mut current_instr_ptr = 0; // Initialize program counter

    while current_instr_ptr < instr_view.instrs.len() {
        let instr = &instr_view.instrs[current_instr_ptr];
        let instr_type = instr.get_instr_kind();
        let op: Opcode = Opcode::u32_to_opcode(instr.op);
        match instr_type {
            InstrKind::Const => {
                let (dest_start, dest_end): (u32, u32) = instr.dest.into();
                let dest = get_var(instr_view, dest_start, dest_end);
                let value =
                    instr.value.try_into().expect("Encountered a null value");

                // Extend the environment so that `dest |-> value`
                env.insert(dest, value);
                current_instr_ptr += 1;
            }
            InstrKind::ValueOp => {
                if Opcode::is_binop(op) {
                    interp_binop(instr_view, op, instr, env);
                } else {
                    match op {
                        Opcode::Not | Opcode::Id => {
                            interp_unop(instr_view, op, instr, env);
                        }
                        _ => {
                            todo!("handle other value operations")
                        }
                    }
                }
                current_instr_ptr += 1;
            }
            InstrKind::EffectOp => {
                if let Opcode::Print = op {
                    let (args_start, args_end): (u32, u32) = instr.args.into();
                    let args = get_args(instr_view, args_start, args_end);
                    assert!(
                        args.len() == 1,
                        "print instruction is malformed (has != 1 arg)"
                    );

                    let arg = args[0];
                    let value_of_arg =
                        env.get(arg).expect("arg missing from env");
                    println!("{value_of_arg}");
                    current_instr_ptr += 1;
                } else if let Opcode::Jmp = op {
                    // TODO: figure out why this doesn't work for `jmp.bril`

                    // Compute the start/end idx of the label in the `labels_store`
                    let (labels_start, labels_end): (u32, u32) =
                        instr.instr_labels.into();

                    println!(
                        "original label index = {:?}, {:?}",
                        labels_start, labels_end
                    );
                    let label_idxes_slice =
                        get_label_idxes(instr_view, labels_start, labels_end);
                    assert!(
                        label_idxes_slice.len() == 1,
                        "jump instruction is malformed (has != 1 label)"
                    );

                    println!("label_idxes_slice = {:?}", label_idxes_slice);

                    let label_idxes = label_idxes_slice[0];

                    println!("label_idxes = {:?}", label_idxes);

                    let all_instrs = instr_view.instrs;
                    println!("instrs = {:#?}", all_instrs);

                    // Iterate over the list of instrs to find the index (PC)
                    // of the instr corresponding to the label
                    let pc_of_label =
                        instr_view.instrs.iter().position(|instr| {
                            let candidate_lbl_idx = instr.label;
                            candidate_lbl_idx == label_idxes
                        });

                    if let Some(new_pc) = pc_of_label {
                        // Update `current_instr_ptr` to the PC of the label
                        current_instr_ptr = new_pc;
                    } else {
                        panic!("cannot find PC corresponding to label")
                    }
                } else if let Opcode::Br = op {
                    let (args_start, args_end): (u32, u32) = instr.args.into();
                    let args = get_args(instr_view, args_start, args_end);
                    assert!(
                        args.len() == 1,
                        "br instruction must only have 1 arg"
                    );
                    let arg = args[0];
                    let value_of_arg =
                        env.get(arg).expect("arg missing from env");

                    let (labels_start, labels_end): (u32, u32) =
                        instr.instr_labels.into();
                    let labels =
                        get_labels_vec(instr_view, labels_start, labels_end);
                    assert!(
                        labels.len() == 2,
                        "br instruction is malformed (has != 2 labels)"
                    );

                    let label1 = labels[0];
                    let label2 = labels[1];
                    // TODO: finish br logic
                } else {
                    todo!()
                }
            }
            InstrKind::Nop => {
                current_instr_ptr += 1;
            }
        }
    }
    Ok(())
}
