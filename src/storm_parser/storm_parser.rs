extern crate mpq;

use std::str;
use mpq::Archive;

struct StormParser {
}

impl StormParser {
    fn parse_file(replay_file: &String) -> Result<String, String> {
        let mut archive = Archive::open(replay_file).unwrap_or(Err("failed to open archive; does the path exist? is it readable?"));
        let file = archive.open_file("(listfile)").unwrap_or(Err("failed to read archive file list; is this a real replay?"));

        let mut buf: Vec<u8> = vec![0; file.size() as usize];
        file.read(&mut archive, &mut buf).unwrap(Err("failed reading from replay"));

        match str::from_utf8(&buf) {
            Ok(s) => Ok(s.to_owned()),
            Err(e) => Err("unknown contents when listing archive files".to_owned())
        }
    }
}
