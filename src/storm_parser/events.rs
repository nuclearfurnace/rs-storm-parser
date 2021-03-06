use std::io::Error;

use mpq::Archive;
use num_traits::FromPrimitive;

use storm_parser::replay::StormReplay;
use storm_parser::binary_reader::BinaryReader;
use storm_parser::tracker::TrackerEventStructure;
use storm_parser::primitives::*;

#[derive(Serialize, Default, Debug)]
pub struct GameEvent {
    pub event_type: ReplayGameEventType,
    pub ticks_elapsed: u32,
    pub player: Option<u32>,
    pub is_global: bool,
    pub data: Option<TrackerEventStructure>,
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
                        let mut game_events: Vec<GameEvent> = Vec::new();
                        let mut ticks_elapsed: u32 = 0;

                        while !reader.eof() {
                            let mut game_event: GameEvent = Default::default();

                            let ticks_multiplier = reader.read_vu32(2)? << 3;
                            let ticks_delta = reader.read_vu32(6 + ticks_multiplier)?;
                            ticks_elapsed += ticks_delta;
                            game_event.ticks_elapsed = ticks_elapsed;

                            match reader.read_vu32(5)? {
                                16 => {
                                    game_event.is_global = true;
                                },
                                i => {
                                    game_event.player = Some(i);
                                }
                            };

                            let event_type_raw = reader.read_vu32(7)?;
                            let event_type = ReplayGameEventType::from_u32(event_type_raw)
                                .ok_or(ReplayError::new(ReplayErrorKind::StructureError,
                                    &format!("unknown game event type '{}'", event_type_raw)))?;
                            game_event.event_type = event_type;

                            game_event.data = match event_type {
                                ReplayGameEventType::Unknown => None,
                                ReplayGameEventType::DropOurselvesEvent => None,
                                ReplayGameEventType::StartGameEvent => None,
                                ReplayGameEventType::UserFinishedLoadingSyncEvent => None,
                                ReplayGameEventType::UserOptionsEvent => {
                                    let mut event = get_tracker_event_array(14);

                                    event.array[0] = get_tracker_event_bool(&mut reader)?; // m_gameFullyDownloaded
                                    event.array[1] = get_tracker_event_bool(&mut reader)?; // m_developmentCheatsEnabled
                                    event.array[2] = get_tracker_event_bool(&mut reader)?; // m_testCheatsEnabled
                                    event.array[3] = get_tracker_event_bool(&mut reader)?; // m_multiplayerCheatsEnabled
                                    event.array[4] = get_tracker_event_bool(&mut reader)?; // m_syncChecksummingEnabled
                                    event.array[5] = get_tracker_event_bool(&mut reader)?; // m_isMapToMapTransition
                                    event.array[6] = get_tracker_event_bool(&mut reader)?; // m_debugPauseEnabled
                                    event.array[7] = get_tracker_event_bool(&mut reader)?; // m_useGalaxyAsserts
                                    event.array[8] = get_tracker_event_bool(&mut reader)?; // m_platformMac
                                    event.array[9] = get_tracker_event_bool(&mut reader)?; // m_cameraFollow
                                    event.array[10] = get_tracker_event_u32(&mut reader)?; // m_baseBuildNum
                                    event.array[11] = get_tracker_event_u32(&mut reader)?; // m_buildNum
                                    event.array[12] = get_tracker_event_u32(&mut reader)?; // m_versionFlags
                                    event.array[13] = get_tracker_event_blob(&mut reader, 9)?; // m_hotkeyProfile

                                    Some(event)
                                },
                                ReplayGameEventType::BankFileEvent => {
                                    Some(get_tracker_event_blob(&mut reader, 7)?)
                                },
                                ReplayGameEventType::BankSectionEvent => {
                                    Some(get_tracker_event_blob(&mut reader, 6)?)
                                },
                                ReplayGameEventType::BankKeyEvent => {
                                    let mut event = get_tracker_event_array(3);
                                    event.array[0] = get_tracker_event_blob(&mut reader, 6)?;
                                    event.array[1] = get_tracker_event_u32(&mut reader)?;
                                    event.array[2] = get_tracker_event_blob(&mut reader, 7)?;

                                    Some(event)
                                },
                                ReplayGameEventType::BankSignatureEvent => {
                                    let array_len = reader.read_vu32(5)?;
                                    let mut event = get_tracker_event_array(array_len);
                                    for _i in 0..array_len {
                                        event.array.push(get_tracker_event_uint(&mut reader, 8)?);
                                    }
                                    event.blob = Some(reader.read_len_prefixed_blob(7)?);

                                    Some(event)
                                },
                                ReplayGameEventType::CameraSaveEvent => {
                                    reader.read_vu32(3)?; // m_which
                                    reader.read_vu32(16)?; // x
                                    reader.read_vu32(16)?; // y
                                    None
                                },
                                ReplayGameEventType::CommandManagerResetEvent => {
                                    reader.read_u32()?; // m_sequence
                                    None
                                },
                                ReplayGameEventType::GameCheatEvent => {
                                    // m_target
                                    let mut event = get_tracker_event_array(4);

                                    event.array[0] = match reader.read_vu32(2)? {
                                        1 => get_tracker_event_point3d(&mut reader)?, // TargetPoint
                                        2 => get_tracker_event_target_unit(&mut reader)?, // TargetUnit
                                        _ => get_tracker_event_empty() // None
                                    };

                                    reader.read_u32()?; // m_time
                                    reader.read_len_prefixed_string(10)?; // m_verb
                                    reader.read_len_prefixed_string(10)?; // m_arguments

                                    Some(event)
                                },
                                ReplayGameEventType::CmdEvent => {
                                    let mut event = get_tracker_event_array(5);

                                    // m_cmdFlags
                                    let cmd_flags_len = if replay.replay_build < 33684     { 22 }
                                                   else if replay.replay_build < 37117     { 23 }
                                                   else if replay.replay_build < 38236     { 24 }
                                                   else if replay.replay_build < 42958     { 25 }
                                                   else if replay.replay_build < 44256     { 24 }
                                                   else if replay.replay_build <= 45635    { 26 }
                                                   else if replay.replay_version_major < 2 { 25 }
                                                   else                                    { 26 };

                                    let mut cmd_flags = get_tracker_event_array(cmd_flags_len);
                                    for i in 0..cmd_flags_len {
                                        cmd_flags.array[i as usize] = get_tracker_event_bool(&mut reader)?;
                                    }
                                    event.array[0] = cmd_flags;

                                    // m_abil
                                    if reader.read_bool()? {
                                        let mut array = get_tracker_event_array(3);

                                        array.array[0] = get_tracker_event_uint(&mut reader, 16)?; // m_abilLink
                                        array.array[1] = get_tracker_event_uint(&mut reader, 5)?; // m_abilCmdIndex
                                        if reader.read_bool()? {
                                            array.array[2] = get_tracker_event_uint(&mut reader, 8)?; // m_abilCmdData
                                        }

                                        event.array[1] = array;
                                    }

                                    // m_data
                                    event.array[2] = match reader.read_vu32(2)? {
                                        1 => get_tracker_event_point3d(&mut reader)?, // TargetPoint
                                        2 => get_tracker_event_target_unit(&mut reader)?, // TargetUnit
                                        3 => get_tracker_event_u32(&mut reader)?, // Data
                                        _ => get_tracker_event_empty(), // None or unknown
                                    };

                                    // m_vector
                                    if replay.replay_build >= 44256 && reader.read_bool()? {
                                        get_tracker_event_point3d(&mut reader)?;
                                    }

                                    if replay.replay_build >= 33684 {
                                        reader.read_vu32(32)?; // m_sequence
                                    }
                                    if reader.read_bool()? {
                                        event.array[3] = get_tracker_event_u32(&mut reader)?; // m_otherUnit
                                    }
                                    if reader.read_bool()? {
                                        event.array[4] = get_tracker_event_u32(&mut reader)?; // m_unitGroup
                                    }

                                    Some(event)
                                },
                                ReplayGameEventType::SelectionDeltaEvent => {
                                    let mut event = get_tracker_event_array(2);

                                    event.array[0] = get_tracker_event_uint(&mut reader, 4)?; // m_controlGroupId

                                    let array_bit_len = if replay.replay_version_major < 2 { 9 } else { 6 };
                                    let index_bit_len = if replay.replay_version_major < 2 { 9 } else { 5 };

                                    // m_delta
                                    let mut delta = get_tracker_event_array(4);
                                    delta.array[0] = get_tracker_event_uint(&mut reader, index_bit_len)?;

                                    // m_removeMask
                                    match reader.read_vu32(2)? {
                                        0 => {}, // None
                                        1 => { // Mask
                                            let mask_bits = reader.read_vu32(array_bit_len)?;
                                            reader.read_vu32(mask_bits)?;
                                        },
                                        2 | 3 => { // OneIndices or ZeroIndices
                                            let array_len = reader.read_vu32(array_bit_len)?;
                                            let mut array = get_tracker_event_array(array_len);
                                            for i in 0..array_len {
                                                array.array[i as usize] = get_tracker_event_uint(&mut reader, index_bit_len)?;
                                            }

                                            delta.array[1] = array;
                                        },
                                        _ => panic!("unknown m_removeMask value")
                                    }

                                    // m_addSubgroups
                                    let subgroup_array_len = reader.read_vu32(array_bit_len)?;
                                    let mut subgroup_array = get_tracker_event_array(subgroup_array_len);
                                    for i in 0..subgroup_array_len {
                                        let mut array = get_tracker_event_array(4);
                                        array.array[0] = get_tracker_event_uint(&mut reader, 16)?;
                                        array.array[1] = get_tracker_event_uint(&mut reader, 8)?;
                                        array.array[2] = get_tracker_event_uint(&mut reader, 8)?;
                                        array.array[3] = get_tracker_event_uint(&mut reader, array_bit_len)?;

                                        subgroup_array.array[i as usize] = array;
                                    }
                                    delta.array[2] = subgroup_array;

                                    // m_addUnitTags
                                    let unit_array_len = reader.read_vu32(array_bit_len)?;
                                    let mut unit_array = get_tracker_event_array(unit_array_len);
                                    for i in 0..unit_array_len {
                                        unit_array.array[i as usize] = get_tracker_event_u32(&mut reader)?;
                                    }
                                    delta.array[3] = unit_array;

                                    event.array[1] = delta;

                                    Some(event)
                                },
                                ReplayGameEventType::ControlGroupUpdateEvent => {
                                    reader.read_vu32(4)?; // m_controlGroupIndex

                                    // m_controlGroupUpdate
                                    if replay.replay_build < 36359 { // Not sure exactly when this change happened - roughly around here.  This primarily affected 'The Lost Vikings' hero
                                        reader.read_vu32(2)?;
                                    } else {
                                        reader.read_vu32(3)?;
                                    }

                                    // m_mask
                                    let bit_len = if replay.replay_version_major < 2 { 9 } else { 6 };
                                    match reader.read_vu32(2)? {
                                        1 => { // Mask
                                            let mask_len = reader.read_vu32(bit_len)?;
                                            reader.read_vu32(mask_len)?;
                                            None
                                        },
                                        2 | 3 => { // OneIndices or ZeroIndices
                                            let value_bit_len = if replay.replay_version_major < 2 { 9 } else { 5 };
                                            let array_len = reader.read_vu32(bit_len)?;
                                            let mut event = get_tracker_event_array(array_len);
                                            for i in 0..array_len {
                                                event.array[i as usize] = get_tracker_event_uint(&mut reader, value_bit_len)?;
                                            }

                                            Some(event)
                                        },
                                        _ => None
                                    }
                                },
                                ReplayGameEventType::SelectionSyncCheckEvent => {
                                    reader.read_vu32(4)?; // m_controlGroupId

                                    // m_selectionSyncData
                                    if replay.replay_version_major < 2 {
                                        reader.read_vu32(9)?; // m_count
                                        reader.read_vu32(9)?; // m_subgroupCount
                                        reader.read_vu32(9)?; // m_activeSubgroupIndex
                                    } else {
                                        reader.read_vu32(6)?; // m_count
                                        reader.read_vu32(6)?; // m_subgroupCount
                                        reader.read_vu32(5)?; // m_activeSubgroupIndex
                                    }

                                    reader.read_u32()?; // m_unitTagsChecksum
                                    reader.read_u32()?; // m_subgroupIndicesChecksum
                                    reader.read_u32()?; // m_subgroupsChecksum

                                    None
                                },
                                ReplayGameEventType::ResourceTradeEvent => {
                                    reader.read_vu32(4)?; // m_recipientId
                                    reader.read_i32()?; // m_resources, should be offset -2147483648
                                    reader.read_i32()?; // m_resources, should be offset -2147483648
                                    reader.read_i32()?; // m_resources, should be offset -2147483648

                                    None
                                },
                                ReplayGameEventType::TriggerChatMessageEvent => {
                                    Some(get_tracker_event_blob(&mut reader, 10)?)
                                },
                                ReplayGameEventType::SetAbsoluteGameSpeedEvent => {
                                    reader.read_vu32(3)?; // m_speed
                                    None
                                },
                                ReplayGameEventType::TriggerPingEvent => {
                                    let mut event = get_tracker_event_array(5);
                                    event.array[0] = get_tracker_event_i32(&mut reader)?;
                                    event.array[1] = get_tracker_event_i32(&mut reader)?;
                                    event.array[2] = get_tracker_event_u32(&mut reader)?;
                                    event.array[3] = get_tracker_event_bool(&mut reader)?;
                                    event.array[4] = get_tracker_event_i32(&mut reader)?;

                                    Some(event)
                                },
                                ReplayGameEventType::UnitClickEvent => {
                                    Some(get_tracker_event_u32(&mut reader)?) // m_unitTag
                                },
                                ReplayGameEventType::TriggerSkippedEvent => None,
                                ReplayGameEventType::TriggerSoundLengthQueryEvent => {
                                    let mut event = get_tracker_event_array(2);
                                    event.array[0] = get_tracker_event_u32(&mut reader)?;
                                    event.array[1] = get_tracker_event_u32(&mut reader)?;

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerSoundOffsetEvent => {
                                    Some(get_tracker_event_u32(&mut reader)?)
                                },
                                ReplayGameEventType::TriggerTransmissionOffsetEvent => {
                                    let mut event = get_tracker_event_array(2);
                                    event.array[0] = get_tracker_event_i32(&mut reader)?;
                                    event.array[1] = get_tracker_event_u32(&mut reader)?;

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerTransmissionCompleteEvent => {
                                    Some(get_tracker_event_i32(&mut reader)?)
                                },
                                ReplayGameEventType::CameraUpdateEvent => {
                                    let mut event = get_tracker_event_array(6);

                                    if reader.read_bool()? {
                                        // m_target, x/y
                                        let mut array = get_tracker_event_array(2);
                                        array.array[0] = get_tracker_event_uint(&mut reader, 16)?;
                                        array.array[1] = get_tracker_event_uint(&mut reader, 16)?;

                                        event.array[0] = array;
                                    }
                                    if reader.read_bool()? {
                                        // m_distance
                                        event.array[1] = get_tracker_event_uint(&mut reader, 16)?;
                                    }
                                    if reader.read_bool()? {
                                        // m_pitch
                                        event.array[2] = get_tracker_event_uint(&mut reader, 16)?;
                                    }
                                    if reader.read_bool()? {
                                        // m_yaw
                                        event.array[3] = get_tracker_event_uint(&mut reader, 16)?;
                                    }
                                    if reader.read_bool()? {
                                        // m_reason
                                        event.array[4] = get_tracker_event_i8(&mut reader)?;
                                    }

                                    // m_follow
                                    event.array[5] = get_tracker_event_bool(&mut reader)?;

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerPlanetMissionLaunchedEvent => {
                                    reader.skip_bytes(4)?; // m_difficultyLevel, i32
                                    None
                                },
                                ReplayGameEventType::TriggerDialogControlEvent => {
                                    let mut event = get_tracker_event_array(3);
                                    event.array[0] = get_tracker_event_vint(&mut reader, 32)?;
                                    event.array[1] = get_tracker_event_vint(&mut reader, 32)?;

                                    event.array[2] = match reader.read_vu32(3)? {
                                        1 => get_tracker_event_bool(&mut reader)?, // Checked
                                        2 => get_tracker_event_u32(&mut reader)?, // ValueChanged
                                        3 => get_tracker_event_i32(&mut reader)?, // SelectionChanged
                                        4 => get_tracker_event_blob(&mut reader, 11)?, // TextChanged
                                        5 => get_tracker_event_u32(&mut reader)?, // MouseButton
                                        _ => get_tracker_event_empty(), // None (0) or unknown
                                    };

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerSoundLengthSyncEvent => {
                                    let mut event = get_tracker_event_array(2);

                                    let first_array_len = reader.read_vu32(7)?;
                                    let mut first_array = get_tracker_event_array(first_array_len);
                                    for i in 0..first_array_len {
                                        first_array.array[i as usize] = get_tracker_event_u32(&mut reader)?;
                                    }

                                    let second_array_len = reader.read_vu32(7)?;
                                    let mut second_array = get_tracker_event_array(first_array_len);
                                    for i in 0..second_array_len {
                                        second_array.array[i as usize] = get_tracker_event_u32(&mut reader)?;
                                    }

                                    event.array[0] = first_array;
                                    event.array[1] = second_array;

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerConversationSkippedEvent => {
                                    Some(get_tracker_event_bool(&mut reader)?)
                                },
                                ReplayGameEventType::TriggerMouseClickedEvent => {
                                    /*let mut event = get_tracker_event_array(6);

                                    event.array[0] = get_tracker_event_u32(&mut reader)?; // m_button
                                    event.array[1] = get_tracker_event_bool(&mut reader)?; // m_down
                                    event.array[2] = get_tracker_event_uint(&mut reader, 11)?; // m_posUI, X
                                    event.array[3] = get_tracker_event_uint(&mut reader, 11)?; // m_posUI, Y
                                    event.array[4] = get_tracker_event_point3d(&mut reader)?; // m_posWorld, XYZ
                                    event.array[5] = get_tracker_event_i8(&mut reader)?; // m_flags

                                    Some(event)*/
                                    reader.skip_bytes(17)?;
                                    None
                                },
                                ReplayGameEventType::TriggerMouseMovedEvent => {
                                    /*let mut event = get_tracker_event_array(4);

                                    event.array[0] = get_tracker_event_uint(&mut reader, 11)?; // m_posUI, X
                                    event.array[1] = get_tracker_event_uint(&mut reader, 11)?; // m_posUI, Y
                                    event.array[2] = get_tracker_event_point3d(&mut reader)?; // m_posWorld, XYZ
                                    event.array[3] = get_tracker_event_i8(&mut reader)?; // m_flags

                                    Some(event)*/
                                    reader.skip_bytes(13)?;
                                    None
                                },
                                ReplayGameEventType::TriggerHotkeyPressedEvent => {
                                    Some(get_tracker_event_u32(&mut reader)?)
                                },
                                ReplayGameEventType::TriggerTargetModeUpdateEvent => {
                                    reader.read_vu32(16)?; // m_abilLink
                                    reader.read_vu32(5)?; // m_abilCmdIndex
                                    reader.read_vu32(8)?; // m_state (-128)
                                    None
                                },
                                ReplayGameEventType::TriggerSoundtrackDoneEvent => {
                                    Some(get_tracker_event_u32(&mut reader)?)
                                },
                                ReplayGameEventType::TriggerKeyPressedEvent => {
                                    let mut event = get_tracker_event_array(2);
                                    event.array[0] = get_tracker_event_i8(&mut reader)?;
                                    event.array[1] = get_tracker_event_i8(&mut reader)?;

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerCutsceneBookmarkFiredEvent => {
                                    let mut event = get_tracker_event_array(2);
                                    event.array[0] = get_tracker_event_i32(&mut reader)?; // m_cutsceneId
                                    event.array[1] = get_tracker_event_blob(&mut reader, 7)?; // m_bookmarkName

                                    Some(event)
                                },
                                ReplayGameEventType::TriggerCutsceneEndSceneFiredEvent => {
                                    // m_cutsceneId
                                    Some(get_tracker_event_i32(&mut reader)?)
                                },
                                ReplayGameEventType::GameUserLeaveEvent => {
                                    // m_leaveReason
                                    if replay.replay_build >= 55929 {
                                        reader.read_vu32(5)?;
                                    } else {
                                        reader.read_vu32(4)?;
                                    }

                                    None
                                },
                                ReplayGameEventType::GameUserJoinEvent => {
                                    let mut event = get_tracker_event_array(5);
                                    event.array[0] = get_tracker_event_uint(&mut reader, 2)?;
                                    event.array[1] = get_tracker_event_blob(&mut reader, 8)?;
                                    if reader.read_bool()? {
                                        event.array[2] = get_tracker_event_blob(&mut reader, 7)?;
                                    }
                                    if reader.read_bool()? {
                                        event.array[3] = get_tracker_event_blob(&mut reader, 8)?;
                                    }
                                    if reader.read_bool()? {
                                        event.array[4] = get_tracker_event_bytes(&mut reader, 40)?;
                                    }

                                    Some(event)
                                },
                                ReplayGameEventType::CommandManagerStateEvent => {
                                    let mut event = get_tracker_event_uint(&mut reader, 2)?; // m_state
                                    if replay.replay_build >= 33684 {
                                        if reader.read_bool()? {
                                            // m_sequence
                                            let default = get_tracker_event_empty();
                                            let mut array = vec![default; 3];
                                            array[0] = get_tracker_event_vint(&mut reader, 8)?;
                                            array[1] = get_tracker_event_vint(&mut reader, 8)?;
                                            array[2] = get_tracker_event_vint(&mut reader, 16)?;

                                            event.array = array;
                                        }
                                    }

                                    Some(event)
                                },
                                ReplayGameEventType::CmdUpdateTargetPointEvent => {
                                    if replay.replay_build >= 40336 && reader.read_bool()? {
                                        reader.skip_bytes(4)?;
                                    }

                                    Some(get_tracker_event_point3d(&mut reader)?)
                                },
                                ReplayGameEventType::CmdUpdateTargetUnitEvent => {
                                    if replay.replay_build >= 40336 && reader.read_bool()? {
                                        reader.skip_bytes(4)?;
                                    }

                                    Some(get_tracker_event_target_unit(&mut reader)?)
                                },
                                ReplayGameEventType::HeroTalentSelectedEvent => {
                                    Some(get_tracker_event_u32(&mut reader)?) // m_index
                                },
                                ReplayGameEventType::HeroTalentTreeSelectionPanelToggled => {
                                    Some(get_tracker_event_bool(&mut reader)?) // m_shown
                                }
                            };

                            reader.align();
                            game_events.push(game_event);
                        }

                        replay.game_events = game_events;

                        Ok(())
                    },
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError,  "failed to read game events file"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to open game events file"))
        }
    }
}

fn get_tracker_event_empty() -> TrackerEventStructure {
    Default::default()
}

fn get_tracker_event_array(slots: u32) -> TrackerEventStructure {
    let mut event = get_tracker_event_empty();
    let default = get_tracker_event_empty();
    event.data_type = 0x00;
    event.array = vec![default; slots as usize];
    event
}

fn get_tracker_event_bool(reader: &mut BinaryReader) -> Result<TrackerEventStructure, Error> {
    get_tracker_event_uint(reader, 1)
}

fn get_tracker_event_uint(reader: &mut BinaryReader, bits: u32) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let uint = reader.read_vu32(bits)?;
    event.data_type = 0x07;
    event.unsigned_int = Some(uint as u64);
    Ok(event)
}

fn get_tracker_event_vint(reader: &mut BinaryReader, bits: u32) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let vint = reader.read_vu32(bits)? as i64;
    event.data_type = 0x09;
    event.variable_int = Some(vint);
    Ok(event)
}

fn get_tracker_event_i8(reader: &mut BinaryReader) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let vint = (reader.read_vu32(8)? as i8) as i64;
    event.data_type = 0x09;
    event.variable_int = Some(vint);
    Ok(event)
}

