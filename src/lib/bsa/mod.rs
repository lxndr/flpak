// https://en.uesp.net/wiki/Skyrim_Mod:Archive_File_Format

mod hash;
#[cfg(test)]
mod hash_tests;
mod header;
mod read_file_index;
mod reader;
#[cfg(test)]
mod reader_tests;
mod version;
mod write_file_index;
mod writer;
#[cfg(test)]
mod writer_tests;

use hash::*;
use header::*;
use read_file_index::*;
use version::*;
use write_file_index::*;

pub use reader::make_reader;
pub use writer::create_archive;
