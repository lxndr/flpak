use std::io::{Error, Read, Write};

pub trait Decompressor {
    fn decompress<R: Read, W: Write>(&self, input: &mut R, output: &mut W) -> Result<(), Error>;
}

pub struct NoCompression;

impl Decompressor for NoCompression {
    fn decompress<R: Read, W: Write>(&self, input: &mut R, output: &mut W) -> Result<(), Error> {
        std::io::copy(input, output)?;
        Ok(())
    }
}

pub struct LZ4Decompressor;

impl Decompressor for LZ4Decompressor {
    fn decompress<R: Read, W: Write>(&self, input: &mut R, output: &mut W) -> Result<(), Error> {
        let mut reader = lz4_flex::frame::FrameDecoder::new(input);
        std::io::copy(&mut reader, output)?;
        Ok(())
    }
}
