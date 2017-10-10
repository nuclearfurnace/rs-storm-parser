use std::error::Error;
use std::fmt;
use std::io;

use backtrace::Backtrace;

#[derive(Primitive, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum GameSpeed {
    Unknown = 0,
    Slower = 1,
    Slow = 2,
    Normal = 3,
    Fast = 4,
    Faster = 5,
}

impl Default for GameSpeed {
    fn default() -> GameSpeed { GameSpeed::Unknown }
}

impl GameSpeed {
    pub fn from_str(s: &str) -> GameSpeed {
        match s.to_lowercase().as_ref() {
            "slor" => GameSpeed::Slower,
            "slow" => GameSpeed::Slow,
            "norm" => GameSpeed::Normal,
            "fast" => GameSpeed::Fast,
            "fasr" => GameSpeed::Faster,
            _ => panic!("unknown game speed: {}", s)
        }
    }
}

#[derive(SignedPrimitive, Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub enum GameMode {
    Unknown = -9,
    Event = -2,
    Custom = -1,
    TryMe = 0,
    Practice = 1,
    Cooperative = 2,
    QuickMatch = 3,
    HeroLeague = 4,
    TeamLeague = 5,
    UnrankedDraft = 6,
    Brawl = 7,
}

impl Default for GameMode {
    fn default() -> GameMode { GameMode::Unknown }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum PlayerType {
    Human,
    Computer,
    Spectator
}

impl Default for PlayerType {
    fn default() -> PlayerType { PlayerType::Computer }
}

#[derive(Primitive, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum ReplayAttributeEventType
{
    Unknown = 0,

    PlayerTypeAttribute = 500,

    /* 2000 - 2024 are related to team sizes */
    TeamSizeAttribute = 2001,

    GameSpeedAttribute = 3000,
    DifficultyLevelAttribute = 3004,
    GameTypeAttribute = 3009,
    /* 3100 - 3300 are related to AI builds (for Starcraft 2) */

    Hero = 4002,
    SkinAndSkinTint = 4003,
    CharacterLevel = 4008,
    LobbyMode = 4010,
    ReadyMode = 4018,

    DraftTeam1BanChooserSlot = 4022,
    DraftTeam1Ban1 = 4023,
    DraftTeam1Ban2 = 4025,

    DraftTeam2BanChooserSlot = 4027,
    DraftTeam2Ban1 = 4028,
    DraftTeam2Ban2 = 4030,

    /* 4100 - 4200 are related to Artifacts, no longer in the game */
}

#[derive(Primitive, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum ReplayGameEventType
{
    StartGameEvent = 2,
    DropOurselvesEvent = 3,
    UserFinishedLoadingSyncEvent = 5,
    UserOptionsEvent = 7,
    BankFileEvent = 9,
    BankSectionEvent = 10,
    BankKeyEvent = 11,
    BankSignatureEvent = 13,
    CameraSaveEvent = 14,
    CommandManagerResetEvent = 25,
    GameCheatEvent = 26,
    CmdEvent = 27,
    SelectionDeltaEvent = 28,
    ControlGroupUpdateEvent = 29,
    SelectionSyncCheckEvent = 30,
    ResourceTradeEvent = 31,
    TriggerChatMessageEvent = 32,
    SetAbsoluteGameSpeedEvent = 34,
    TriggerPingEvent = 36,
    UnitClickEvent = 39,
    TriggerSkippedEvent = 44,
    TriggerSoundLengthQueryEvent = 45,
    TriggerSoundOffsetEvent = 46,
    TriggerTransmissionOffsetEvent = 47,
    TriggerTransmissionCompleteEvent = 48,
    CameraUpdateEvent = 49,
    TriggerPlanetMissionLaunchedEvent = 53,
    TriggerDialogControlEvent = 55,
    TriggerSoundLengthSyncEvent = 56,
    TriggerConversationSkippedEvent = 57,
    TriggerMouseClickedEvent = 58,
    TriggerMouseMovedEvent = 59,
    TriggerHotkeyPressedEvent = 61,
    TriggerTargetModeUpdateEvent = 62,
    TriggerSoundtrackDoneEvent = 64,
    TriggerKeyPressedEvent = 66,
    TriggerCutsceneBookmarkFiredEvent = 97,
    TriggerCutsceneEndSceneFiredEvent = 98,
    GameUserLeaveEvent = 101,
    GameUserJoinEvent = 102,
    CommandManagerStateEvent = 103,
    CmdUpdateTargetPointEvent = 104,
    CmdUpdateTargetUnitEvent = 105,
    HeroTalentSelectedEvent = 110,
    HeroTalentTreeSelectionPanelToggled = 112
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Difficulty {
    Beginner,
    Recruit,
    Adept,
    Veteran,
    Elite,
}

impl Default for Difficulty {
    fn default() -> Difficulty { Difficulty::Beginner }
}

impl Difficulty {
    pub fn from_str(s: &str) -> Difficulty {
        match s.to_lowercase().as_ref() {
            "vyey" => Difficulty::Beginner,
            "easy" => Difficulty::Recruit,
            "medi" => Difficulty::Adept,
            "hdvh" => Difficulty::Veteran,
            "vyhd" => Difficulty::Elite,
            _ => panic!("unknown difficulty type: {}", s)
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum TeamSize {
    OneVsOne,
    TwoVsTwo,
    ThreeVsThree,
    FourVsFour,
    FiveVsFive,
    FFA,
}

impl Default for TeamSize {
    fn default() -> TeamSize { TeamSize::FFA }
}

impl TeamSize {
    pub fn from_str(s: &str) -> TeamSize {
        match s.to_lowercase().as_ref() {
            "1v1" => TeamSize::OneVsOne,
            "2v2" => TeamSize::TwoVsTwo,
            "3v3" => TeamSize::ThreeVsThree,
            "4v4" => TeamSize::FourVsFour,
            "5v5" => TeamSize::FiveVsFive,
            "ffa" => TeamSize::FFA,
            _ => panic!("unknown team size: {}", s)
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DraftBans {
    pub team_one_first_ban: String,
    pub team_one_second_ban: String,
    pub team_two_first_ban: String,
    pub team_two_second_ban: String,
}

#[derive(Serialize, Deserialize, Copy, Clone, Default)]
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
    pub index: u32,
    // 4-bytes long, ARGB
    pub color: Vec<u32>,
    pub team: u32,
    pub handicap: i32,
    pub is_winner: bool,
    pub is_silenced: bool,
    pub character: String,
    pub character_level: u32,
    pub skin: Option<String>,
    pub mount: Option<String>,
    pub difficulty: Difficulty,
    pub is_auto_select: bool,
}
