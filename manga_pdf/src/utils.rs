mod natural;

use std::{
    cmp::{Ordering},
    fs::{File},
    path::{Path, PathBuf},
};
use serde::de::{DeserializeOwned};
use serde_json;
use self::natural::{NaturalIterator};

pub fn read_json_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, String> {
    let path = path.as_ref();
    let json_file = File::open(path)
        .map_err(|_| format!("Failed to open the JSON file {}", path.display()))?;
    let json_object = serde_json::from_reader(json_file)
        .map_err(|e| format!("Failed to deserialize the JSON from {} ({})", path.display(), e))?;
    Ok(json_object)
}

pub fn natural_sort_pathbuf(path1: &PathBuf, path2: &PathBuf) -> Ordering {
    natural_sort(path1.to_str().unwrap(), path2.to_str().unwrap())
}

pub fn natural_sort(str1: &str, str2: &str) -> Ordering {
    let naturals1 = NaturalIterator::new(str1);
    let naturals2 = NaturalIterator::new(str2);

    // Find the first two numbers that aren't the same
    if let Some((n1, n2)) = naturals1.zip(naturals2).find(|(n1, n2)| n1 != n2) {
        n1.cmp(&n2)
    } else {
        // Use the length as a tie breaker
        str1.len().cmp(&str2.len())
    }
}
