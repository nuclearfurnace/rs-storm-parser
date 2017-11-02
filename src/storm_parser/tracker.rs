use std::io;
use std::collections::HashMap;

use storm_parser::binary_reader::BinaryReader;
use storm_parser::primitives::*;

#[derive(Serialize, Clone, Debug, Default)]
pub struct TrackerEvent {
    pub(crate) data_type: u32,
    #[serde(skip_serializing_if = "is_vec_empty")]
    pub(crate) array: Vec<TrackerEvent>,
    #[serde(skip_serializing_if = "is_map_empty")]
    pub(crate) dictionary: HashMap<i32, TrackerEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) blob: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) choice_flag: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) choice_data: Option<Box<TrackerEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) optional_data: Option<Box<TrackerEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unsigned_int: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) variable_int: Option<i64>,
}

impl TrackerEvent {
    pub fn new(r: &mut BinaryReader) -> ReplayResult<TrackerEvent> {
        let mut event: TrackerEvent = Default::default();

        event.data_type = r.read_u8()? as u32;
        match event.data_type {
            0x00 => {
                let array_len = TrackerEvent::read_variable_int(r)?;
                let mut array: Vec<TrackerEvent> = Vec::new();
                for _ in 0..array_len {
                    let event = TrackerEvent::new(r)?;
                    array.push(event);
                }

                event.array = array;
            },
            0x01 => panic!("unsupported tracker event type"),
            0x02 => {
                let blob_len = TrackerEvent::read_variable_int(r)?;
                let buf = r.read_bytes(blob_len as u32)?;

                event.blob = Some(buf);
            },
            0x03 => {
                let choice_flag = TrackerEvent::read_variable_int(r)? as i32;
                let choice_data = TrackerEvent::new(r)?;

                event.choice_flag = Some(choice_flag);
                event.choice_data = Some(Box::new(choice_data));
            },
            0x04 => {
                let should_read = r.read_u8()?;
                if should_read != 0 {
                    let optional_data = TrackerEvent::new(r)?;
                    event.optional_data = Some(Box::new(optional_data));
                }
            },
            0x05 => {
                // dictionary, read size as variable int, and for N, read key
                // as variable int and then the value as a tracking event
                let mut dictionary: HashMap<i32, TrackerEvent> = HashMap::new();
                let dictionary_len = TrackerEvent::read_variable_int(r)?;
                for _ in 0..dictionary_len {
                    let key = TrackerEvent::read_variable_int(r)? as i32;
                    let value = TrackerEvent::new(r)?;

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
                event.variable_int = Some(TrackerEvent::read_variable_int(r)?);
            },
            _ => panic!("unsupported tracker event type")
        }

        Ok(event)
    }

    pub fn get_array(&self) -> &Vec<TrackerEvent> {
        &self.array
    }

    pub fn get_dict(&self) -> &HashMap<i32, TrackerEvent> {
        &self.dictionary
    }

    pub fn get_dict_entry(&self, index: i32) -> &TrackerEvent {
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

    pub fn get_choice_data(&self) -> &TrackerEvent {
        self.choice_data.as_ref().unwrap().as_ref()
    }

    pub fn get_optional_data(&self) -> &TrackerEvent {
        self.optional_data.as_ref().unwrap().as_ref()
    }

    pub fn get_uint(&self) -> u64 {
        self.unsigned_int.unwrap()
    }

    pub fn get_vint(&self) -> i64 {
        self.variable_int.unwrap()
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
}

fn is_vec_empty(v: &Vec<TrackerEvent>) -> bool {
    v.len() == 0
}

fn is_map_empty(m: &HashMap<i32, TrackerEvent>) -> bool {
    m.len() == 0
}
