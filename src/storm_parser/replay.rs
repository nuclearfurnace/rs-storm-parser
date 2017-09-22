use std::io::Cursor;
use mpq::Archive;
use serde_json;
use storm_parser::tracker::TrackerEvent;

#[derive(Serialize, Deserialize)]
enum GameSpeed {
    Unknown,
    Slower,
    Slow,
    Normal,
    Fast,
    Faster,
}

impl GameSpeed {
    fn from_i32(v: i32) -> GameSpeed {
        match v {
            0 => GameSpeed::Unknown,
            1 => GameSpeed::Slower,
            2 => GameSpeed::Slow,
            3 => GameSpeed::Normal,
            4 => GameSpeed::Fast,
            5 => GameSpeed::Faster,
            _ => GameSpeed::Unknown,
        }
    }
}

impl Default for GameSpeed {
    fn default() -> GameSpeed { GameSpeed::Unknown }
}

#[derive(Serialize, Deserialize)]
enum GameMode {
    Unknown,
    Event,
    Custom,
    TryMe,
    Practice,
    Cooperative,
    QuickMatch,
    HeroLeague,
    TeamLeague,
    UnrankedDraft,
    Brawl,
}

impl GameMode {
    fn from_i32(v: i32) -> GameMode {
        match v {
            -9 => GameMode::Unknown,
            -2 => GameMode::Event,
            -1 => GameMode::Custom,
            0 => GameMode::TryMe,
            1 => GameMode::Practice,
            2 => GameMode::Cooperative,
            3 => GameMode::QuickMatch,
            4 => GameMode::HeroLeague,
            5 => GameMode::TeamLeague,
            6 => GameMode::UnrankedDraft,
            7 => GameMode::Brawl,
            _ => GameMode::Unknown,
        }
    }
}

impl Default for GameMode {
    fn default() -> GameMode { GameMode::Unknown }
}

#[derive(Serialize, Deserialize, Default)]
pub struct StormReplay {
    // High-level attributes of the replay.
    replay_build: i32,
    replay_version_major: i32,
    replay_version: String,

    frames: i32,
    game_speed: GameSpeed,
    game_mode: GameMode,
    map: String,
}

impl StormReplay {
    pub fn new(archive: &mut Archive) -> Result<StormReplay, String> {
        let mut replay: StormReplay = Default::default();

        StormReplay::parse_basic_details(&mut replay, archive)?;

        Ok(replay)
    }

    fn parse_basic_details(replay: &mut StormReplay, archive: &mut Archive) -> Result<(), String> {
        match archive.read_user_data() {
            Ok(result) => match result {
                Some(data) => {
                    let mut user_data_cursor = Cursor::new(data);
                    match TrackerEvent::new(&mut user_data_cursor) {
                        Ok(mut event) => {
                            println!("event: {:?}", event);

                            // mut TrackerEvent -> Option<HashMap<i32, TrackerEvent>> -> HashMap<i32, TrackerEvent>
                            let event_dict = event.dictionary.unwrap();
                            // HashMap<i32, TrackerEvent> -> Option<&TrackerEvent> -> &TrackerEvent
                            let unwrapped_dict = event_dict.get(&1).unwrap();
                            // &TrackerEvent -> Option<HashMap<i32, TrackerEvent>> -> HashMap<i32, TrackerEvent>
                            let version_dict = unwrapped_dict.dictionary.unwrap();
                            let version_string = format!("{}.{}.{}.{}",
                                version_dict.get(&0).unwrap().variable_int.unwrap(),
                                version_dict.get(&1).unwrap().variable_int.unwrap(),
                                version_dict.get(&2).unwrap().variable_int.unwrap(),
                                version_dict.get(&3).unwrap().variable_int.unwrap());

                            replay.replay_version = version_string;
                            Ok(())
                        },
                        _ => Err("failed to parse basic replay details".to_owned())
                    }
                },
                _ => Err("no replay detail data found in replay; is this a corrupted replay or another game?".to_owned())
            },
            _ => Err("failed to read replay detail; is something wrong with the file permissions? *shrug*".to_owned())
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(s) => Ok(s),
            Err(_) => Err("failed to convert replay structure to JSON".to_owned())
        }
    }
}
