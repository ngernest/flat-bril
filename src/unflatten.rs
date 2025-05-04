use crate::types::*;

#[allow(unused_imports)]
use serde_json::json;

// TODO: figure out how to handle labels when unflattening since they're
// not currently represented in the `InstrStore`
// (labels are just printed directly to stdout when we flatten)

/// Takes an `InstrStore` (flattened instrs + arrays storing args/dests etc.)
/// corresponding to a Bril function and returns its JSON representation
#[allow(dead_code)]
pub fn unflatten_instrs<'a>(instr_store: &InstrStore) -> serde_json::Value {
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
        let mut val_str: Option<String> = None;
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
        let mut dest: Option<&[u8]> = None;
        if let Some((start_idx, end_idx)) = instr.dest {
            let start_idx = start_idx as usize;
            let end_idx = end_idx as usize;
            dest = Some(&instr_store.var_store[start_idx..=end_idx]);
        } else {
            // There is no `dest` field,
            // so the instr is an effect op (not a value op)
            is_value_op = false;
        }

        // Convert the (start_idx, end_idx) for args in the instr to
        // an actual list of strings (by doing `args_store[start_idx..=end_idx]`)
        let mut args: Vec<&[u8]> = vec![];
        if let Some((start_idx, end_idx)) = instr.args {
            let start_idx = start_idx as usize;
            let end_idx = end_idx as usize;
            let arg_idxes: Vec<(u32, u32)> =
                instr_store.args_idxes_store[start_idx..=end_idx].to_vec();
            for (start, end) in arg_idxes {
                let start = start as usize;
                let end = end as usize;
                let arg: &[u8] = &instr_store.var_store[start..=end];
                args.push(arg);
            }
        }

        // Convert the (start_idx, end_idx) for labels in the instr to
        // an actual list of strings
        let mut labels: Vec<&[u8]> = vec![];
        if let Some((start_idx, end_idx)) = instr.labels {
            let start_idx = start_idx as usize;
            let end_idx = end_idx as usize;
            let labels_idxes: Vec<(u32, u32)> =
                instr_store.labels_idxes_store[start_idx..=end_idx].to_vec();
            for (start, end) in labels_idxes {
                let start = start as usize;
                let end = end as usize;
                let label: &[u8] = &instr_store.labels_store[start..=end];
                labels.push(label);
            }
        }

        // Convert the (start_idx, end_idx) for funcs in the instr to
        // an actual list of strings
        let mut funcs: Option<&[u8]> = None;
        if let Some((start_idx, end_idx)) = instr.funcs {
            let start_idx = start_idx as usize;
            let end_idx = end_idx as usize;
            funcs = Some(&instr_store.funcs_store[start_idx..=end_idx]);
        }

        let args_for_json: Vec<&str> = args
            .iter()
            .map(|arg| str::from_utf8(arg).expect("invalid utf-8"))
            .collect();
        let labels_for_json: Vec<&str> = labels
            .iter()
            .map(|label| str::from_utf8(label).expect("invalid utf-8"))
            .collect();

        let dest_for_json =
            str::from_utf8(dest.expect("missing dest")).expect("invalid utf-8");

        let mut funcs_for_json = vec![];
        if Option::is_some(&funcs) {
            let func_str = str::from_utf8(funcs.expect("missing funcs"))
                .expect("invalid utf-8");
            funcs_for_json.push(func_str)
        }

        // Build a JSON object corresponding to the flattened instruction
        if is_value_op {
            let _instr_json = serde_json::json!({
              "op": op_str,
              "dest": dest_for_json,
              "type": ty_str.expect("Expected string representing a type"),
              "value": val_str.expect("Expected value string"),
              "args": args_for_json,
              "labels": labels_for_json,
              "funcs": funcs_for_json
            });
        } else {
            let _instr_json = serde_json::json!({
              "op": op_str,
              "args": args,
              "labels": labels,
              "funcs": funcs
            });
        }
    }

    todo!("actually return a list of JSON objects here")
}
