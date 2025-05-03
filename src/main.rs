use std::io::{self, Read};

mod flatten;
mod types;

fn main() {
    // Enable stack backtrace for debugging
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Read in the JSON representation of a Bril file from stdin
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");

    // Parse the JSON into serde_json's `Value` datatype
    let json: serde_json::Value =
        serde_json::from_str(&buffer).expect("Unable to parse malformed JSON");
    let functions = json["functions"]
        .as_array()
        .expect("Expected `functions` to be a JSON array");
    for func in functions {
        let _instrs = flatten::create_instrs(func.clone());
        // TODO: figure out what to do with _instrs
    }
}

/* -------------------------------------------------------------------------- */
/*                                    Tests                                   */
/* -------------------------------------------------------------------------- */
#[cfg(test)]
mod tests {
    use std::{fs, fs::File, io::BufReader};

    use crate::types::{Instr, Opcode, OPCODE_BUFFER, OPCODE_IDX};

    use super::*;

    // We use `strum` to iterate over every variant in the `Opcode` enum easily
    use strum::IntoEnumIterator;

    /// Test that opcode serialization is correct
    /// (what this test does is it converts the opcode to a string using `serde`,
    /// and checks that the corresponding substring when we index into `OPCODES`
    /// is the same)
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

    /// Checks that for all opcodes, their start/end indexes in `OPCODE_IDX` are correct
    #[test]
    fn test_opcode_indexes_correct() {
        for opcode in Opcode::iter() {
            let idx = opcode.get_index();
            let (start_idx, end_idx) = OPCODE_IDX[idx];
            let op_str = &OPCODE_BUFFER[start_idx..=end_idx];
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
                let instrs: Vec<Instr> =
                    flatten::create_instrs(functions[0].clone());
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
