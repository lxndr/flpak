#[cfg(test)]
use std::path::Path;

#[test]
fn new_reader() {
    let res = super::make_reader(
        Path::new("./samples/rpa/correct.rpa"),
        crate::reader::Options { strict: false },
    );

    let rdr = res.ok().unwrap();

    assert_eq!(rdr.file_count(), 6);
}
