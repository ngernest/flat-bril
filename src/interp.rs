use std::collections::HashMap;
use std::str;

use crate::types::*;

// An environment maps variable names (`&str`s) to values
pub type Environment<'a> = HashMap<&'a str, BrilValue>;

/// Extracts the variable name (string) that occupies `start_idx` to `end_idx`
/// (inclusive) in `instr_view.var_store`
pub fn get_varname_str<'a>(
    instr_view: &'a InstrView,
    start_idx: u32,
    end_idx: u32,
) -> &'a str {
    let start_idx = start_idx as usize;
    let end_idx = end_idx as usize;
    str::from_utf8(&instr_view.var_store[start_idx..=end_idx])
        .expect("invalid utf-8")
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
                let dest = get_varname_str(instr_view, dest_start, dest_end);
                let value =
                    instr.value.try_into().expect("Encountered a null value");

                // Extend the environment so that `dest |-> value`
                env.insert(dest, value);
            }
            InstrKind::ValueOp => {
                if let Opcode::Add = op {
                    let (dest_start, dest_end): (u32, u32) = instr.dest.into();
                    let dest =
                        get_varname_str(instr_view, dest_start, dest_end);

                    let (args_start, args_end): (u32, u32) = instr.args.into();
                    let arg_start = args_start as usize;
                    let arg_end = args_end as usize;
                    let args_idxes_slice =
                        &instr_view.arg_idxes_store[arg_start..=arg_end];
                    let args: Vec<&str> = args_idxes_slice
                        .iter()
                        .map(|i32pair| {
                            let (start_idx, end_idx) =
                                <(u32, u32)>::from(*i32pair);
                            get_varname_str(instr_view, start_idx, end_idx)
                        })
                        .collect();
                    assert!(
                        args.len() == 2,
                        "no. of args to arithmetic op is not 2"
                    );
                    let x = env
                        .get(args[0])
                        .expect("left operand missing from env");
                    let y = env
                        .get(args[1])
                        .expect("right operand missing from env");
                    match (x, y) {
                        (BrilValue::IntVal(vx), BrilValue::IntVal(vy)) => {
                            let value = BrilValue::IntVal(vx.wrapping_add(*vy));
                            env.insert(dest, value);
                        }
                        (_, _) => {
                            panic!("operands to arithmetic instruction are ill-typed")
                        }
                    }
                } else {
                    todo!()
                }
            }
            InstrKind::EffectOp => {
                if let Opcode::Print = op {
                    let (args_start, args_end): (u32, u32) = instr.args.into();
                    let arg_start = args_start as usize;
                    let arg_end = args_end as usize;
                    let args_idxes_slice =
                        &instr_view.arg_idxes_store[arg_start..=arg_end];
                    assert!(
                        args_idxes_slice.len() == 1,
                        "print instruction is malformed (has > 1 arg)"
                    );

                    let (arg_start_idx, arg_end_idx): (u32, u32) =
                        args_idxes_slice[0].into();
                    let arg =
                        get_varname_str(instr_view, arg_start_idx, arg_end_idx);
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
