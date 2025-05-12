use clap::{Arg, ArgAction, Command};
use std::io::{self, Read};

mod flatten;
mod memfile;
mod types;
mod unflatten;
mod interp;

fn main() {
    // Enable stack backtrace for debugging
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let matches = Command::new("flat-bril")
        .arg(
            Arg::new("memfile")
                .long("memfile")
                .action(ArgAction::Append)
                .num_args(0..)
                .value_name("VALUES"),
        )
        .arg(
            Arg::new("roundtrip").long("roundtrip").action(ArgAction::SetTrue)
        )
        .get_matches();

    // Example: `bril2json < test/nop.bril | cargo run -- --memfile <INT_VALUES>`
    if let Some(possible_arg_values) = matches.get_many::<String>("memfile") {
        let int_arg_values: Result<Vec<i64>, _> = 
          possible_arg_values.map(|s| s.parse::<i64>()).collect();
        match int_arg_values {
            Ok(_cmd_line_args) => {
                // TODO: figure out how to populate the env with the 
                // values of the args supplied to `main`
                // Call the main function from memfile.rs
                memfile::main();
            },
            Err(_) => panic!("all arguments to main must be integer literals")
        }

        
    } else {
        // Read in the JSON representation of a Bril file from stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Unable to read from stdin");

        // Parse the JSON into serde_json's `Value` datatype
        let json: serde_json::Value = serde_json::from_str(&buffer)
            .expect("Unable to parse malformed JSON");
        let functions = json["functions"]
            .as_array()
            .expect("Expected `functions` to be a JSON array");
        let mut func_json_vec = vec![];
        for func in functions {
            let instr_store = flatten::flatten_instrs(func);
            let func_json = unflatten::unflatten_instrs(&instr_store);
            func_json_vec.push(func_json);
        }
        let prog_json = serde_json::json!({
            "functions": func_json_vec
        });
        println!("{prog_json}");
    }
}
