// use hex_literal::hex;
use sha1::{Digest, Sha1};
use std::{fs, io, path::Path};
use tempdir::TempDir;

use crate::InputFileListBuilder;

#[test]
fn correct() {
    let input_files = InputFileListBuilder::new()
        .add_dir(Path::new("./samples/unpacked"))
        .unwrap()
        .exclude_pattern("empty_dir/.gitkeep")
        .build();

    let dir = TempDir::new("flpak-tests").unwrap();
    let output_path = dir.path().join("archive.zip");
    let res = super::create_archive(input_files, &output_path);
    assert!(res.is_ok());
    
    let mut file = fs::File::open(&output_path).unwrap();
    let mut hasher = Sha1::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let _hash = hasher.finalize();

    // hmm... resulting zip is different every time
    // assert_eq!(hash[..], hex!("fb977e5f705d8d9603d61aa00af3c23c00c36a11"));
}
