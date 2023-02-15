use std::io::{Result, Write};

const ZSTRING_TERMINATOR: [u8; 1] = [0];

pub trait WriteEx: Write {
    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u32(&mut self, val: u32, big_endian: bool) -> Result<()> {
        let bytes = if big_endian {
            val.to_be_bytes()
        } else {
            val.to_le_bytes()
        };

        self.write_all(&bytes)
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u32_le(&mut self, val: u32) -> Result<()> {
        self.write_u32(val, false)
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u64(&mut self, val: u64, big_endian: bool) -> Result<()> {
        let bytes = if big_endian {
            val.to_be_bytes()
        } else {
            val.to_le_bytes()
        };

        self.write_all(&bytes)
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u64_le(&mut self, val: u64) -> Result<()> {
        self.write_u64(val, false)
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_zstring(&mut self, val: &str) -> Result<()> {
        self.write_all(val.as_bytes())?;
        self.write_all(&ZSTRING_TERMINATOR)?;
        Ok(())
    }
}

impl<R: Write + ?Sized> WriteEx for R {}
