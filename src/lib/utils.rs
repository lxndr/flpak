use std::{borrow::Cow, io, str};

use encoding_rs::Encoding;

#[macro_export]
macro_rules! io_error {
    ($kind:ident, $($arg:tt)+) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($($arg)+))
    };
}

pub fn buffer_to_zstring<'a>(
    buf: &'a [u8],
    encoding: &'static Encoding,
) -> io::Result<Cow<'a, str>> {
    let Some(null_byte_position) = buf.iter().position(|&x| x == 0) else {
        return Err(io_error!(UnexpectedEof, "should be a null-terminated string"));
    };

    let (cow, _, had_error) = encoding.decode(&buf[..null_byte_position]);

    if had_error {
        return Err(io_error!(
            InvalidData,
            "should be a correct sequence of characters"
        ));
    }

    Ok(cow)
}

#[cfg(test)]
mod tests {
    mod buffer_to_zstring {
        use encoding_rs::UTF_8;

        use super::super::buffer_to_zstring;
        use std::io::ErrorKind;

        #[test]
        fn correct() {
            let res = buffer_to_zstring(b"text\0\0\0\0\0", UTF_8);
            assert_eq!(res.unwrap(), "text");
        }

        #[test]
        fn not_null_terminated() {
            let res = buffer_to_zstring(b"text", UTF_8);
            assert_eq!(res.unwrap_err().kind(), ErrorKind::UnexpectedEof);
        }

        #[test]
        fn invalid_utf8_sequence() {
            let res = buffer_to_zstring(b"text\xc3\x28\0", UTF_8);
            assert_eq!(res.unwrap_err().kind(), ErrorKind::InvalidData);
        }
    }
}
