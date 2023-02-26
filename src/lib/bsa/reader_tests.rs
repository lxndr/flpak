use std::path::Path;

use crate::reader::Options;

use super::make_reader;

#[test]
fn correct_v105() {
    let res = make_reader(
        Path::new("./samples/bsa/correct_v105.bsa"),
        Options { strict: true },
    );

    assert_eq!(res.is_ok(), true);
}

#[test]
fn correct_v103_xbox() {
    let res = make_reader(
        Path::new("./samples/bsa/correct_v103_xbox.bsa"),
        Options { strict: true },
    );

    assert_eq!(res.is_ok(), true);
}

#[test]
fn invalid_signature() {
    let res = make_reader(
        Path::new("./samples/bsa/invalid_signature.bsa"),
        Options { strict: true },
    );

    let err = res.err().unwrap();
    assert_eq!(
        err.to_string(),
        "invalid signature [58, 58, 58, 58], expected [42, 53, 41, 0]"
    );
}

#[test]
fn invalid_header() {
    let res = make_reader(
        Path::new("./samples/bsa/invalid_header.bsa"),
        Options { strict: true },
    );

    let err = res.err().unwrap();
    assert_eq!(
        err.to_string(),
        "failed to read file header: failed to fill whole buffer"
    );
}

#[test]
fn invalid_version() {
    let res = make_reader(
        Path::new("./samples/bsa/invalid_version.bsa"),
        Options { strict: true },
    );

    let err = res.err().unwrap();
    assert_eq!(
        err.to_string(),
        "failed to read file header: invalid version 102"
    );
}

#[test]
fn invalid_flags() {
    let res = make_reader(
        Path::new("./samples/bsa/invalid_flags.bsa"),
        Options { strict: true },
    );

    let err = res.err().unwrap();
    assert_eq!(err.to_string(), "failed to read file header: invalid flags");
}

#[test]
fn invalid_file_flags() {
    let res = make_reader(
        Path::new("./samples/bsa/invalid_file_flags.bsa"),
        Options { strict: true },
    );

    let err = res.err().unwrap();
    assert_eq!(
        err.to_string(),
        "failed to read file header: invalid file flags"
    );
}
