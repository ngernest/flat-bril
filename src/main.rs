use std::path::Path;

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

// To create an fbril file:
// `bril2json < test/call.bril | cargo run -- --filename test/call.fbril --fbril`

// To interpret a file: `cargo run -- --filename test/call.fbril --interp`

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
        .arg(
            Arg::new("fbril")
                .long("fbril")
                .action(ArgAction::SetTrue)
                .value_name("VALUE")
                .requires("filename")
                .help("Output filename"),
        )
        .arg(
            Arg::new("filename")
                .long("filename")
                .help("File to open")
                .value_name("FILE_TO_OPEN"),
        )
        .get_matches();

    if matches.get_flag("json") {
        // Check that JSON -> flat -> JSON round trip works
        json_roundtrip::json_roundtrip();
    } else if matches.get_flag("fbril") {
        // Convert the JSON Bril program to a flat Bril program
        match matches.get_one::<String>("filename") {
            Some(filename) => {
                println!("Processing {}", filename);
                memfile::json_to_fbril(filename.clone());
            }
            None => {
                eprintln!("Error: --fbril requires a filename argument");
                std::process::exit(1);
            }
        }
    } else if let Some(possible_arg_values) =
        matches.get_many::<String>("interp")
    {
        let filename = matches
            .get_one::<String>("filename")
            .expect("missing filename");

        let arg_values: Vec<&str> =
            possible_arg_values.map(|s| s.as_str()).collect();
        // Actually interpret a flat Bril file
        if !Path::new(&filename).exists() {
            panic!("tried to open a non-existent file");
        }

        let new_mmap =
            memfile::mmap_new_file(filename.as_str(), 100000000, false);
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
        interp_program(program, arg_values);
    }
}
