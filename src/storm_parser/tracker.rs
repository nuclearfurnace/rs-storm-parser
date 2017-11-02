use std::io;
use std::collections::HashMap;

use mpq::Archive;
use num_traits::FromPrimitive;

use storm_parser::replay::StormReplay;
use storm_parser::binary_reader::BinaryReader;
use storm_parser::primitives::*;

#[derive(Serialize, Clone, Debug, Default)]
pub struct TrackerEventStructure {
    pub(crate) data_type: u32,
    #[serde(skip_serializing_if = "is_vec_empty")]
    pub(crate) array: Vec<TrackerEventStructure>,
    #[serde(skip_serializing_if = "is_map_empty")]
    pub(crate) dictionary: HashMap<i32, TrackerEventStructure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) blob: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) choice_flag: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) choice_data: Option<Box<TrackerEventStructure>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) optional_data: Option<Box<TrackerEventStructure>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unsigned_int: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) variable_int: Option<i64>,
}

impl TrackerEventStructure {
    pub fn new(r: &mut BinaryReader) -> ReplayResult<TrackerEventStructure> {
        let mut event: TrackerEventStructure = Default::default();

        event.data_type = r.read_u8()? as u32;
        match event.data_type {
            0x00 => {
                let array_len = read_variable_int(r)?;
                let mut array: Vec<TrackerEventStructure> = Vec::new();
                for _ in 0..array_len {
                    let event = TrackerEventStructure::new(r)?;
                    array.push(event);
                }

                event.array = array;
            },
            0x01 => panic!("unsupported tracker event type"),
            0x02 => {
                let blob_len = read_variable_int(r)?;
                let buf = r.read_bytes(blob_len as u32)?;

                event.blob = Some(buf);
            },
            0x03 => {
                let choice_flag = read_variable_int(r)? as i32;
                let choice_data = TrackerEventStructure::new(r)?;

                event.choice_flag = Some(choice_flag);
                event.choice_data = Some(Box::new(choice_data));
            },
            0x04 => {
                let should_read = r.read_u8()?;
                if should_read != 0 {
                    let optional_data = TrackerEventStructure::new(r)?;
                    event.optional_data = Some(Box::new(optional_data));
                }
            },
            0x05 => {
                // dictionary, read size as variable int, and for N, read key
                // as variable int and then the value as a tracking event
                let mut dictionary: HashMap<i32, TrackerEventStructure> = HashMap::new();
                let dictionary_len = read_variable_int(r)?;
                for _ in 0..dictionary_len {
                    let key = read_variable_int(r)? as i32;
                    let value = TrackerEventStructure::new(r)?;

                    dictionary.insert(key, value);
                }

                event.dictionary = dictionary;
            },
            0x06 => {
                event.unsigned_int = Some(r.read_u8()? as u64);
            },
            0x07 => {
                event.unsigned_int = Some(r.read_u32_le()? as u64);
            },
            0x08 => {
                event.unsigned_int = Some(r.read_u64_le()?);
            },
            0x09 => {
                event.variable_int = Some(read_variable_int(r)?);
            },
            _ => panic!("unsupported tracker event type")
        }

        Ok(event)
    }

    pub fn get_array(&self) -> &Vec<TrackerEventStructure> {
        &self.array
    }

    pub fn get_dict(&self) -> &HashMap<i32, TrackerEventStructure> {
        &self.dictionary
    }

    pub fn get_dict_entry(&self, index: i32) -> &TrackerEventStructure {
        self.dictionary.get(&index).unwrap()
    }

    pub fn get_blob(&self) -> &Vec<u8> {
        self.blob.as_ref().unwrap().as_ref()
    }
    pub fn get_blob_text(&self) -> String {
        let blob = self.get_blob().clone();
        String::from_utf8(blob).unwrap()
    }

    pub fn get_choice_flag(&self) -> i32 {
        self.choice_flag.unwrap()
    }

    pub fn get_choice_data(&self) -> &TrackerEventStructure {
        self.choice_data.as_ref().unwrap().as_ref()
    }

    pub fn get_optional_data(&self) -> &TrackerEventStructure {
        self.optional_data.as_ref().unwrap().as_ref()
    }

    pub fn get_uint(&self) -> u64 {
        self.unsigned_int.unwrap()
    }

    pub fn get_vint(&self) -> i64 {
        self.variable_int.unwrap()
    }
}

fn read_variable_int(r: &mut BinaryReader) -> Result<i64, io::Error> {
    let mut x: i64 = 0;

    let mut k: u64 = 0;
    loop {
        let next = r.read_u8()? as i64;

        x = x | ((next & 0x7F) << k) as i64;
         if next & 0x80 == 0 {
            break
        }

        k = k + 7;
    }

     if x & 1 > 0 {
        Ok(-(x >> 1))
    } else {
        Ok(x >> 1)
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct TrackerEvent {
    pub event_type: ReplayTrackerEventType,
    pub ticks_elapsed: u32,
    pub data: TrackerEventStructure
}

pub struct ReplayTrackerEvents {
}

impl ReplayTrackerEvents {
    pub fn parse_replay_tracker_events(replay: &mut StormReplay, archive: &mut Archive) -> ReplayResult<()> {
        match archive.open_file("replay.tracker.events") {
            Ok(file) => {
                let file_size = file.size();
                let mut file_buf: Vec<u8> = vec![0; file_size as usize];

                match file.read(archive, file_buf.as_mut()) {
                    Ok(_) => {
                        let mut reader = BinaryReader::new(&file_buf);
                        let mut tracker_events: Vec<TrackerEvent> = Vec::new();
                        let mut ticks_elapsed: u32 = 0;

                        while !reader.eof() {
                            let mut tracker_event: TrackerEvent = Default::default();

                            // Per barrett777's notes, this is usually 03 ?? 09, where the middle byte has been at least two distinct values.
                            reader.read_bytes(3)?;

                            let ticks_delta = read_variable_int(&mut reader)?;
                            ticks_elapsed += ticks_delta as u32;
                            tracker_event.ticks_elapsed = ticks_elapsed;

                            let tracker_event_type_raw = read_variable_int(&mut reader)?;
                            let tracker_event_type = ReplayTrackerEventType::from_u32(tracker_event_type_raw as u32)
                                .ok_or(ReplayError::new(ReplayErrorKind::StructureError, "unknown tracker event type"))?;
                            tracker_event.event_type = tracker_event_type;

                            let tracker_data = TrackerEventStructure::new(&mut reader)?;
                            if tracker_event_type == ReplayTrackerEventType::StatGameEvent &&
                                tracker_data.get_dict_entry(3).optional_data.is_some() {
                                let mut optional_data = tracker_data.get_dict_entry(3).get_optional_data().get_array();
                                for item in optional_data {
                                    let item_vint_value = item.get_dict_entry(1).get_vint();
                                    item.get_dict_entry(1).variable_int = Some(item_vint_value / 4096);
                                }
                            }

                            tracker_event.data = tracker_data;

                            tracker_events.push(tracker_event)
                        }

                        replay.tracker_events = tracker_events;

                        Ok(())
                    },
                    Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError,  "failed to read tracker events file"))
                }
            },
            Err(_) => Err(ReplayError::new(ReplayErrorKind::ArchiveError, "failed to open tracker events file"))
        }
    }
}

fn is_vec_empty(v: &Vec<TrackerEventStructure>) -> bool {
    v.len() == 0
}

fn is_map_empty(m: &HashMap<i32, TrackerEventStructure>) -> bool {
    m.len() == 0
}
