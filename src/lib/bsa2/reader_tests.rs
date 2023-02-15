use std::path::Path;

#[test]
fn new_reader() {
    let registry = super::make_reader(
        Path::new("./samples/tes5/Dragonborn.bsa"),
        crate::reader::Options { strict: false },
    );

    assert_eq!(registry.is_ok(), true);
}
