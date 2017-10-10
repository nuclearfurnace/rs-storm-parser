use chrono::Duration;
use mpq::Archive;

use storm_parser::replay::StormReplay;
use storm_parser::binary_reader::BinaryReader;
use storm_parser::tracker::TrackerEvent;
use storm_parser::primitives::*;

pub struct GameEvent {
    pub event_type: ReplayGameEventType,
    pub ticks_elapsed: Duration,
    pub player: Player,
    pub is_global: bool,
    pub data: TrackerEvent,
}

pub struct ReplayGameEvents {
}

impl ReplayGameEvents {
    pub fn parse_replay_game_events(replay: &mut StormReplay, archive: &mut Archive) -> ReplayResult<()> {
         match archive.open_file("replay.game.events") {
            Ok(file) => {
                let file_size = file.size();
                let mut file_buf: Vec<u8> = vec![0; file_size as usize];

                match file.read(archive, file_buf.as_mut()) {
                    Ok(_) => {
                        let mut reader = BinaryReader::new(&file_buf);
                        Ok(())
                    },
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError,  "failed to read game events file"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to open game events file"))
        }
    }
}
