use std::io::Cursor;
use std::fmt::Write;

use chrono::prelude::*;
use mpq::Archive;
use serde_json;

use storm_parser::tracker::TrackerEvent;
use storm_parser::details::ReplayDetails;
use storm_parser::init::ReplayInit;
use storm_parser::primitives::{Point, GameSpeed, GameMode, PlayerType, Player, ReplayResult, ReplayError, ReplayErrorKind};

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Default)]
pub struct StormReplay {
    // High-level attributes of the self.
    pub replay_build: u32,
    pub replay_version_major: u32,
    pub replay_version: String,
    pub random_value: u32,

    pub frames: u32,
    pub game_length_sec: u32,
    pub game_speed: GameSpeed,
    pub game_mode: GameMode,
    pub map: String,
    #[derivative(Default(value="Utc::now()"))]
    pub timestamp: DateTime<Utc>,
    pub map_size: Point,

    pub players: Vec<Player>,
    pub players_by_user_id: [Player; 16],
    pub players_by_slot_id: [Player; 16],
}

impl StormReplay {
    pub fn new(archive: &mut Archive) -> ReplayResult<StormReplay> {
        let mut replay: StormReplay = Default::default();

        replay.parse_replay_metadata(archive)?;
        replay.parse_replay_details(archive)?;
        replay.parse_replay_init(archive)?;

        Ok(replay)
    }

    fn parse_replay_metadata(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        match archive.read_user_data() {
            Ok(result) => match result {
                Some(data) => {
                    let mut user_data_cursor = Cursor::new(data);
                    match TrackerEvent::new(&mut user_data_cursor) {
                        Ok(mut event) => {
                            let version_string = format!("{}.{}.{}.{}",
                                event.get_dict_entry(1).get_dict_entry(0).get_vint(),
                                event.get_dict_entry(1).get_dict_entry(1).get_vint(),
                                event.get_dict_entry(1).get_dict_entry(2).get_vint(),
                                event.get_dict_entry(1).get_dict_entry(3).get_vint());

                            self.replay_version = version_string;
                            self.replay_build = event.get_dict_entry(1).get_dict_entry(4).get_vint() as u32;

                            if self.replay_build >= 51978 {
                                self.replay_version_major = event.get_dict_entry(1).get_dict_entry(1).get_vint() as u32;
                            } else {
                                self.replay_version_major = 1;
                            }

                            if self.replay_build >= 39951 {
                                // As noted by barrett777, this build number seems to be a more accurate build number,
                                // and was noticed as changing after build 39951.
                                self.replay_build = event.get_dict_entry(6).get_vint() as u32;
                            }

                            // The SC2/HoTS game engine runs at 16 frames per second, so it tracks the match length in
                            // frames, and we do some simple math here fto get the real time.
                            self.frames = event.get_dict_entry(3).get_vint() as u32;
                            self.game_length_sec = self.frames / 16;

                            Ok(())
                        },
                        _ => Err(ReplayError::new(ReplayErrorKind::StructureError, "failed to parse basic replay details"))
                    }
                },
                _ => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "no replay metadata found in replay; is this a corrupted replay or another game?"))
            },
            _ => Err(ReplayError::new(ReplayErrorKind::FileError, "failed to read replay detail; is something wrong with the file permissions? *shrug*"))
        }
    }

    fn parse_replay_details(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        ReplayDetails::parse_replay_details(self, archive)
    }

    fn parse_replay_init(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        ReplayInit::parse_replay_init(self, archive)
    }

    pub fn get_player_by_user_id_or_slot_id(&mut self, user_id: u32, slot_id: u32) -> Option<&mut Player> {
        self.players.iter_mut()
            .find(|ref p| p.user_id == user_id || p.slot_id == slot_id)
    }

    pub fn to_json(&self) -> ReplayResult<String> {
        match serde_json::to_string(self) {
            Ok(s) => Ok(s),
            Err(_) => Err(ReplayError::new(ReplayErrorKind::OutputError, "failed to convert replay structure to JSON"))
        }
    }
}
