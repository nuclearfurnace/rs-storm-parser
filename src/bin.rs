extern crate storm_parser;
extern crate clap;

use clap::{Arg, App};

use storm_parser::StormParser;

fn main() {
    let matches = App::new("storm-parser")
        .arg(Arg::with_name("validate")
             .long("validate")
             .help("whether or not to run purely in validation mode")
             .required(false))
        .arg(Arg::with_name("INPUT")
             .help("sets the input replay file to parse")
             .required(true))
        .get_matches();

    let replay_file = matches.value_of("INPUT").unwrap().to_string();
    if matches.is_present("validate") {
        ::std::process::exit(match StormParser::validate_replay(&replay_file) {
            Ok(result) => {
                println!("{}", result);
                0
            },
            Err(_) => 1
        });
    } else {
        ::std::process::exit(match StormParser::parse_replay(&replay_file) {
            Ok(result) => {
                println!("{}", result);
                0
            },
            Err(e) => {
                println!("{:?}", e);
                1
            }
        });
    }
}
