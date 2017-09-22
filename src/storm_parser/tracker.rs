use std::io;
use std::collections::HashMap;
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Default)]
pub struct TrackerEvent {
    pub data_type: i32,
    pub array: Option<Vec<TrackerEvent>>,
    pub dictionary: Option<HashMap<i32, TrackerEvent>>,
    pub blob: Option<Vec<u8>>,
    pub choice_flag: Option<i32>,
    pub choice_data: Option<Box<TrackerEvent>>,
    pub optional_data: Option<Box<TrackerEvent>>,
    pub unsigned_int: Option<u64>,
    pub variable_int: Option<i64>,
}

impl TrackerEvent {
    pub fn new<R: io::Read>(r: &mut R) -> Result<TrackerEvent, io::Error> {
        let mut event: TrackerEvent = Default::default();

        event.data_type = r.read_u8()? as i32;
        match event.data_type {
            0x00 => {
                println!("Reading in array object...");
                let array_len = TrackerEvent::read_variable_int(r)?;
                let mut array: Vec<TrackerEvent> = Vec::new();
                for i in 0..array_len {
                    let event = TrackerEvent::new(r)?;
                    array.push(event);
                }

                event.array = Some(array);
            },
            0x01 => panic!("unsupported tracker event type"),
            0x02 => {
                println!("Reading in blob object...");
                // blob, read N bytes, where N is variable int
                let blob_len = TrackerEvent::read_variable_int(r)?;
                let mut buf: Vec<u8> = vec![0; blob_len as usize];
                r.read_exact(&mut buf)?;

                event.blob = Some(buf);
            },
            0x03 => {
                println!("Reading in choice object...");
                // choice, read choice flag as variable int, and choice data
                // as a tracking event
            },
            0x04 => {
                println!("Reading in optional object...");
                // optional, read byte, and if not 0, read optional data as
                // a tracking event
            },
            0x05 => {
                println!("Reading in dictionary object...");
                // dictionary, read size as variable int, and for N, read key
                // as variable int and then the value as a tracking event
                let mut dictionary: HashMap<i32, TrackerEvent> = HashMap::new();
                let dictionary_len = TrackerEvent::read_variable_int(r)?;
                println!("Dictionary should have {} entries...", dictionary_len);
                for i in 0..dictionary_len {
                    println!("Reading in dictionary entry #{}...", i);
                    let key = TrackerEvent::read_variable_int(r)? as i32;
                    let value = TrackerEvent::new(r)?;

                    dictionary.insert(key, value);
                }

                event.dictionary = Some(dictionary);
            },
            0x06 => {
                println!("Reading in u8 object...");
                event.unsigned_int = Some(r.read_u8()? as u64);
            },
            0x07 => {
                println!("Reading in u32 object...");
                event.unsigned_int = Some(r.read_u32::<LittleEndian>()? as u64);
            },
            0x08 => {
                println!("Reading in u64 object...");
                event.unsigned_int = Some(r.read_u64::<LittleEndian>()?);
            },
            0x09 => {
                println!("Reading in variable integer object...");
                event.variable_int = Some(TrackerEvent::read_variable_int(r)?);
            },
            _ => panic!("unsupported tracker event type")
        }

        Ok(event)
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
