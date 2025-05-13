use clap::{Arg, ArgAction, Command};
use interp::interp_program;
use types::Header;
use zerocopy::FromBytes;
mod flatten;
mod interp;
mod json_roundtrip;
mod memfile;
mod types;
mod unflatten;

fn main() {
    // Enable stack backtrace for debugging
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let matches = Command::new("flat-bril")
        .arg(
            Arg::new("interp")
                .long("interp")
                .action(ArgAction::Append)
                .num_args(0..)
                .value_name("VALUES"),
        )
        .arg(Arg::new("json").long("json").action(ArgAction::SetTrue))
        .arg(Arg::new("fbril").long("fbril").action(ArgAction::SetTrue))
        .get_matches();

    if matches.get_flag("json") {
        // Check that JSON -> flat -> JSON round trip works
        json_roundtrip::json_roundtrip();
    } else if matches.get_flag("fbril") {
        // Convert the JSON Bril program to a flat BRil program
        memfile::json_to_fbril();
    } else if let Some(possible_arg_values) =
        matches.get_many::<String>("interp")
    {
        // Actually interpret a flat Bril file
        let int_arg_values: Result<Vec<i64>, _> =
            possible_arg_values.map(|s| s.parse::<i64>()).collect();
        match int_arg_values {
            Ok(cmd_line_args) => {
                // TODO: figure out the right filename to open
                // ^^ (make this a cmd-line arg)
                let new_mmap =
                    memfile::mmap_new_file("call.fbril", 100000000, false);
                let (new_header, remaining_mmap) =
                    Header::ref_from_prefix(&new_mmap).unwrap();

                let mut offset = 0;

                let mut program_vec = vec![];
                for size in new_header.sizes {
                    if size != 0 {
                        let size = size as usize;
                        let instr_view = memfile::get_instr_view(
                            &remaining_mmap[offset..offset + size],
                        );
                        program_vec.push(instr_view);
                        offset += size;
                    }
                }
                let program = program_vec.as_slice();
                println!("done deserializing, interpreting program:");
                interp_program(program, cmd_line_args);
            }
            Err(_) => panic!("all arguments to main must be integer literals"),
        }
    }
}
