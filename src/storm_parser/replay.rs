use std::fmt::Write;

use md5;
use chrono::prelude::*;
use mpq::Archive;
use serde_json;
use uuid::Uuid;

use storm_parser::binary_reader::BinaryReader;
use storm_parser::tracker::{TrackerEventStructure, TrackerEvent, ReplayTrackerEvents};
use storm_parser::details::ReplayDetails;
use storm_parser::init::ReplayInit;
use storm_parser::attributes::ReplayAttributes;
use storm_parser::events::{GameEvent, ReplayGameEvents};
use storm_parser::primitives::*;

#[derive(Derivative, Serialize)]
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
    pub team_size: TeamSize,
    pub bans: DraftBans,

    pub game_events: Vec<GameEvent>,
    pub tracker_events: Vec<TrackerEvent>,
}

impl StormReplay {
    pub fn parse(archive: &mut Archive) -> ReplayResult<StormReplay> {
        let mut replay: StormReplay = Default::default();

        replay.parse_replay_metadata(archive)?;
        replay.parse_replay_details(archive)?;
        replay.parse_replay_init(archive)?;
        replay.parse_replay_attributes(archive)?;
        replay.parse_replay_game_events(archive)?;
        replay.parse_replay_tracker_events(archive)?;

        Ok(replay)
    }

    pub fn validate(archive: &mut Archive) -> ReplayResult<String> {
        let mut replay: StormReplay = Default::default();

        replay.parse_replay_metadata(archive)?;
        replay.parse_replay_details(archive)?;
        replay.parse_replay_init(archive)?;

        // We'll build a UUID from sha128(sorted player names + replay version + random value)
        let mut signature = String::new();
        let mut player_names: Vec<String> = replay.players.iter().map(|p| p.name.clone()).collect();
        player_names.sort();
        for name in &player_names {
            write!(&mut signature, "{}", name);
        }
        write!(&mut signature, "{}", replay.replay_version);
        write!(&mut signature, "{}", replay.random_value);

        let hash = md5::compute(signature.as_bytes());
        Uuid::from_bytes(&hash[0..16])
            .map(|uuid| uuid.hyphenated().to_string())
            .map_err(|_| ReplayError::new(ReplayErrorKind::Other, "failed to generate signature for replay"))
    }

    fn parse_replay_metadata(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        match archive.read_user_data() {
            Ok(result) => match result {
                Some(data) => {
                    let mut reader = BinaryReader::new(&data);
                    match TrackerEventStructure::new(&mut reader) {
                        Ok(event) => {
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

    fn parse_replay_attributes(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        ReplayAttributes::parse_replay_attributes(self, archive)
    }

    fn parse_replay_game_events(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        ReplayGameEvents::parse_replay_game_events(self, archive)
    }

    fn parse_replay_tracker_events(&mut self, archive: &mut Archive) -> ReplayResult<()> {
        ReplayTrackerEvents::parse_replay_tracker_events(self, archive)
    }

    pub fn get_player_by_index(&mut self, index: u32) -> Option<&mut Player> {
        self.players.iter_mut().find(|ref p| p.index == index)
    }

    pub fn to_json(&self) -> ReplayResult<String> {
        match serde_json::to_string(self) {
            Ok(s) => Ok(s),
            Err(_) => Err(ReplayError::new(ReplayErrorKind::OutputError, "failed to convert replay structure to JSON"))
        }
    }
}
