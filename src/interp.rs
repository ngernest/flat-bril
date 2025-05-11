#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::str;

use crate::types::*;

// We map strings to values
// (strings are the canonical representation of variables)
pub type Environment<'a> = HashMap<&'a str, BrilValue>;

pub fn execute<'a>(
    view: &'a InstrView,
    env: &mut Environment<'a>,
) -> Result<(), String> {
    for instr in view.instrs.iter() {
        let instr_type = instr.get_instr_kind();
        let op: Opcode = Opcode::u32_to_opcode(instr.op);
        match instr_type {
            InstrKind::Const => {
                let (dest_start, dest_end): (u32, u32) = instr.dest.into();
                let dest_start = dest_start as usize;
                let dest_end = dest_end as usize;
                let dest: &'a str =
                    str::from_utf8(&view.var_store[dest_start..=dest_end])
                        .expect("invalid utf-8");
                let ty = instr.ty;
                let value: BrilValue =
                    instr.value.try_into().expect("Encountered a null value");

                // Extend the environment so that `dest |-> value`
                env.insert(dest, value);
            }
            InstrKind::ValueOp => todo!(),
            InstrKind::EffectOp => {
                if let Opcode::Print = op {
                    let (arg_start, arg_end): (u32, u32) = instr.args.into();
                    let arg_start = arg_start as usize;
                    let arg_end = arg_end as usize;
                    let arg_idxes_vec =
                        &view.arg_idxes_store[arg_start..=arg_end];
                    assert!(
                        arg_idxes_vec.len() == 1,
                        "print has more than 1 arg, malformed"
                    );

                    let (arg_start_idx, arg_end_idx): (u32, u32) =
                        arg_idxes_vec[0].into();
                    let arg_start_idx = arg_start_idx as usize;
                    let arg_end_idx = arg_end_idx as usize;
                    let arg: &str = str::from_utf8(
                        &view.var_store[arg_start_idx..=arg_end_idx],
                    )
                    .expect("invalid utf-8");

                    println!("{arg}");
                } else {
                    todo!()
                }
            }
            InstrKind::Nop => todo!(),
        }
    }

    Ok(())
}
