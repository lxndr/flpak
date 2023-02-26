use std::{io, str};

#[macro_export]
macro_rules! io_error {
    ($kind:ident, $($arg:tt)+) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($($arg)+))
    };
}

pub fn buffer_to_zstring(buf: &[u8]) -> io::Result<&str> {
    let Some(null_byte_position) = buf.iter().position(|&x| x == 0) else {
        return Err(io_error!(UnexpectedEof, "should be a null-terminated string"));
    };

    let val = str::from_utf8(&buf[..null_byte_position])
        .map_err(|_| io_error!(InvalidData, "should be a correct sequence of characters",))?;

    Ok(val)
}

pub fn buffer_to_ascii_zstring(buf: &[u8]) -> io::Result<&str> {
    let val = buffer_to_zstring(buf)?;

    if !val.is_ascii() {
        return Err(io_error!(InvalidData, "should be an ascii string",));
    }

    Ok(val)
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
