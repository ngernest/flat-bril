use crate::types::*;

/* -------------------------------------------------------------------------- */
/*                                 Actual code                                */
/* -------------------------------------------------------------------------- */

/// Takes in a vector of JSON values (representing variables / labels),
/// a vector `global_idxes_vec` storing the start & end index of  
/// the byte representation of each var in `buffer` (a byte sequence)
/// Example:
/// - json_vec = args_json_vec
/// - global_idxes_vec = all_args_idxes
/// - buffer = all_vars
///
/// - json_vec = labels_json_vec
/// - global_idxes_vec = all_labels_idxes
/// - buffer = all_labels
pub fn flatten_instr_array_fields(
    json_vec: &Vec<serde_json::Value>,
    global_idxes_vec: &mut Vec<(u32, u32)>,
    buffer: &mut Vec<u8>,
) -> (u32, u32) {
    // Convert each JSON string in `json_vec` into a
    // `&[u8]` byte slice
    let bytes_vec: Vec<&[u8]> = json_vec
        .iter()
        .map(|v| v.as_str().unwrap().as_bytes())
        .collect();

    // `idxes_vec` stores the start & end indexes
    // of each variable in `bytes_vec` (this is necessary
    // since later on, we're concatenating all the byte slices tgt)
    let mut idxes_vec: Vec<(u32, u32)> = Vec::new();
    let mut n: u32 = 0;
    for (i, var) in bytes_vec.iter().enumerate() {
        if i == 0 {
            idxes_vec.push((0, var.len() as u32));
            n = var.len() as u32;
        } else {
            idxes_vec.push((n, n + var.len() as u32));
            n += var.len() as u32;
        }
    }

    // Compute the start & end indexes of all variables mentioned
    // by this instruction in `idxes-vec`
    let start_idx = global_idxes_vec.len();
    global_idxes_vec.extend_from_slice(idxes_vec.as_slice());
    let end_idx = global_idxes_vec.len() - 1;
    let var_idxes = (start_idx as u32, end_idx as u32);

    // Concatenate all the `&[u8]`s in `bytes_vec` into
    // one single vector of bytes
    let vars_vec: Vec<u8> = bytes_vec.concat();

    // Extend the global bytes vector of vars with the vec of
    // bytes that we just cretaed
    buffer.extend_from_slice(vars_vec.as_slice());

    var_idxes
}

/// Takes in a JSON function representing one single Bril function,
/// and returns a vector containing the flattened instructions in the function
/// (in the same order)
pub fn create_instrs(func_json: &serde_json::Value) -> Vec<Instr> {
    // We reserve a buffer of size `NUM_ARGS` that contains
    // all the variables used in this function.
    // We also do the same for dests, labels and funcs.

    // TODO: maybe use the `smallvec` or `arrayvec` libraries to create
    // these vectors in the future?
    // (these are specialized short vectors which minimize heap allocations)
    // ^^ we should only switch to these after benchmarking the current impl tho

    // `all_vars` is a vec storing the byte representation of all variables
    // that we encounter (the bytes of all the variables are concatenated
    // together into one long byte sequence)
    let mut all_vars: Vec<u8> = Vec::with_capacity(NUM_VARS * 2);

    // `all_args_idxes` stores the start & end indexes of each arg in `all_vars`
    let mut all_args_idxes: Vec<(u32, u32)> = Vec::with_capacity(NUM_ARGS);

    // `all_labels_idxes` stores the start & end indexes of each label in `all_labels`
    let mut all_labels: Vec<u8> = Vec::with_capacity(NUM_LABELS * 5);
    let mut all_labels_idxes: Vec<(u32, u32)> = Vec::with_capacity(NUM_LABELS);

    let mut all_funcs: Vec<u8> = Vec::with_capacity(NUM_FUNCS);

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

            // Obtain the start/end indexes into the all_args_idxes Vec
            // (used to populate the `args` field of the `Instr` struct)
            let mut arg_idxes = None;
            if let Some(args_json_vec) = instr["args"].as_array() {
                let (start_idx, end_idx) = flatten_instr_array_fields(
                    args_json_vec,
                    &mut all_args_idxes,
                    &mut all_vars,
                );
                arg_idxes = Some((start_idx, end_idx))
            }

            // Populate the `dest` field of the `Instr` struct
            let mut dest_idx = None;
            if let Some(dest) = instr["dest"].as_str() {
                dest_idx = Some((
                    all_vars.len() as u32,
                    (all_vars.len() + dest.as_bytes().len()) as u32,
                ));
                all_vars.extend_from_slice(dest.as_bytes());
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
                let (start_idx, end_idx) = flatten_instr_array_fields(
                    labels_json_vec,
                    &mut all_labels_idxes,
                    &mut all_labels,
                );
                labels_idxes = Some((start_idx, end_idx));
            }

            // Handle `func` field in `Instr` struct
            // Because we only handle core Bril we assume only one func is referenced
            let mut func_idx = None;
            if let Some(funcs_json_vec) = instr["funcs"].as_array() {
                let funcs_vec: Vec<&[u8]> = funcs_json_vec
                    .iter()
                    .map(|v| v.as_str().unwrap().as_bytes())
                    .collect();
                assert!(funcs_vec.len() == 1);
                let func = funcs_vec.concat();
                func_idx = Some((
                    all_funcs.len() as u32,
                    (all_funcs.len() + func.len()) as u32,
                ));
                all_funcs.extend_from_slice(func.as_slice());
            }

            let instr = Instr {
                op: opcode_idx,
                args: arg_idxes,
                dest: dest_idx,
                ty,
                labels: labels_idxes,
                value,
                funcs: func_idx,
            };
            all_instrs.push(instr);
        }
    }
    // TODO: figure out what to do with _instr_store
    // let _instr_store = InstrStore {
    //     args_store: all_args,
    //     dests_store: all_dests,
    //     labels_store: all_labels,
    //     funcs_store: all_funcs,
    //     instrs: all_instrs.clone(),
    // };
    all_instrs
}

