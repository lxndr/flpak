use std::{collections::HashMap, fs, io, path::Path};

use rstest::*;
use hex_literal::hex;
use sha1::{Digest, Sha1};
use tempdir::TempDir;

use crate::InputFileListBuilder;

#[rstest]
#[case("103", hex!("1c9ab5419e6494ed374e8911a9c25bbed9d5bcda"))]
#[case("104", hex!("2e35408e80e008d0f9984c9b18ba770569719424"))]
#[case("105", hex!("2b2b844d0b0267056796139e15b3ccf04d71fda4"))]
fn correct(#[case] version: &str, #[case] sha1: [u8; 20]) {
    let input_files = InputFileListBuilder::new()
        .add_dir(Path::new("./samples/unpacked"))
        .unwrap()
        .exclude_pattern("empty_dir/.gitkeep")
        .exclude_pattern("empty_file")
        .exclude_pattern("file001.txt")
        .exclude_pattern("img001.png")
        .build();

    let mut params = HashMap::new();
    params.insert(String::from("version"), version.to_string());

    let dir = TempDir::new("flpak-tests").unwrap();
    let output_path = dir.path().join("archive.bsa");
    let res = super::create_archive(input_files, &output_path, &params);
    assert!(res.is_ok());

    let mut file = fs::File::open(&output_path).unwrap();
    let mut hasher = Sha1::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();

    assert_eq!(hash[..], sha1);
}
