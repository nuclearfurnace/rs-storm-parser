mod replay;
mod tracker;

use mpq::Archive;
use self::replay::StormReplay;

pub struct StormParser {
}

impl StormParser {
    pub fn parse_file(replay_file: &String) -> Result<String, String> {
        match Archive::open(replay_file) {
            Ok(mut archive) => StormParser::parse_archive(&mut archive),
            Err(_) => Err("failed to open archive; does the path exist? is it readable?".to_owned())
        }
    }

    pub fn parse_archive(archive: &mut Archive) -> Result<String, String> {
        match archive.open_file("(listfile)") {
            Ok(file) => {
                let mut buf: Vec<u8> = vec![0; file.size() as usize];
                match file.read(archive, &mut buf) {
                    // If this read was good, we should have a real replay, so proceed.
                    Ok(_) => StormParser::parse_replay(archive),
                    // No listfiles?  That's no bueno.
                    Err(_) => Err("failed reading from replay".to_owned())
                }
            },
            Err(_) => Err("failed to list contents of archive; possible corruption or incompatible replay?".to_owned())
        }
    }

    pub fn parse_replay(archive: &mut Archive) -> Result<String, String> {
        StormReplay::new(archive).and_then(|replay| replay.to_json())
    }
}
