use std::collections::BTreeMap;

pub const DEFAULT_KEY: u32 = 0x42424242;
pub const RENPY_PADDING: &[u8; 17] = b"Made with Ren'Py.";
pub type FileIndex = BTreeMap<String, Vec<(u64, u64, String)>>;