/* -------------------------------------------------------------------------- */
/*                                    Tests                                   */
/* -------------------------------------------------------------------------- */
#[cfg(test)]
mod flatten_tests {
    use crate::flatten;

    use std::io;
    use std::{fs, fs::File, io::BufReader};

    use crate::types::{Instr, Opcode};

    // We use `strum` to iterate over every variant in the `Opcode` enum easily
    use strum::IntoEnumIterator;

    /// Test that opcode serialization is correct
    /// (what this test does is it converts the opcode to a string using `serde`,
    /// and checks that the corresponding substring when we index into `OPCODES`
    /// is the same)c
    #[test]
    fn test_opcode_serialization_round_trip() {
        for opcode in Opcode::iter() {
            let json: serde_json::Value = serde_json::json!(opcode);
            let deserialized_op: serde_json::Value =
                serde_json::from_value(json).expect("trouble deserializing");
            let serde_op_str = deserialized_op.as_str().unwrap();
            let op_str = opcode.as_str();
            assert_eq!(serde_op_str, op_str);
        }
    }

    /// Checks that for all opcodes, the `op_idx_to_op_str` method
    /// is implemented correctly
    #[test]
    fn test_op_idx_to_op_str() {
        for opcode in Opcode::iter() {
            let idx = opcode.get_index();
            let op_str = Opcode::op_idx_to_op_str(idx);
            assert_eq!(opcode.as_str(), op_str);
        }
    }

    /// Test that for each JSON file in the `test` directory,
    /// its flattened presentation is well-formed
    /// (i.e. for pairs of indices, the end index is always >= the start index)
    #[test]
    fn test_bril_instrs_wf() -> io::Result<()> {
        for entry in fs::read_dir("test")? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && path.extension().and_then(|ext| ext.to_str()).unwrap()
                    == "json"
            {
                let file = File::open(path).expect("Unable to open file");
                let reader = BufReader::new(file);

                let json: serde_json::Value = serde_json::from_reader(reader)
                    .expect("Unable to parse JSON");
                let functions = json["functions"]
                    .as_array()
                    .expect("Expected `functions` to be a JSON array");
                let instrs: Vec<Instr> = flatten::create_instrs(&functions[0]);
                for instr in instrs {
                    if let Some((args_start, args_end)) = instr.args {
                        assert!(
                            args_end >= args_start,
                            "{} >= {} is false",
                            args_end,
                            args_start
                        );
                    }
                    if let Some((labels_start, labels_end)) = instr.labels {
                        assert!(
                            labels_end >= labels_start,
                            "{} >= {} is false",
                            labels_end,
                            labels_start
                        );
                    }
                    if let Some((funcs_start, funcs_end)) = instr.funcs {
                        assert!(
                            funcs_end >= funcs_start,
                            "{} >= {} is false",
                            funcs_end,
                            funcs_start
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
