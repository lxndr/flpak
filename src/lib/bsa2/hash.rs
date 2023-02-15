pub fn calc_file_name_hash(name: &str) -> u64 {
    let filename = normalize_path(name);

    let (name, ext) = match filename.rfind('.') {
        Some(pos) => (&filename[..pos], &filename[pos..]),
        None => (filename.as_str(), ""),
    };

    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    let ext_bytes = ext.as_bytes();

    let mut hash1 = calc_name_hash(name);
    hash1 |= calc_ext_hash(ext);

    let hash2: u64 = if name_len > 3 {
        calc_slice_hash(&name_bytes[1..name_len - 2]).wrapping_add(calc_slice_hash(ext_bytes))
    } else {
        calc_slice_hash(ext_bytes)
    };

    (hash2 << 32) + (hash1 as u64)
}

pub fn calc_folder_name_hash(name: &str) -> u64 {
    let name = normalize_path(name);
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();

    let hash1 = calc_name_hash(&name);
    let hash2: u64 = if name_len > 3 {
        calc_slice_hash(&name_bytes[1..name_len - 2])
    } else {
        0
    };

    (hash2 << 32) + u64::from(hash1)
}

fn normalize_path(path: &str) -> String {
    assert!(!path.is_empty(), "Path cannot be empty");
    path.to_ascii_lowercase().replace('/', "\\")
}

fn calc_name_hash(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let len = bytes.len();

    let mut hash: u32 = u32::from(bytes[len - 1]);
    hash |= u32::from(if len < 3 { 0 } else { bytes[len - 2] }) << 8;
    hash |= (len as u32) << 16;
    hash |= u32::from(bytes[0]) << 24;

    hash
}

fn calc_ext_hash(ext: &str) -> u32 {
    match ext {
        ".kf" => 0x80,
        ".nif" => 0x8000,
        ".dds" => 0x8080,
        ".wav" => 0x80000000,
        _ => 0,
    }
}

fn calc_slice_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0;

    for &byte in bytes {
        hash = hash.wrapping_mul(0x1003f).wrapping_add(u64::from(byte));
    }

    hash
}

#[cfg(test)]
mod tests {
    #[test]
    fn calc_file_name_hash() {
        assert_eq!(
            super::calc_file_name_hash("dlcanchfrelevmachplatf01.nif"),
            3351549684976496689,
        );
        assert_eq!(super::calc_file_name_hash("go.nif"), 10578188054420422767,);
    }

    #[test]
    fn calc_folder_name_hash() {
        assert_eq!(
            super::calc_folder_name_hash(&String::from(
                "sound\\voice\\hearthfires.esm\\femaleelfhaughty"
            )),
            18022389080945785,
        );
        assert_eq!(super::calc_folder_name_hash(&String::from("x")), 2013331576,);
        assert_eq!(
            super::calc_folder_name_hash(&String::from("xx")),
            2013397112,
        );
        assert_eq!(
            super::calc_folder_name_hash(&String::from("xxx")),
            2013493368,
        );
    }
}
