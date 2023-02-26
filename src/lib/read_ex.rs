use std::{
    io::{self, BufRead, Result},
    mem, slice,
};

use crate::io_error;

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
    fn read_u32_be(&mut self) -> Result<u32> {
        self.read_u32(true)
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
                return Err(io_error!(
                    InvalidData,
                    "Failed to read null terminated string: {e}",
                ))
            }
        };

        Ok(s)
    }

    /// Reads a sized null-terminated string.
    /// First byte signifies integer length of following string including null.
    fn read_u8_zstring(&mut self) -> Result<String> {
        let len: usize = self.read_u8()?.try_into().expect("should fit into `usize`");

        if len == 0 {
            return Err(io_error!(
                InvalidData,
                "string cannot be 0 length",
            ));
        }

        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;

        String::from_utf8(buf[..len - 1].to_vec())
            .map_err(|err| io_error!(InvalidData, "{}", err))
    }

    /// Reads a sized string.
    /// First byte signifies integer length of following string.
    fn read_u8_string(&mut self) -> Result<String> {
        let len: usize = self.read_u8()?.try_into().expect("should fit into `usize`");

        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;

        String::from_utf8(buf).map_err(|err| io_error!(InvalidData, "{}", err))
    }

    /// Reads a sized string.
    /// First two bytes signify little endian 16 bit integer length of following string.
    fn read_u16le_string(&mut self) -> Result<String> {
        let len: usize = self
            .read_u16_le()?
            .try_into()
            .expect("should fit into `usize`");

        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;

        String::from_utf8(buf).map_err(|err| io_error!(InvalidData, "{}", err))
    }

    fn read_u8_vec(&mut self, count: usize) -> io::Result<Vec<u8>> {
        let mut v = vec![0u8; count];
        self.read_exact(&mut v)?;
        Ok(v)
    }

    fn read_u32_le_vec(&mut self, count: usize) -> io::Result<Vec<u32>> {
        let mut v = Vec::new();

        for _ in 0..count {
            let item = self.read_u32_le()?;
            v.push(item);
        }

        Ok(v)
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

    #[inline]
    fn read_c_struct_vec<T: Sized>(&mut self, count: usize) -> io::Result<Vec<T>> {
        let mut vec = Vec::with_capacity(count);

        for _ in 0..count {
            let val = self.read_c_struct()?;
            vec.push(val);
        }

        Ok(vec)
    }
}

impl<R: BufRead + ?Sized> ReadEx for R {}
