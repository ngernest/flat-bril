use crate::types::*;

#[allow(unused_imports)]
use serde_json::json;

/// Takes an `InstrStore` (flattened instrs + arrays storing args/dests etc.)
/// corresponding to a Bril function and returns its JSON representation
#[allow(dead_code)]
pub fn unflatten_instrs<'a>(instr_store: &'a InstrStore) -> serde_json::Value {
    for instr in &instr_store.instrs {
        // Flag for tracking whether the instr is a value op or an effect op
        let mut is_value_op = true;

        let op_str = Opcode::op_idx_to_op_str(instr.op as usize);

        // Extract the `ty` field of the instr as a string
        let mut ty_str = None;
        if let Some(ty) = &instr.ty {
            ty_str = Some(ty.as_str());
        }

        // Convert the `value` field in the `Instr` to a string
        let mut val_str = None;
        if let Some(val) = &instr.value {
            match val {
                BrilValue::BoolVal(b) => {
                    val_str = Some(format!("{b}"));
                }
                BrilValue::IntVal(n) => {
                    val_str = Some(format!("{n}"));
                }
            };
        }

        // Convert the `dest` index of the instr to an actual string
        // containing the dest
        let mut dest = None;
        if let Some(dest_idx) = instr.dest {
            dest = Some(instr_store.dests_store[dest_idx as usize]);
        } else {
            // There is no `dest` field,
            // so the instr is an effect op (not a value op)
            is_value_op = false;
        }

        // TODO: handle args/funcs/labels
        // (these are more tricky since we have start_idx and end_idx)

        // Build a JSON object corresponding to the flattened instruction
        if is_value_op {
            let _instr_json = serde_json::json!({
              "op": op_str,
              "dest": dest.expect("Expected dest string"),
              "type": ty_str.expect("Expected type string"),
              "value": val_str.expect("Expected value string")
            });
        } else {
            let _instr_json = serde_json::json!({
              "op": op_str
            });
            todo!("figure out how to implement json for effect op")
        }
    }

    todo!("actually return a list of JSON objects here")
}
