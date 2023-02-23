use super::Hash;

#[test]
fn calc_file_name_hash() {
    assert_eq!(
        Hash::from_file_name("dlcanchfrelevmachplatf01.nif"),
        Hash::from(0x2E8318AE6418B031),
    );
    assert_eq!(
        Hash::from_file_name("go.nif"),
        Hash::from(0x92CD45FD6702806F),
    );
}

#[test]
fn calc_folder_name_hash() {
    assert_eq!(
        Hash::from_folder_path(&String::from(
            "sound\\voice\\hearthfires.esm\\femaleelfhaughty"
        )),
        Hash::from(0x00400744732C7479),
    );
    assert_eq!(
        Hash::from_folder_path(&String::from("x")),
        Hash::from(0x0000000078010078),
    );
    assert_eq!(
        Hash::from_folder_path(&String::from("xx")),
        Hash::from(0x0000000078020078),
    );
    assert_eq!(
        Hash::from_folder_path(&String::from("xxx")),
        Hash::from(0x0000000078037878),
    );
}
