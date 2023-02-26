use std::{
    fs,
    io::{self, Read, Seek},
};

use crate::{io_error, utils::buffer_to_ascii_zstring, ReadEx, WriteEx};

pub const BSA_SIGNATURE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
pub const BSA_HEADER_SIZE: u64 = 12;

#[repr(C, packed)]
pub struct Header {
    pub signature: [u8; 4],
    pub hash_table_offset: u32,
    pub file_count: u32,
}

impl Header {
    pub fn read(r: &mut impl io::BufRead) -> io::Result<Header> {
        let mut hdr = r.read_c_struct::<Header>()?;

        if cfg!(target_endian = "big") {
            hdr.hash_table_offset = u32::from_le(hdr.hash_table_offset);
            hdr.file_count = u32::from_le(hdr.file_count);
        }

        Ok(hdr)
    }

    pub fn write(
        w: &mut impl io::Write,
        hash_table_offset: u32,
        file_count: usize,
    ) -> io::Result<()> {
        let file_count: u32 = file_count.try_into().expect("should fit into `u32`");
        w.write_all(&BSA_SIGNATURE)?;
        w.write_u32_le(hash_table_offset)?;
        w.write_u32_le(file_count)?;
        Ok(())
    }

    pub fn absolute_hash_table_offset(&self) -> u64 {
        u64::from(self.hash_table_offset) + BSA_HEADER_SIZE
    }
}

#[repr(C, packed)]
pub struct FileRecord {
    pub size: u32,
    pub offset: u32,
}

impl FileRecord {
    pub fn write(r: &mut impl io::Write, size: u32, offset: u32) -> io::Result<()> {
        r.write_u32_le(size)?;
        r.write_u32_le(offset)?;
        Ok(())
    }
}

pub fn read_file_index(r: &mut impl io::BufRead, count: usize) -> io::Result<Vec<FileRecord>> {
    let mut index = r.read_c_struct_vec::<FileRecord>(count)?;

    if cfg!(target_endian = "big") {
        for rec in index.iter_mut() {
            rec.size = u32::from_le(rec.size);
            rec.offset = u32::from_le(rec.offset);
        }
    }

    Ok(index)
}

pub fn read_file_names(
    r: &mut io::BufReader<fs::File>,
    hdr: &Header,
    count: usize,
) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    let name_offsets = r.read_u32_le_vec(count)?;

    let rdr_pos = r.stream_position()?;
    let names_block_len: usize = (hdr.absolute_hash_table_offset() - rdr_pos)
        .try_into()
        .expect("should fit into `usize`");
    let mut names_buf = vec![0; names_block_len];
    r.read_exact(&mut names_buf)?;

    for offset in name_offsets {
        let name = buffer_to_ascii_zstring(&names_buf[offset as usize..])
            .map_err(|err| io_error!(InvalidData, "invalid file name: {err}",))?;

        names.push(name.replace('\\', "/"));
    }

    Ok(names)
}
