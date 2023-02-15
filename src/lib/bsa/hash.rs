use std::{
    fmt,
    io::{BufRead, Result, Write},
};

use crate::{WriteEx, ReadEx};

#[derive(Debug, PartialEq)]
pub struct Hash {
    pub low: u32,
    pub high: u32,
}

impl Hash {
    pub fn from_path(filepath: &str) -> Self {
        let name = filepath.to_ascii_lowercase().replace('/', "\\");
        let bytes = name.as_bytes();
        let len = bytes.len();

        let mid_point = len >> 1;
        let mut low_bytes = [0u8; 4];

        for i in 0..mid_point {
            low_bytes[i & 3] ^= bytes[i];
        }

        let low = u32::from_le_bytes(low_bytes);
        let mut high: u32 = 0;

        for (i, &byte) in bytes.iter().enumerate().take(len).skip(mid_point) {
            let tmp = u32::from(byte) << (((i - mid_point) & 3) << 3);
            high ^= tmp;
            high = high.rotate_right(tmp & 0x1f);
        }

        Self { low, high }
    }

    pub fn read_from<R: BufRead>(r: &mut R) -> Result<Self> {
        Ok(Self {
            low: r.read_u32_le()?,
            high: r.read_u32_le()?,
        })
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_u32_le(self.low)?;
        w.write_u32_le(self.high)
    }
}

impl From<u64> for Hash {
    fn from(val: u64) -> Self {
        Self {
            high: (val >> 32) as u32,
            low: val as u32,
        }
    }
}

impl From<Hash> for u64 {
    fn from(value: Hash) -> u64 {
        (u64::from(value.high) << 32) | u64::from(value.low)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // FIXME: why `u64::from(self)` doesn't work?
        write!(f, "{:08x}{:08x}", self.high, self.low)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn calc_filepath_hash() {
        use super::Hash;

        assert_eq!(
            Hash::from_path("meshes\\m\\probe_journeyman_01.nif"),
            Hash::from(13497295320249402166),
        );
        assert_eq!(
            Hash::from_path("textures\\menu_rightbuttonup_bottom.dds"),
            Hash::from(1799937604540321103),
        );
    }
}
