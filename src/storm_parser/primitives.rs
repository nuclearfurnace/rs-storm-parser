use std::error::Error;
use std::fmt;
use std::io;

use backtrace::Backtrace;
use serde_json;

#[derive(Serialize, Deserialize)]
pub enum GameSpeed {
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
pub enum GameMode {
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

#[derive(Serialize, Deserialize)]
pub enum PlayerType {
    Human,
    Computer,
    Spectator
}

impl Default for PlayerType {
    fn default() -> PlayerType { PlayerType::Human }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

pub type ReplayResult<T> = Result<T, ReplayError>;

#[derive(Debug, Clone)]
pub struct ReplayError {
    pub kind: ReplayErrorKind,
    pub msg: String,
    pub backtrace: Backtrace
}

#[derive(Debug, Clone)]
pub enum ReplayErrorKind {
    FileError,
    ArchiveError,
    IntegrityError,
    ReaderError,
    StructureError,
    OutputError
}

impl ReplayError {
    pub fn new(kind: ReplayErrorKind, msg: &str) -> ReplayError {
        ReplayError { kind: kind, msg: msg.to_string(), backtrace: Backtrace::new() }
    }
}

impl Error for ReplayError {
    fn description(&self) -> &str {
        match self.kind {
            ReplayErrorKind::FileError => "file error",
            ReplayErrorKind::ArchiveError => "integrity error",
            ReplayErrorKind::IntegrityError => "integrity error",
            ReplayErrorKind::ReaderError => "error while reading replay",
            ReplayErrorKind::StructureError => "structure error",
            ReplayErrorKind::OutputError => "output error"
        }
    }
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}\n{:?}", self.description(), self.msg, self.backtrace)
    }
}

impl From<io::Error> for ReplayError {
    fn from(error: io::Error) -> Self {
        ReplayError::new(ReplayErrorKind::ReaderError, error.description())
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Player {
    pub name: String,
    pub player_type: PlayerType,
    pub battlenet_region_id: u32,
    pub battlenet_sub_id: u32,
    pub battlenet_id: u32,
    pub user_id: u32,
    pub slot_id: u32,
    // 4-bytes long, ARGB
    pub color: Vec<u32>,
    pub team: u32,
    pub handicap: i32,
    pub is_winner: bool,
    pub is_silenced: bool,
    pub character: String,
    pub skin: Option<String>,
    pub mount: Option<String>,
}
