use std::io::Cursor;

use mpq::Archive;

use storm_parser::replay::StormReplay;
use storm_parser::primitives::{Point, GameMode, PlayerType, ReplayResult, ReplayError, ReplayErrorKind};
use storm_parser::binary_reader::BinaryReader;

pub struct ReplayInit {
}

impl ReplayInit {
    pub fn parse_replay_init(replay: &mut StormReplay, archive: &mut Archive) -> ReplayResult<()> {
        match archive.open_file("replay.initData") {
            Ok(file) => {
                let file_size = file.size();
                let mut file_buf: Vec<u8> = vec![0; file_size as usize];

                match file.read(archive, file_buf.as_mut()) {
                    Ok(_) => {
                        let mut cursor = Cursor::new(file_buf);
                        let mut reader = BinaryReader::new(&mut cursor);

                        let player_array_len = reader.read_vint32(5)?;
                        for _ in 0..player_array_len {
                            reader.read_len_prefixed_blob(8)?; // player name

                            if reader.read_bool()? {
                                reader.read_len_prefixed_blob(8)?; // clanTag
                            }

                            if reader.read_bool()? {
                                reader.read_len_prefixed_blob(40)?; // Clan Logo
                            }

                            if reader.read_bool()? {
                                reader.read_u8()?; // highestLeague
                            }

                            if reader.read_bool()? {
                                reader.read_u32()?; // combinedRaceLevels
                            }

                            reader.read_u32()?; // Random seed (So far, always 0 in Heroes)

                            if reader.read_bool()? {
                                reader.read_u8()?; // Race Preference
                            }

                            if reader.read_bool()? {
                                reader.read_u8()?; // Team Preference
                            }

                            reader.read_bool()?; // test map
                            reader.read_bool()?; // test auto
                            reader.read_bool()?; // examine
                            reader.read_bool()?; // custom interface

                            reader.read_u32()?; // m_testType

                            reader.read_vint32(2)?; //observer

                            reader.read_len_prefixed_blob(9)?; // m_hero - Currently Empty String
                            reader.read_len_prefixed_blob(9)?; // m_skin - Currently Empty String
                            reader.read_len_prefixed_blob(9)?; // m_mount - Currently Empty String
                            if replay.replay_version_major >= 2 {
                                reader.read_len_prefixed_blob(9)?; // m_banner - Currently Empty String
                                reader.read_len_prefixed_blob(9)?; // m_spray - Currently Empty String
                            }
                            reader.read_len_prefixed_blob(7)?; // m_toonHandle - Currently Empty String
                        }

                        replay.random_value = reader.read_u32()?;

                        reader.read_len_prefixed_blob(10)?; // m_gameCacheName - "Dflt"

                        reader.read_bool()?; // Lock Teams
                        reader.read_bool()?; // Teams Together
                        reader.read_bool()?; // Advanced Shared Control
                        reader.read_bool()?; // Random Races
                        reader.read_bool()?; // BattleNet
                        reader.read_bool()?; // AMM
                        reader.read_bool()?; // Competitive
                        reader.read_bool()?; // m_practice
                        reader.read_bool()?; // m_cooperative
                        reader.read_bool()?; // m_noVictoryOrDefeat
                        reader.read_bool()?; // m_heroDuplicatesAllowed
                        reader.read_vint32(2)?; // Fog
                        reader.read_vint32(2)?; // Observers
                        reader.read_vint32(2)?; // User Difficulty
                        reader.read_u64()?; // 64 bit int: Client Debug Flags

                        // m_ammId
                        if replay.replay_build >= 43905 && reader.read_bool()? {
                            let game_mode = reader.read_u32()?;
                            replay.game_mode = match game_mode {
                                50001 => GameMode::QuickMatch,
                                50031 => GameMode::Brawl,
                                50051 => GameMode::UnrankedDraft,
                                50061 => GameMode::HeroLeague,
                                50071 => GameMode::TeamLeague,
                                _ => GameMode::Unknown // 50021 -> AI, 50041 -> Practice
                            }
                        }

                        reader.read_vint32(3)?; // Game Speed

                        // Not sure what this 'Game Type' is
                        reader.read_vint32(3)?;

                        let max_users = reader.read_vint32(5)?;
                        if max_users != 10 {
                            replay.game_mode = GameMode::TryMe;
                        }

                        reader.read_vint32(5)?; // Max Observers
                        reader.read_vint32(5)?; // Max Players
                        reader.read_vint32(4)?; // + 1 = Max Teams
                        reader.read_vint32(6)?; // Max Colors
                        reader.read_u8()?; // + 1 = Max Races
                        reader.read_u8()?; // Max Controls

                        replay.map_size = Point { x: reader.read_vint32(8)? as i32, y: reader.read_vint32(8)? as i32 };
                        if replay.map_size.y == 1 {
                            replay.map_size.y = replay.map_size.x;
                        } else if replay.map_size.x == 0 {
                            replay.map_size.x = replay.map_size.y;
                        }

                        // Rest the structure parsing is untested before this build, per barrett777.
                        if replay.replay_build < 39595 {
                            return Ok(());
                        }

                        reader.read_u32()?; // m_mapFileSyncChecksum
                        reader.read_len_prefixed_blob(11)?; // m_mapFileName
                        reader.read_len_prefixed_blob(8)?; // m_mapAuthorName
                        reader.read_u32()?; // m_modFileSyncChecksum

                        // m_slotDescriptions
                        let slot_desc_len = reader.read_vint32(5)?;
                        for _ in 0..slot_desc_len {
                            let colors_len = reader.read_vint32(6)?;
                            reader.read_bit_array(colors_len)?; // m_allowedColors
                            let races_len = reader.read_vint32(8)?;
                            reader.read_bit_array(races_len)?; // m_allowedRaces
                            let difficulty_len = reader.read_vint32(6)?;
                            reader.read_bit_array(difficulty_len)?; // m_allowedDifficulty
                            let controls_len = reader.read_vint32(8)?;
                            reader.read_bit_array(controls_len)?; // m_allowedControls
                            let observe_types_len = reader.read_vint32(2)?;
                            reader.read_bit_array(observe_types_len)?; // m_allowedObserveTypes
                            let ai_builds_len = reader.read_vint32(7)?;
                            reader.read_bit_array(ai_builds_len)?; // m_allowedAIBuilds
                        }

                        reader.read_vint32(6)?; // m_defaultDifficulty
                        reader.read_vint32(7)?; // m_defaultAIBuild

                        // m_cacheHandles
                        let cache_handles_len = reader.read_vint32(6)?;
                        for _ in 0..cache_handles_len {
                            reader.read_bytes(40)?;
                        }

                        reader.read_bool()?; // m_hasExtensionMod
                        reader.read_bool()?; // m_isBlizzardMap
                        reader.read_bool()?; // m_isPremadeFFA
                        reader.read_bool()?; // m_isCoopMode

                        reader.read_vint32(3)?; // m_phase
                        reader.read_vint32(5)?; // m_maxUsers
                        reader.read_vint32(5)?; // m_maxObservers

                        // m_slots
                        let slots_len = reader.read_vint32(5)?;
                        for _ in 0..slots_len {
                            let mut user_id: Option<u32> = None;

                            reader.read_u8()?; // m_control
                            if reader.read_bool()? {
                                user_id = Some(reader.read_vint32(4)?); // m_userId
                            }
                            reader.read_vint32(4)?; // m_teamId
                            if reader.read_bool()? {
                                reader.read_vint32(5)?; // m_colorPref
                            }
                            if reader.read_bool()? {
                                reader.read_u8()?; // m_racePref
                            }
                            reader.read_vint32(6)?; // m_difficulty
                            reader.read_vint32(7)?; // m_aiBuild
                            reader.read_vint32(7)?; // m_handicap

                            // m_observe
                            let observer_status = reader.read_vint32(2)?;

                            reader.read_u32()?; // m_logoIndex

                            reader.read_len_prefixed_blob(9)?; // m_hero

                            let skin_skin_tint = match reader.read_len_prefixed_string(9) { // m_skin
                                Ok(result) => match result.as_ref() {
                                    "" => None,
                                    _ => Some(result.clone())
                                },
                                Err(_) => None
                            };

                            let mount_mount_tint = match reader.read_len_prefixed_string(9) { // m_mount
                                Ok(result) => match result.as_ref() {
                                    "" => None,
                                    _ => Some(result.clone())
                                },
                                Err(_) => None
                            };

                            // m_artifacts
                            let artifacts_len = reader.read_vint32(4)?;
                            for _ in 0..artifacts_len {
                                reader.read_len_prefixed_blob(9)?;
                            }

                            let mut working_set_slot_id: Option<u32> = None;
                            if reader.read_bool()? {
                                working_set_slot_id = Some(reader.read_vint32(8)?); // m_workingSetSlotId
                            }

                            if user_id.is_some() && working_set_slot_id.is_some() {
                                let actual_slot_id = working_set_slot_id.unwrap();
                                let actual_user_id = user_id.unwrap();

                                let mut player = replay.get_player_by_user_id_or_slot_id(actual_user_id, actual_slot_id).unwrap();

                                // Just re-up the user and slot IDs while we're here.
                                player.user_id = actual_user_id;
                                player.slot_id = actual_slot_id;

                                if observer_status == 2 {
                                    player.player_type = PlayerType::Spectator;
                                }

                                player.skin = skin_skin_tint;
                                player.mount = mount_mount_tint;
                            }

                            // m_rewards
                            let rewards_len = reader.read_vint32(17)?;
                            for _ in 0..rewards_len {
                                reader.read_u32()?;
                            }

                            reader.read_len_prefixed_blob(7)?; // m_toonHandle

                            // m_licenses
                            if replay.replay_build < 49582 || replay.replay_build == 49838 {
                                let licenses_len = reader.read_vint32(9)?;
                                for _ in 0..licenses_len {
                                    reader.read_u32()?;
                                }
                            }

                            if reader.read_bool()? {
                                reader.read_vint32(4)?; // m_tandemLeaderUserId
                            }

                            if replay.replay_build <= 41504 {
                                reader.read_len_prefixed_blob(9)?; // m_commander - Empty string
                                reader.read_u32()?; // m_commanderLevel - So far, always 0
                            }

                            if reader.read_bool()? && user_id.is_some() { // m_hasSilencePenalty
                                let actual_user_id = user_id.unwrap();
                                let mut player = replay.get_player_by_user_id_or_slot_id(actual_user_id, 0).unwrap();
                                player.is_silenced = true;
                            }

                            if replay.replay_version_major >= 2 {
                                reader.read_len_prefixed_blob(9)?; // m_banner
                                reader.read_len_prefixed_blob(9)?; // m_spray
                                reader.read_len_prefixed_blob(9)?; // m_announcerPack
                                reader.read_len_prefixed_blob(9)?; // m_voiceLine

                                // m_heroMasteryTiers
                                if replay.replay_build >= 52561 {
                                    let hero_mastery_tiers_len = reader.read_vint32(10)?;
                                    for _ in 0..hero_mastery_tiers_len {
                                        reader.read_u32()?; // m_hero
                                        reader.read_u8()?; // m_tier
                                    }
                                }
                            }
                        }

                        let random_value_second = reader.read_u32()?;
                        if random_value_second != replay.random_value { // m_randomSeed
                            return Err(ReplayError::new(ReplayErrorKind::IntegrityError, "replay random seeds did not match"));
                        }

                        if reader.read_bool()? {
                            reader.read_vint32(4)?; // m_hostUserId
                        }

                        reader.read_bool()?; // m_isSinglePlayer

                        reader.read_u8()?; // m_pickedMapTag - So far, always 0

                        reader.read_u32()?; // m_gameDuration - So far, always 0

                        reader.read_vint32(6)?; // m_defaultDifficulty

                        reader.read_vint32(7)?; // m_defaultAIBuild

                        Ok(())
                    },
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to read init file"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to open init file"))
        }
    }
}
