mod info;
mod volume;

use std::path::{Path};

// Page size calculations
const POINTS_PER_INCH: f64 = 72.0;
const POINTS_PER_MM: f64 = 1.0 / (10.0 * 2.54) * POINTS_PER_INCH;

pub fn create_pdf(volume_json: impl AsRef<Path>, out_dir: impl AsRef<Path>) {
    // Create the path that we got
    let volume_json = volume_json.as_ref();
    let volume_info = crate::utils::read_json_file(&volume_json).unwrap();

    println!("Starting {}", volume_json.display());

    self::volume::make_volume(volume_info, out_dir)
        .expect(&format!("Failed to make the volume: {}", volume_json.display()));
}
