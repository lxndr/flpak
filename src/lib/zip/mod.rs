// https://en.wikipedia.org/wiki/ZIP_(file_format)

mod reader;
#[cfg(test)]
mod reader_tests;
mod writer;
#[cfg(test)]
mod writer_tests;

pub use reader::make_reader;
pub use writer::create_archive;
