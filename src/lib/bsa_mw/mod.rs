// file format description https://en.uesp.net/wiki/Morrowind_Mod:BSA_File_Format

mod hash;
mod reader;
#[cfg(test)]
mod reader_tests;
mod records;
mod writer;
#[cfg(test)]
mod writer_tests;

pub use reader::make_reader;
pub use writer::create_archive;