fn get_tracker_event_i32(reader: &mut BinaryReader) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let vint = (reader.read_vu32(32)? as i32) as i64;
    event.data_type = 0x09;
    event.variable_int = Some(vint);
    Ok(event)
}

fn get_tracker_event_u32(reader: &mut BinaryReader) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let uint = reader.read_u32()?;
    event.data_type = 0x07;
    event.unsigned_int = Some(uint as u64);
    Ok(event)
}

fn get_tracker_event_blob(reader: &mut BinaryReader, blob_len: u32) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let blob = reader.read_len_prefixed_blob(blob_len)?;
    event.data_type = 0x02;
    event.blob = Some(blob);
    Ok(event)
}

fn get_tracker_event_bytes(reader: &mut BinaryReader, bytes: u32) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_empty();
    let bytes = reader.read_bytes(bytes)?;
    event.data_type = 0x02;
    event.blob = Some(bytes);
    Ok(event)
}

fn get_tracker_event_point3d(reader: &mut BinaryReader) -> Result<TrackerEventStructure, Error> {
    let x = get_tracker_event_uint(reader, 20)?;
    let y = get_tracker_event_uint(reader, 20)?;
    let z = get_tracker_event_i32(reader)?;

    let mut array = get_tracker_event_array(3);
    array.array[0] = x;
    array.array[1] = y;
    array.array[2] = z;

    Ok(array)
}

fn get_tracker_event_target_unit(reader: &mut BinaryReader) -> Result<TrackerEventStructure, Error> {
    let mut event = get_tracker_event_array(7);
    event.array[0] = get_tracker_event_uint(reader, 16)?; // m_targetUnitFlags
    event.array[1] = get_tracker_event_uint(reader, 8)?; // m_timer
    event.array[2] = get_tracker_event_u32(reader)?; // m_tag
    event.array[3] = get_tracker_event_uint(reader, 16)?; // m_snapshotUnitLink
    if reader.read_bool()? {
        event.array[4] = get_tracker_event_uint(reader, 4)?; // m_snapshotControlPlayerId
    }
    if reader.read_bool()? {
        event.array[5] = get_tracker_event_uint(reader, 4)?; // m_snapshotUpkeepPlayerId
    }
    event.array[6] = get_tracker_event_point3d(reader)?; // m_snapshotPoint (X, Y, Z)

    Ok(event)
}
