use num_enum::TryFromPrimitive;
use std::{
    io::{self, Read},
    mem, slice,
};

pub const LOCAL_FILE_HEADER_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
pub const DATA_DESCRIPTOR_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x07, 0x08];
pub const CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
pub const END_OF_CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
pub const ZIP64_END_OF_CENTRAL_DIRECTORY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];

enum Signature {
    LocalFileHeader = 0x04034b50,
    DataDescriptor,
    CentralDirectory,
    EndOfVentralDirectory,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u16)]
pub enum CompressionMethod {
    NONE = 0,
    DEFLATE = 8,
}

#[repr(C, packed)]
pub struct NativeLocalFileHeader {
    version: u16,
    flags: u16,
    compression_method: u16,
    mod_time: u16,
    mod_date: u16,
    crc: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    filename_length: u16,
    extra_length: u16,
}

pub struct LocalFileHeader {
    pub version: u16,
    pub flags: u16,
    pub compression_method: CompressionMethod,
    pub mod_time: u16,
    pub mod_date: u16,
    pub crc: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub filename: String,
    pub extra: Vec<u8>,
}

pub trait StructDeserializer<R: io::Read, S: Sized> {
    fn deserialize(r: &mut R) -> io::Result<S>;
}

impl<R: io::Read> StructDeserializer<R, LocalFileHeader> for LocalFileHeader {
    fn deserialize(r: &mut R) -> io::Result<LocalFileHeader> {
        let h = unsafe {
            let mut p: NativeLocalFileHeader = mem::zeroed();
            let raw_size = mem::size_of::<NativeLocalFileHeader>();
            let raw_slice = slice::from_raw_parts_mut(&mut p as *mut _ as *mut u8, raw_size);
            r.read_exact(raw_slice)?;
            p
        };

        let compression_method_int = u16::from_le(h.compression_method);
        let compression_method =
            CompressionMethod::try_from(compression_method_int).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "unknown compression method {compression_method_int}: {}",
                        err
                    ),
                )
            })?;

        let filename_length = u16::from_le(h.filename_length);
        let extra_field_length =
            usize::try_from(u16::from_le(h.extra_length)).expect("`u16` should fit into `usize`");

        let mut filename = String::new();
        r.take(u64::from(filename_length))
            .read_to_string(&mut filename)?;

        let mut extra_field = vec![0u8; extra_field_length];
        r.read_exact(&mut extra_field)?;

        Ok(LocalFileHeader {
            version: u16::from_le(h.version),
            flags: u16::from_le(h.flags),
            compression_method,
            mod_time: u16::from_le(h.mod_time),
            mod_date: u16::from_le(h.mod_date),
            crc: u32::from_le(h.crc),
            compressed_size: u32::from_le(h.compressed_size),
            uncompressed_size: u32::from_le(h.uncompressed_size),
            filename,
            extra: extra_field,
        })
    }
}

#[repr(C, packed)]
struct NativeCentralDirectoryFileHeader {
    version_made_by: u16,
    version_needed: u16,
    flags: u16,
    compression_method: u16,
    mod_time: u16,
    mod_date: u16,
    crc: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_length: u16,
    comment_length: u16,
    disk_number: u16,
    internal_attributes: u16,
    external_attributes: u32,
}
