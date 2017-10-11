use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};

pub struct BinaryReader<'a> {
    inner: Cursor<&'a Vec<u8>>,
    pos: u32,
    val: u8,
}

impl<'a> BinaryReader<'a> {
    pub fn new(buf: &'a Vec<u8>) -> BinaryReader<'a> {
        BinaryReader{ inner: Cursor::new(buf), pos: 0, val: 0 }
    }

    pub fn position(&mut self) -> u64 {
        self.inner.position()
    }

    pub fn read(&mut self, mut bits: u32) -> Result<u64, Error> {
        let mut value: u64 = 0;

        while bits > 0 {
            let val_pos = self.pos & 7;
            let val_rem_bits = 8 - val_pos;
            if val_pos == 0 {
                let mut val_buf = [0u8; 1];
                self.inner.read_exact(&mut val_buf)?;
                self.val = val_buf[0];
            }

            let read_bits = if val_rem_bits > bits { bits } else { val_rem_bits };
            let shifted_val = self.val as u32 >> val_pos;
            let read_mask = (1u32 << read_bits) - 1;
            value <<= read_bits;
            value |= (shifted_val & read_mask) as u64;
            self.pos += read_bits;
            bits -= read_bits;
        }

        Ok(value)
    }

    pub fn is_aligned(&self) -> bool {
        (self.pos % 8) == 0
    }

    pub fn align(&mut self) {
        if (self.pos % 8) > 0 {
            self.pos = (self.pos & 0x7ffffff8) + 8;
        }
    }

    pub fn skip_bytes(&mut self, count: u64) -> Result<u64, Error> {
        self.inner.seek(SeekFrom::Current(count as i64))
    }

    pub fn read_bool(&mut self) -> Result<bool, Error> {
        self.read(1).map(|x| if x > 0 { true } else { false })
    }

    pub fn read_bit_array(&mut self, bits: u32) -> Result<Vec<bool>, Error> {
        let mut array: Vec<bool> = vec![false; bits as usize];
        for i in 0..bits {
            array[i as usize] = self.read_bool()?;
        }
        Ok(array)
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        self.read(8).map(|x| x as u8)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.read(32).map(|x| x as u32)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, Error> {
        self.read(32).map(|x| u32::from_be(x as u32))
    }

    pub fn read_u64_le(&mut self) -> Result<u64, Error> {
        self.read(64).map(|x| u64::from_be(x as u64))
    }

    pub fn read_vu32(&mut self, bits: u32) -> Result<u32, Error> {
        self.read(bits).map(|x| x as u32)
    }

    pub fn read_i32(&mut self) -> Result<i32, Error> {
        self.read(32).map(|x| x as u32).map(|x| x as i32)
    }

    pub fn read_bytes(&mut self, count: u32) -> Result<Vec<u8>, Error> {
        let mut buf: Vec<u8> = vec![0; count as usize];
        self.read_bytes_direct(buf.as_mut_slice())?;
        Ok(buf)
    }

    pub fn read_bytes_direct(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let buf_len = buf.len();
        if self.is_aligned() {
            self.inner.read_exact(buf)?;
        } else {
            for i in 0..buf_len {
                buf[i] = self.read_u8()?;
            }
        }

        Ok(())
    }

    pub fn read_string(&mut self, len: u32) -> Result<String, Error> {
        let raw = self.read_bytes(len)?;
        String::from_utf8(raw).map_err(|_| Error::new(ErrorKind::Other, "failed to convert string to utf-8"))
    }

    pub fn read_len_prefixed_blob(&mut self, size_bits: u32) -> Result<Vec<u8>, Error> {
        let blob_len = self.read_vu32(size_bits)?;
        self.align();
        self.read_bytes(blob_len)
    }

    pub fn read_len_prefixed_string(&mut self, size_bits: u32) -> Result<String, Error> {
        let blob = self.read_len_prefixed_blob(size_bits)?;
        String::from_utf8(blob).map_err(|_| Error::new(ErrorKind::Other, "failed to parse utf8 string"))
    }
}
