// https://wiki.nexusmods.com/index.php/Bethesda_mod_archives

// mod dds;
mod reader;
//#[cfg(test)]
//mod reader_tests;
mod records;

pub use reader::make_reader;
