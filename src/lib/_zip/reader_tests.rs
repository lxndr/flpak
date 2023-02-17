#[cfg(test)]
use std::path::Path;

#[test]
fn new_reader() {
    let res = super::make_reader(
        Path::new("./samples/zip/correct.zip"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.ok().unwrap();
    assert_eq!(rdr.len(), 5);
}
