mod binary_reader;
mod primitives;
mod replay;
mod tracker;
mod details;
mod init;
mod attributes;

use mpq::Archive;
use self::replay::StormReplay;
use self::primitives::{ReplayResult, ReplayError, ReplayErrorKind};

pub struct StormParser {
}

impl StormParser {
    pub fn parse_file(replay_file: &String) -> ReplayResult<String> {
        match Archive::open(replay_file) {
            Ok(mut archive) => StormParser::parse_archive(&mut archive),
            Err(_) => Err(ReplayError::new(ReplayErrorKind::FileError, "failed to open archive; does the path exist? is it readable?"))
        }
    }

    pub fn parse_archive(archive: &mut Archive) -> ReplayResult<String> {
        match archive.open_file("(listfile)") {
            Ok(file) => {
                let mut buf: Vec<u8> = vec![0; file.size() as usize];
                match file.read(archive, &mut buf) {
                    Ok(_) => StormReplay::new(archive).and_then(|replay| replay.to_json()),
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed reading from replay"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to list contents of archive; possible corruption or incompatible replay?"))
        }
    }
}
