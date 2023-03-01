use std::path::Path;

use rstest::rstest;

use crate::reader::Options;

use super::make_reader;

#[rstest]
#[case("103", false, false)]
#[case("103", true, false)]
#[case("103", true, true)]
#[case("104", false, false)]
#[case("104", true, false)]
#[case("105", false, false)]
#[case("105", true, false)]
fn correct(#[case] version: &str, #[case] compress: bool, #[case] embed_names: bool) {
    let compress = if compress { "_comp" } else { "" };
    let embed_names = if embed_names { "_names" } else { "" };

    let res = make_reader(
        Path::new(&format!("./samples/bsa/correct_v{version}{compress}{embed_names}.bsa")),
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
