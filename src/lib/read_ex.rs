use std::{
    io::{self, BufRead, Error, ErrorKind, Result},
    mem, slice,
};

pub trait ReadEx: BufRead {
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_u16(&mut self, big_endian: bool) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;

        if big_endian {
            Ok(u16::from_be_bytes(buf))
        } else {
            Ok(u16::from_le_bytes(buf))
        }
    }

    #[inline]
    fn read_u16_le(&mut self) -> Result<u16> {
        self.read_u16(false)
    }

    #[inline]
    fn read_u32(&mut self, big_endian: bool) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;

        if big_endian {
            Ok(u32::from_be_bytes(buf))
        } else {
            Ok(u32::from_le_bytes(buf))
        }
    }

    #[inline]
    fn read_u32_le(&mut self) -> Result<u32> {
        self.read_u32(false)
    }

    #[inline]
    fn read_u64(&mut self, big_endian: bool) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;

        if big_endian {
            Ok(u64::from_be_bytes(buf))
        } else {
            Ok(u64::from_le_bytes(buf))
        }
    }

    #[inline]
    fn read_u64_le(&mut self) -> Result<u64> {
        self.read_u64(false)
    }

    fn read_zstring(&mut self) -> Result<String> {
        let mut buf = Vec::new();
        self.read_until(0, &mut buf)?;
        buf.pop();

        let s = match String::from_utf8(buf) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to read null terminated string: {e}"),
                ))
            }
        };

        Ok(s)
    }

    /// Reads a sized string.
    /// First two bytes signify little endian 16 bit integer length of following string.
    fn read_u16le_string(&mut self) -> Result<String> {
        let len: usize = self
            .read_u16_le()?
            .try_into()
            .expect("`u16` should fit into `usize`");

        let mut name_buf = vec![0u8; len];
        self.read_exact(&mut name_buf)?;

        String::from_utf8(name_buf)
            .map_err(|err| Error::new(ErrorKind::InvalidData, err.to_string()))
    }

    #[inline]
    fn read_c_struct<T: Sized>(&mut self) -> io::Result<T> {
        Ok(unsafe {
            let mut ptr: T = mem::zeroed();
            let raw_size = mem::size_of::<T>();
            let raw_slice = slice::from_raw_parts_mut(&mut ptr as *mut _ as *mut u8, raw_size);
            self.read_exact(raw_slice)?;
            ptr
        })
    }
}

impl<R: BufRead + ?Sized> ReadEx for R {}
