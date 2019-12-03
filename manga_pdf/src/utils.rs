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
