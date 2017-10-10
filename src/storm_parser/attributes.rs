use std::ffi::CStr;

use mpq::Archive;
use num_traits::{FromPrimitive, ToPrimitive};
use unicode_reverse::reverse_grapheme_clusters_in_place;

use storm_parser::replay::StormReplay;
use storm_parser::binary_reader::BinaryReader;
use storm_parser::primitives::*;

const EMPTY_STR: [u8; 4] = [0, 0, 0, 0];

pub struct ReplayAttribute {
    header: u32,
    attribute_type: Option<ReplayAttributeEventType>,
    player_id: u32,
    value: [u8; 4]
}

impl ReplayAttribute {
    pub fn get_value_str(&self) -> Option<String> {
        if self.value == EMPTY_STR {
            return None
        }

        let mut buf = self.value.to_vec();
        if buf[0] == 0u8 {
            buf.reverse();
        }

        let parsed = match buf[3] {
            0u8 => match CStr::from_bytes_with_nul(&buf) {
                Ok(cstr) => {
                    match cstr.to_str() {
                        Ok(cstr_utf8) => Some(cstr_utf8.to_string()),
                        Err(_) => None
                    }
                },
                Err(e) => panic!("error reading value as C string: {}", e)
            },
            _ => String::from_utf8(buf).ok()
        };

        parsed.map(|mut x| { reverse_grapheme_clusters_in_place(&mut x); x })
    }

    pub fn get_value_int(&self) -> Option<u32> {
        match self.get_value_str() {
            Some(s) => {
                s.trim().parse::<u32>().ok()
            },
            None => panic!("could not parse value int")
        }
    }
}

pub struct ReplayAttributes {
}

