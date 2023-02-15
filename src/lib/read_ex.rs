use std::io::{BufRead, Error, ErrorKind, Result};

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
}

impl<R: BufRead + ?Sized> ReadEx for R {}
