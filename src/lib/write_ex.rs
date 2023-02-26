use std::io::{Result, Write};

const ZSTRING_TERMINATOR: [u8; 1] = [0];

pub trait WriteEx: Write {
    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write_all(&[val])
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u16(&mut self, val: u16, big_endian: bool) -> Result<()> {
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
    fn write_u16_le(&mut self, val: u16) -> Result<()> {
        self.write_u16(val, false)
    }

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
    fn write_u32_be(&mut self, val: u32) -> Result<()> {
        self.write_u32(val, true)
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

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u8_string(&mut self, val: &str) -> Result<()> {
        self.write_u8(val.len() as u8)?;
        self.write_all(val.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn write_u8_zstring(&mut self, val: &str) -> Result<()> {
        let len = u8::try_from(val.len() + 1).expect("string is too long");
        self.write_u8(len)?;
        self.write_all(val.as_bytes())?;
        self.write_all(&ZSTRING_TERMINATOR)?;
        Ok(())
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u32_le_vec(&mut self, vec: &[u32]) -> Result<()> {
        for &val in vec {
            self.write_u32_le(val)?;
        }

        Ok(())
    }

    /// # Errors
    /// Will return same errors as [`Write::write`] does.
    #[inline]
    fn write_u64_le_vec(&mut self, vec: &[u64]) -> Result<()> {
        for &val in vec {
            self.write_u64_le(val)?;
        }

        Ok(())
    }
}

impl<R: Write + ?Sized> WriteEx for R {}
