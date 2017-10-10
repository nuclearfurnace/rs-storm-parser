extern crate storm_parser;

use std::env;
use storm_parser::StormParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    let replay_file = &args[1];
    ::std::process::exit(match StormParser::parse_file(replay_file) {
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
