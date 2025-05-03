use crate::types::*;

/* -------------------------------------------------------------------------- */
/*                                 Actual code                                */
/* -------------------------------------------------------------------------- */

/// Takes in a JSON function representing one single Bril function,
/// and returns a vector containing the flattened instructions in the function
/// (in the same order)
pub fn create_instrs(func_json: serde_json::Value) -> Vec<Instr> {
    // We reserve a buffer of size `NUM_ARGS` that contains
    // all the variables used in this function.
    // We also do the same for dests, labels and funcs.

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
