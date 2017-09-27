use std::io;
use std::collections::HashMap;
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Default)]
pub struct TrackerEvent {
    pub data_type: i32,
    array: Option<Vec<TrackerEvent>>,
    dictionary: Option<HashMap<i32, TrackerEvent>>,
    blob: Option<Vec<u8>>,
    choice_flag: Option<i32>,
    choice_data: Option<Box<TrackerEvent>>,
    optional_data: Option<Box<TrackerEvent>>,
    unsigned_int: Option<u64>,
    variable_int: Option<i64>,
}

impl TrackerEvent {
    pub fn new<R: io::Read>(r: &mut R) -> Result<TrackerEvent, io::Error> {
        let mut event: TrackerEvent = Default::default();

        event.data_type = r.read_u8()? as i32;
        match event.data_type {
            0x00 => {
                let array_len = TrackerEvent::read_variable_int(r)?;
                let mut array: Vec<TrackerEvent> = Vec::new();
                for _ in 0..array_len {
                    let event = TrackerEvent::new(r)?;
                    array.push(event);
                }

                event.array = Some(array);
            },
            0x01 => panic!("unsupported tracker event type"),
            0x02 => {
                let blob_len = TrackerEvent::read_variable_int(r)?;
                let mut buf: Vec<u8> = vec![0; blob_len as usize];
                r.read_exact(&mut buf)?;

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

                event.dictionary = Some(dictionary);
            },
            0x06 => {
                event.unsigned_int = Some(r.read_u8()? as u64);
            },
            0x07 => {
                event.unsigned_int = Some(r.read_u32::<LittleEndian>()? as u64);
            },
            0x08 => {
                event.unsigned_int = Some(r.read_u64::<LittleEndian>()?);
            },
            0x09 => {
                event.variable_int = Some(TrackerEvent::read_variable_int(r)?);
            },
            _ => panic!("unsupported tracker event type")
        }

        Ok(event)
    }

    pub fn get_array(&self) -> &Vec<TrackerEvent> {
        self.array.as_ref().unwrap()
    }

    pub fn get_dict(&self) -> &HashMap<i32, TrackerEvent> {
        self.dictionary.as_ref().unwrap()
    }

    pub fn get_dict_entry(&self, index: i32) -> &TrackerEvent {
        self.get_dict().get(&index).unwrap()
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

    fn read_variable_int<R: io::Read>(r: &mut R) -> Result<i64, io::Error> {
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
