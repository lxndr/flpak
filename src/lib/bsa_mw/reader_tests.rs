use crate::FileType;
use std::path::Path;

#[test]
fn new_reader() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/correct.bsa"),
        crate::reader::Options { strict: true },
    );

    let rdr = res.ok().unwrap();

    assert_eq!(rdr.len(), 6);

    let file = rdr.get_file(0);
    assert_eq!(file.file_type, FileType::RegularFile);
    assert_eq!(file.name, "dir1/img002.jpg");
    assert_eq!(file.size.unwrap(), 11590);

    let file = rdr.get_file(1);
    assert_eq!(file.file_type, FileType::RegularFile);
    assert_eq!(file.name, "dir1/file002.txt");
    assert_eq!(file.size.unwrap(), 0);

    let file = rdr.get_file(2);
    assert_eq!(file.file_type, FileType::RegularFile);
    assert_eq!(file.name, "img001.png");
    assert_eq!(file.size.unwrap(), 22290);
}

#[test]
fn failed_to_open() {
    let res = super::make_reader(
        Path::new("./samples/bsa/none.bsa"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.err().unwrap();
    assert_eq!(
        rdr.to_string(),
        "failed to open file: No such file or directory (os error 2)"
    );
}

#[test]
fn invalid_header() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/invalid_header.bsa"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.err().unwrap();
    assert_eq!(
        rdr.to_string(),
        "failed to read file's header: failed to fill whole buffer"
    );
}

#[test]
fn invalid_signature() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/invalid_signature.bsa"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.err().unwrap();
    assert_eq!(
        rdr.to_string(),
        "invalid signature [1, 2, 3, 4], expected [0, 1, 0, 0]"
    );
}

#[test]
fn invalid_file_records() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/invalid_file_records.bsa"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.err().unwrap();
    assert_eq!(
        rdr.to_string(),
        "failed to read file record: failed to fill whole buffer"
    );
}

/*
#[test]
fn invalid_file_names() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/invalid_file_names.bsa"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.err().unwrap();
    assert_eq!(rdr.to_string(), "failed to read file's header");
}

#[test]
fn invalid_hash_records() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/invalid_hash_records.bsa"),
        crate::reader::Options { strict: true },
    );

    let rdr = res.err().unwrap();
    assert_eq!(rdr.to_string(), "failed to read file's header");
}

#[test]
fn invalid_hashes() {
    let res = super::make_reader(
        Path::new("./samples/bsa-mw/invalid_hashes.bsa"),
        crate::reader::Options { strict: true },
    );

    let rdr = res.err().unwrap();
    assert_eq!(rdr.to_string(), "failed to read file's header");
}
*/
