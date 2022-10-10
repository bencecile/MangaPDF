mod utils;
mod volumes;

use std::path::{PathBuf};

use rayon::prelude::*;
use serde::{Deserialize};

fn main() {
    let run_info: RunInfo = utils::read_json_file("run_info.json").unwrap();

    let volume_json_files: Vec<PathBuf> = run_info.json_files.iter()
        .map(|json_file| run_info.info_folder.join(json_file))
        .collect();
    // Make PDFs from all of the JSON files
    volume_json_files.par_iter()
        .for_each(|json_file| volumes::create_pdf(json_file, &run_info.out_folder));
}

#[derive(Deserialize)]
struct RunInfo {
    out_folder: PathBuf,
    info_folder: PathBuf,
    json_files: Vec<String>,
}
