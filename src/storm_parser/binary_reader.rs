use std::io::prelude::*;
use std::io::{Read, Cursor, SeekFrom, Error, ErrorKind};

use bitstream_io::{LE, BitReader};

pub struct BinaryReader<'a> {
    inner: BitReader<'a, LE>
}

impl<'a> BinaryReader<'a> {
    pub fn new(cursor: &'a mut Cursor<Vec<u8>>) -> BinaryReader<'a> {
        BinaryReader{ inner: BitReader::new(cursor) }
    }

    pub fn skip_bytes(&mut self, count: u32) -> Result<(), Error> {
        self.skip_bits(count * 8)
    }

    pub fn skip_bits(&mut self, count: u32) -> Result<(), Error> {
        self.inner.skip(count)
    }

    pub fn read_bool(&mut self) -> Result<bool, Error> {
        self.inner.read_bit()
    }

    pub fn read_bit_array(&mut self, bits: u32) -> Result<Vec<bool>, Error> {
        let mut array: Vec<bool> = vec![false; bits as usize];
        for i in 0..bits {
            array[i as usize] = self.inner.read_bit()?;
        }
        Ok(array)
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.inner.read::<u8>(8)
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        self.inner.read::<u16>(16)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.inner.read::<u32>(32)
    }

    pub fn read_u64(&mut self) -> Result<u64, Error> {
        self.inner.read::<u64>(64)
    }

    pub fn read_i16(&mut self) -> Result<i16, Error> {
        self.inner.read::<i16>(16)
    }

    pub fn read_i32(&mut self) -> Result<i32, Error> {
        self.inner.read::<i32>(32)
    }

    pub fn read_i64(&mut self) -> Result<i64, Error> {
        self.inner.read::<i64>(64)
    }

    pub fn read_vint32(&mut self, bits: u32) -> Result<u32, Error> {
        self.inner.read::<u32>(bits)
    }

    pub fn read_vint64(&mut self, bits: u32) -> Result<u64, Error> {
        self.inner.read::<u64>(bits)
    }

    pub fn read_bytes(&mut self, count: u64) -> Result<Vec<u8>, Error> {
        let mut buf: Vec<u8> = vec![0; count as usize];
        self.inner.read_bytes(buf.as_mut_slice())?;

        Ok(buf)
    }

    pub fn read_string(&mut self, len: u32) -> Result<String, Error> {
        let raw = self.read_bytes(len as u64)?;
        String::from_utf8(raw).map_err(|e| Error::new(ErrorKind::Other, "failed to convert string to utf-8"))
    }

    pub fn read_len_prefixed_blob(&mut self, size_bits: u32) -> Result<Vec<u8>, Error> {
        let blob_len = self.read_vint64(size_bits)?;
        self.align();
        self.read_bytes(blob_len)
    }

    pub fn read_len_prefixed_string(&mut self, size_bits: u32) -> Result<String, Error> {
        let blob = self.read_len_prefixed_blob(size_bits)?;
        String::from_utf8(blob).map_err(|e| Error::new(ErrorKind::Other, "failed to parse utf8 string"))
    }

    pub fn align(&mut self) {
        self.inner.byte_align()
    }
}
