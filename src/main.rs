use clap::{Arg, ArgAction, Command};
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
            Ok(_cmd_line_args) => {
                // TODO: figure out how to populate the env with the
                // values of the args supplied to `main`
                todo!()
            }
            Err(_) => panic!("all arguments to main must be integer literals"),
        }
    }
}
