extern crate storm_parser;

use std::env;
use std::str;
use storm_parser::StormParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    let replay_file = &args[1];
    println!("Reading from file {}...", replay_file);

    match StormParser::parse_file(replay_file) {
        Ok(result) => {
            println!("File processed.  Result: ");
            println!("{}", result);
        },
        Err(e) => println!("Error encountered: {}", e)
    };

    println!("Done.");
}
