use std::{
    path::{Path},
};

pub fn to_utf16(string: &str) -> Vec<u8> {
    // Write the Big endian UTF-16 identifier bytes
    let mut utf_bytes: Vec<u8> = vec![0xFE, 0xFF];
    string.encode_utf16().for_each(|two_bytes| {
        // Push the upper byte first
        utf_bytes.push(((two_bytes & 0xFF00) >> 8) as u8);
        utf_bytes.push((two_bytes & 0x00FF) as u8);
    });

    utf_bytes
}

pub fn file_name<'a>(path: &'a Path) -> &'a str {
    path.file_name().unwrap()
        .to_str().unwrap()
}
pub fn compare_file_name(path: &Path, other: &str) -> bool { file_name(path) == other }
