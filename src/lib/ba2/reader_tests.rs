use std::path::Path;

#[test]
fn correct_general_archive() {
    let registry = super::make_reader(
        Path::new("./samples/bs2/correct_general.ba2"),
        crate::reader::Options { strict: true },
    );

    assert_eq!(registry.is_ok(), true);
}
