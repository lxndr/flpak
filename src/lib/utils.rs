use std::{fs, io, path::Path, str};

use crate::reader;

pub fn buffer_to_zstring(buf: &[u8]) -> io::Result<&str> {
    let Some(null_byte_position) = buf.iter().position(|&x| x == 0) else {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "should be a null-terminated string"));
    };

    let val = str::from_utf8(&buf[..null_byte_position]).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "should be a correct sequence of characters",
        )
    })?;

    Ok(val)
}

pub fn buffer_to_ascii_zstring(buf: &[u8]) -> io::Result<&str> {
    let val = buffer_to_zstring(buf)?;

    if !val.is_ascii() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "should be an ascii string",
        ));
    }

    Ok(val)
}

pub fn open_file(path: &Path) -> reader::Result<(io::BufReader<fs::File>, fs::Metadata)> {
    let file = fs::File::open(path).map_err(reader::Error::OpeningInputFile)?;
    let metadata = file
        .metadata()
        .map_err(reader::Error::ReadingInputFileMetadata)?;
    let reader = io::BufReader::new(file);
    Ok((reader, metadata))
}

#[cfg(test)]
mod tests {
    mod buffer_to_zstring {
        use super::super::buffer_to_zstring;
        use std::io::ErrorKind;

        #[test]
        fn correct() {
            let res = buffer_to_zstring(b"text\0\0\0\0\0");
            assert_eq!(res.unwrap(), "text");
        }

        #[test]
        fn not_null_terminated() {
            let res = buffer_to_zstring(b"text");
            assert_eq!(res.unwrap_err().kind(), ErrorKind::UnexpectedEof,);
        }

        #[test]
        fn invalid_utf8_sequence() {
            let res = buffer_to_zstring(b"text\xc3\x28\0");
            assert_eq!(res.unwrap_err().kind(), ErrorKind::InvalidData,);
        }
    }

    mod buffer_to_ascii_zstring {
        use super::super::buffer_to_ascii_zstring;
        use std::io::ErrorKind;

        #[test]
        fn invalid_ascii() {
            let res = buffer_to_ascii_zstring(b"text\xc3\xb1\0");
            assert_eq!(res.unwrap_err().kind(), ErrorKind::InvalidData,);
        }
    }
}
