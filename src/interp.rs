#![allow(dead_code)]

use std::collections::HashMap;

use crate::types::*;

pub type Envrionment = HashMap<String, BrilValue>;

pub fn execute<T: std::io::Write, U: std::io::Write>(
    view: &InstrView,
    env: Envrionment,
    out: T
) -> Result<(), String> {
    for instr in view.instrs.iter() {
        let instr_type = instr.get_instr_kind();
        match instr_type {
            InstrKind::Const => todo!(),
            InstrKind::ValueOp => todo!(),
            InstrKind::EffectOp => todo!(),
            InstrKind::Nop => todo!(),
        }
    }

    Ok(())
}