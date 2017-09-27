use std::io::Cursor;

use chrono::prelude::*;
use chrono::Duration;
use mpq::Archive;
use lazysort::Sorted;

use storm_parser::replay::StormReplay;
use storm_parser::tracker::TrackerEvent;
use storm_parser::primitives::{Player, PlayerType, ReplayResult, ReplayError, ReplayErrorKind};

pub struct ReplayDetails {
}

impl ReplayDetails {
    pub fn parse_replay_details(replay: &mut StormReplay, archive: &mut Archive) -> ReplayResult<()> {
        match archive.open_file("replay.details") {
            Ok(file) => {
                let file_size = file.size();
                let mut file_buf: Vec<u8> = vec![0; file_size as usize];

                match file.read(archive, file_buf.as_mut()) {
                    Ok(_) => {
                        let mut details_cursor = Cursor::new(file_buf);
                        match TrackerEvent::new(&mut details_cursor) {
                            Ok(mut event) => {
                                let mut players: Vec<Player> = Vec::new();
                                let players_array = event.get_dict_entry(0).get_optional_data().get_array();
                                for x in players_array {
                                    // Haven't really figured out why this has to be so dynamic/adaptive, since I can't imagine them
                                    // changing this often or ever?  Keeping it, though, because we're just trying to translate and
                                    // get things working before optimizing.
                                    let player_color = x.get_dict_entry(3).get_dict()
                                                    .keys()
                                                    .sorted()
                                                    .map(|i| x.get_dict_entry(3).get_dict_entry(*i).get_vint() as u32)
                                                    .collect();

                                    let player = Player {
                                        name: x.get_dict_entry(0).get_blob_text(),
                                        player_type: PlayerType::Human,
                                        battlenet_region_id: x.get_dict_entry(1).get_dict_entry(0).get_vint() as u32,
                                        battlenet_sub_id: x.get_dict_entry(1).get_dict_entry(2).get_vint() as u32,
                                        battlenet_id: x.get_dict_entry(1).get_dict_entry(4).get_vint() as u32,
                                        user_id: 0,
                                        slot_id: 0,
                                        color: player_color,
                                        team: x.get_dict_entry(5).get_vint() as u32,
                                        handicap: x.get_dict_entry(6).get_vint() as i32,
                                        is_winner: x.get_dict_entry(8).get_vint() == 1,
                                        character: x.get_dict_entry(10).get_blob_text(),
                                        is_silenced: false,
                                        skin: None,
                                        mount: None,
                                    };

                                    players.push(player);
                                }

                                /*let slot_id_data = event.get_dict_entry(0).get_optional_data().get_array();
                                for (i, player) in players.iter_mut().enumerate() {
                                    let slot_id = slot_id_data[i].get_dict_entry(9).get_vint() as u32;
                                    player.slot_id = slot_id;
                                }*/

                                replay.players = players;
                                replay.map = event.get_dict_entry(1).get_blob_text();
                                replay.timestamp = get_timestamp_from_file_time(event.get_dict_entry(5).get_vint());

                                // Again, from barrett777, there were some builds with messed up timestamps and so we'll just hard-code
                                // them if we see them to a date that was within the window of when the build was live.
                                if replay.replay_build == 34053 && replay.timestamp < Utc.ymd(2015, 2, 8).and_hms(0, 0, 0) {
                                    replay.timestamp = Utc.ymd(2015, 2, 13).and_hms(0, 0, 0);
                                } else if replay.replay_build == 34190 && replay.timestamp < Utc.ymd(2015, 2, 15).and_hms(0, 0, 0) {
                                    replay.timestamp = Utc.ymd(2015, 2, 20).and_hms(0, 0, 0);
                                }

                                Ok(())
                            },
                            Err(_) => Err(ReplayError::new(ReplayErrorKind::StructureError, "failed to parse details structure"))
                        }
                    },
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to read details file"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to open details file"))
        }
    }
}

fn get_timestamp_from_file_time(file_time: i64) -> DateTime<Utc> {
    let epoch = Utc.ymd(1601, 1, 1).and_hms(0, 0, 0);
    let ft_duration = Duration::milliseconds(file_time / 10000);
    epoch + ft_duration
}
