use std::{
    fs::{File},
    path::{Path},
};
use serde::de::{DeserializeOwned};
use serde_json;

pub fn read_json_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, String> {
    let path = path.as_ref();
    let json_file = File::open(path)
        .map_err(|_| format!("Failed to open the JSON file {}", path.display()))?;
    let json_object = serde_json::from_reader(json_file)
        .map_err(|e| format!("Failed to deserialize the JSON from {} ({})", path.display(), e))?;
    Ok(json_object)
}

pub fn file_name(path: &Path) -> &str {
    path.file_name().unwrap()
        .to_str().unwrap()
}
pub fn compare_file_name(path: &Path, other: &str) -> bool { file_name(path) == other }

pub fn byte_size_string(byte_size: u64) -> String {
    const PREFIXES: &'static [(&'static str, u64)] = &[
        ("GB", 1 << 30),
        ("MB", 1 << 20),
        ("KB", 1 << 10),
    ];
    let make_size_string = |prefix, threshold| {
        format!("{:.3} {}", (byte_size as f64) / (threshold as f64), prefix)
    };

    for &(prefix, threshold) in PREFIXES {
        if byte_size > threshold {
            return make_size_string(prefix, threshold);
        }
    }
    make_size_string("B", 1)
}
