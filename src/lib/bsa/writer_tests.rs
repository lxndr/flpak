use std::{collections::HashMap, fs, io, path::Path};

use hex_literal::hex;
use sha1::{Digest, Sha1};
use tempdir::TempDir;

use crate::InputFileListBuilder;

#[test]
fn correct() {
    let input_files = InputFileListBuilder::new()
        .add_dir(Path::new("./samples/unpacked"))
        .unwrap()
        .exclude_pattern("empty_dir/.gitkeep")
        .exclude_pattern("empty_file")
        .exclude_pattern("file001.txt")
        .exclude_pattern("img001.png")
        .build();

    let dir = TempDir::new("flpak-tests").unwrap();
    let output_path = dir.path().join("archive.bsa");
    let res = super::create_archive(input_files, &output_path, HashMap::new());
    assert!(res.is_ok());

    let mut file = fs::File::open(&output_path).unwrap();
    let mut hasher = Sha1::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();

    assert_eq!(hash[..], hex!("2b2b844d0b0267056796139e15b3ccf04d71fda4"));
}