impl ReplayAttributes {
    pub fn parse_replay_attributes(replay: &mut StormReplay, archive: &mut Archive) -> ReplayResult<()> {
        match archive.open_file("replay.attributes.events") {
            Ok(file) => {
                let file_size = file.size();
                let mut file_buf: Vec<u8> = vec![0; file_size as usize];

                match file.read(archive, file_buf.as_mut()) {
                    Ok(_) => {
                        let mut reader = BinaryReader::new(&file_buf);

                        // Skip the header.
                        reader.skip_bytes(5)?;

                        // Why this is LE, I have no fucking idea. *shrug*
                        let attribute_count = reader.read_u32_le()?;
                        let mut attributes: Vec<ReplayAttribute> = Vec::with_capacity(attribute_count as usize);

                        for _ in 0..attribute_count {
                            let header = reader.read_u32_le()?;
                            let type_val = ReplayAttributeEventType::from_u32(reader.read_u32_le()?);
                            let player_id = reader.read_u8()? as u32;

                            let mut attribute = ReplayAttribute {
                                header: header,
                                attribute_type: type_val,
                                player_id: player_id,
                                value: [0u8; 4],
                            };

                            reader.read_bytes_direct(&mut attribute.value)?;

                            attributes.push(attribute);
                        }

                        // Filter out unknown event types, and then sort ascending on the value of the event type.
                        attributes.retain(|x| x.attribute_type.is_some());
                        attributes.sort_by(|a, b| {
                            let aa = a.attribute_type.as_ref().unwrap().to_u64().unwrap();
                            let bb = b.attribute_type.as_ref().unwrap().to_u64().unwrap();

                            aa.cmp(&bb)
                        });

                        for attribute in attributes {
                            if !attribute.attribute_type.is_some() {
                                continue;
                            }

                            let attribute_type = attribute.attribute_type.unwrap();
                            match attribute_type {
                                ReplayAttributeEventType::PlayerTypeAttribute => {
                                    match replay.get_player_by_index(attribute.player_id - 1) {
                                        Some(player) => {
                                            match attribute.get_value_str() {
                                                Some(player_type) => {
                                                    match player_type.to_lowercase().as_ref() {
                                                        "comp" => {
                                                            player.player_type = PlayerType::Computer;
                                                        },
                                                        "humn" => {
                                                            player.player_type = PlayerType::Human;
                                                        },
                                                        "open" => {
                                                            // Less than 10 players in a Custom game
                                                        },
                                                        s => panic!("unexpected player type: {}", s)
                                                    }
                                                },
                                                None => {}
                                            }
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::TeamSizeAttribute => {
                                    match attribute.get_value_str() {
                                        Some(team_size) => {
                                            replay.team_size = TeamSize::from_str(&team_size);
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::DifficultyLevelAttribute => {
                                    match replay.get_player_by_index(attribute.player_id - 1) {
                                        Some(player) => {
                                            match attribute.get_value_str() {
                                                Some(difficulty) => {
                                                    player.difficulty = Difficulty::from_str(&difficulty);
                                                },
                                                None => {}
                                            }
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::GameSpeedAttribute => {
                                    match attribute.get_value_str() {
                                        Some(speed) => {
                                            replay.game_speed = GameSpeed::from_str(&speed);
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::GameTypeAttribute => {
                                    match attribute.get_value_str() {
                                        Some(game_type) => {
                                            match game_type.to_lowercase().as_ref() {
                                                "priv" => {
                                                    replay.game_mode = GameMode::Custom;
                                                },
                                                "amm" => {
                                                    if replay.replay_build < 33684 {
                                                        replay.game_mode = GameMode::QuickMatch;
                                                    }
                                                },
                                                s => panic!("unknown game type: {}", s)
                                            }
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::Hero | ReplayAttributeEventType::SkinAndSkinTint => {
                                    match replay.get_player_by_index(attribute.player_id - 1) {
                                        Some(player) => {
                                            match attribute.get_value_str() {
                                                Some(hero) => {
                                                    player.is_auto_select = hero == "Rand";
                                                },
                                                None => {}
                                            }
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::CharacterLevel => {
                                    match replay.get_player_by_index(attribute.player_id - 1) {
                                        Some(player) => {
                                            match attribute.get_value_int() {
                                                Some(level) => {
                                                    player.character_level = level;

                                                    if player.is_auto_select && player.character_level > 1 {
                                                        player.is_auto_select = false;
                                                    }
                                                },
                                                None => {}
                                            }
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::LobbyMode => {
                                    if replay.replay_build < 43905 && replay.game_mode != GameMode::Custom {
                                        match attribute.get_value_str() {
                                            Some(s) => match s.to_lowercase().as_ref() {
                                                "stan" => {
                                                    replay.game_mode = GameMode::QuickMatch;
                                                },
                                                "drft" => {
                                                    replay.game_mode = GameMode::HeroLeague;
                                                },
                                                s => panic!("unknown game mode: {}", s)
                                            },
                                            None => {}
                                        }
                                    }
                                },
                                ReplayAttributeEventType::ReadyMode => {
                                    if replay.replay_build < 43905 && replay.game_mode == GameMode::HeroLeague {
                                        match attribute.get_value_str() {
                                            Some(s) => {
                                                if &s == "fcfs" {
                                                    replay.game_mode = GameMode::TeamLeague;
                                                }
                                            },
                                            None => {}
                                        }
                                    }
                                },
                                ReplayAttributeEventType::DraftTeam1BanChooserSlot => {},
                                ReplayAttributeEventType::DraftTeam2BanChooserSlot => {},
                                ReplayAttributeEventType::DraftTeam1Ban1 => {
                                    match attribute.get_value_str() {
                                        Some(s) => {
                                            replay.bans.team_one_first_ban = s;
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::DraftTeam1Ban2 => {
                                    match attribute.get_value_str() {
                                        Some(s) => {
                                            replay.bans.team_one_second_ban = s;
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::DraftTeam2Ban1 => {
                                    match attribute.get_value_str() {
                                        Some(s) => {
                                            replay.bans.team_two_first_ban = s;
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::DraftTeam2Ban2 => {
                                    match attribute.get_value_str() {
                                        Some(s) => {
                                            replay.bans.team_two_second_ban = s;
                                        },
                                        None => {}
                                    }
                                },
                                ReplayAttributeEventType::Unknown => {},
                            }
                        }

                        Ok(())
                    },
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError,  "failed to read attributes file"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to open attributes file"))
        }
    }
}
