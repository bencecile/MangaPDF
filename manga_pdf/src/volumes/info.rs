use std::path::{Path, PathBuf};
use std::fs;

use serde::{Deserialize};

use crate::utils;
use crate::volumes::{POINTS_PER_MM};

/// This is the info json file that we use for each volume
#[derive(Deserialize)]
pub struct VolumeInfo {
    /// The path to the folder of images
    image_folder: String,
    /// The name of the output PDF (without the extension)
    save_name: String,
    /// The width of the pages, in millimeters
    width: f64,
    /// The height of the pages, in millimeters
    height: f64,
    /// The list of chapters for the PDF
    chapters: Vec<ChapterInfo>,
    /// Info for all of the wide pages
    wide_page_info: Vec<WidePageInfo>,
}
impl VolumeInfo {
    /// Gets the (width, height) dimensions usable for a PDF (units in device space)
    pub fn dimensions_in_device_space(&self) -> (f64, f64) {
        (self.width * POINTS_PER_MM, self.height * POINTS_PER_MM)
    }
    pub fn save_path(&self, base_dir: impl AsRef<Path>) -> PathBuf {
        base_dir.as_ref().join(&format!("{}.pdf", self.save_name))
    }
    pub fn find_images(&self) -> Result<Vec<PathBuf>, String> {
        // Create a list of paths to the images that are in the folder
        let mut images: Vec<PathBuf> = fs::read_dir(&self.image_folder)
            .map_err(|e| format!("Failed to read the image folder {}. {}", &self.image_folder, e))?
            .filter_map(|file| {
                // Let anything fail here
                let file = file.unwrap();
                if file.file_type().unwrap().is_file() {
                    Some(file.path())
                } else {
                    None
                }
            }).collect();
        // Sort all of the images so that they're in the correct order
        images.sort_by(utils::natural_sort_pathbuf);
        Ok(images)
    }
    pub fn chapter_list(&self) -> &[ChapterInfo] { &self.chapters }
    pub fn wide_page_info(&self) -> Vec<WidePageInfo> {
        // Ignore any empty page lists to make my life easier when making the info JSONs
        self.wide_page_info.iter().filter_map(|wide_page_info| {
            if wide_page_info.pages.is_empty() {
                None
            } else {
                Some(wide_page_info.clone())
            }
        }).collect()
    }
}

/// This the the chapter mapping info
#[derive(Deserialize)]
pub struct ChapterInfo {
    pub chapter_name: String,
    pub file_name: String,
}

#[derive(Clone, Deserialize)]
pub struct WidePageInfo {
    /// The percentage gap between each page in a wide page (1 is 100% of the total original width)
    pub page_gap: f64,
    /// Have a list of tupled image names that need to be combined (0: left -> len: right) together for an extra wide page (見開き)
    pub pages: Vec<String>,
}
