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

/// Interprets a unary value operation (`not` and `id`)
/// (panics if `op` is not an unop)
pub fn interp_unop<'a>(
    instr_view: &'a InstrView,
    op: Opcode,
    instr: &FlatInstr,
    env: &mut Environment<'a>,
) {
    if !Opcode::is_unop(op) {
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

    if !Opcode::is_binop(op) {
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
pub fn execute<'a>(
    instr_view: &'a InstrView,
    env: &mut Environment<'a>,
) -> Result<(), String> {
    for instr in instr_view.instrs.iter() {
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
                } else {
                    todo!()
                }
            }
            InstrKind::Nop => {
                continue;
            }
        }
    }
    Ok(())
}
