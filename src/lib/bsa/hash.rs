use std::{fmt, io, path::PathBuf};

use encoding_rs::WINDOWS_1252;

use crate::{PathBufUtils, ReadEx, WriteEx};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hash(u64);

impl Hash {
    pub fn from_file_name(fname: &str) -> Hash {
        assert_ne!(fname.len(), 0);
        let (fname, _, _) = WINDOWS_1252.encode(fname);

        let (name, ext) = match fname.iter().rposition(|&b| b == b'.') {
            Some(pos) => (&fname[..pos], &fname[pos..]),
            None => (&fname[..], &b""[..]),
        };

        let name_len = name.len();

        let mut hash1 = Self::calc_name_hash(&name);
        hash1 |= Self::calc_ext_hash(&ext);

        let hash2: u64 = if name_len > 3 {
            Self::calc_slice_hash(&name[1..name_len - 2]).wrapping_add(Self::calc_slice_hash(&ext))
        } else {
            Self::calc_slice_hash(&ext)
        };

        Hash((hash2 << 32) + u64::from(hash1))
    }

    pub fn from_folder_path(name: &PathBuf) -> Hash {
        let name = name.try_to_win().expect("should be a valid ascii path");
        assert_ne!(name.len(), 0);

        let (name, _, _) = WINDOWS_1252.encode(&name);
        let name_len = name.len();

        let hash1 = Self::calc_name_hash(&name);
        let hash2: u64 = if name_len > 3 {
            Self::calc_slice_hash(&name[1..name_len - 2])
        } else {
            0
        };

        Hash((hash2 << 32) + u64::from(hash1))
    }

    fn calc_name_hash(name: &[u8]) -> u32 {
        let len = name.len();

        let mut hash: u32 = u32::from(name[len - 1]);
        hash |= u32::from(if len < 3 { 0 } else { name[len - 2] }) << 8;
        hash |= (len as u32) << 16;
        hash |= u32::from(name[0]) << 24;

        hash
    }

    fn calc_ext_hash(ext: &[u8]) -> u32 {
        match ext {
            b".kf" => 0x80,
            b".nif" => 0x8000,
            b".dds" => 0x8080,
            b".wav" => 0x80000000,
            _ => 0,
        }
    }

    fn calc_slice_hash(bytes: &[u8]) -> u64 {
        let mut hash: u64 = 0;

        for &byte in bytes {
            hash = hash.wrapping_mul(0x1003f).wrapping_add(u64::from(byte));
        }

        hash
    }
}

impl From<u64> for Hash {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Hash> for u64 {
    fn from(val: Hash) -> Self {
        val.0
    }
}

impl fmt::LowerHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

pub trait ReadHash: io::BufRead {
    fn read_hash(&mut self, is_xbox: bool) -> io::Result<Hash> {
        if is_xbox {
            let low = self.read_u32_le()?;
            let high = self.read_u32_be()?;
            return Ok(Hash(u64::from(high) << 32 | u64::from(low)));
        } else {
            Ok(Hash(self.read_u64_le()?))
        }
    }
}

impl<R: io::BufRead + ?Sized> ReadHash for R {}

pub trait WriteHash: io::Write {
    fn write_hash(&mut self, hash: &Hash, is_xbox: bool) -> io::Result<()> {
        if is_xbox {
            let low = hash.0.try_into().expect("low u32");
            let high = (hash.0 >> 32).try_into().expect("high u32");
            self.write_u32_le(low)?;
            self.write_u32_be(high)?;
        } else {
            self.write_u64_le(hash.0)?;
        }

        Ok(())
    }
}

impl<R: io::Write + ?Sized> WriteHash for R {}
