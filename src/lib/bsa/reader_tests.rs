use std::path::Path;

#[test]
fn new_reader() {
    let registry = super::make_reader(
        Path::new("./samples/bsa/correct_v105.bsa"),
        crate::reader::Options { strict: true },
    );

    assert_eq!(registry.is_ok(), true);
}
