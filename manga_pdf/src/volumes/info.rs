use std::{
    path::{Path, PathBuf},
};
use serde::{Deserialize};
use lib_stream_pdf::{DocumentInfo, PDFImage};
use super::{POINTS_PER_MM};

#[derive(Deserialize)]
pub struct VolumeInfo {
    /// The name of the output PDF (without the extension)
    save_name: String,
    title: String,
    author: Option<String>,
    /// The width of the pages, in millimeters
    width: f64,
    /// The height of the pages, in millimeters
    height: f64,
    chapters: Vec<ChapterInfo>,
    page_info: Vec<PageInfo>,
    lossless_images: Vec<String>,
}
impl VolumeInfo {
    /// Gets the (width, height) dimensions usable for a PDF (units in device space)
    pub fn dimensions_in_device_space(&self) -> (f64, f64) {
        (self.width * POINTS_PER_MM, self.height * POINTS_PER_MM)
    }
    pub fn save_path(&self, base_dir: impl AsRef<Path>) -> PathBuf {
        base_dir.as_ref().join(&format!("{}.pdf", self.save_name))
    }
    pub fn chapter_list(&self) -> &[ChapterInfo] { &self.chapters }
    pub fn page_image_infos(&self) -> Vec<PageImageInfo> {
        // Ignore any empty page lists to make my life easier when making the info JSONs
        self.page_info.iter().filter_map(|page_info| {
            if page_info.images.is_empty() {
                None
            } else {
                let images = page_info.images.iter().map(|image_path| {
                    let is_lossless = self.is_image_lossless(&image_path);
                    (image_path.clone(), is_lossless)
                }).collect();
                Some(PageImageInfo {
                    image_gap: page_info.image_gap,
                    images,
                })
            }
        }).collect()
    }
    pub fn make_document_info(&self) -> DocumentInfo {
        let document_info = DocumentInfo::new()
            .with_title(&self.title);
        if let Some(author) = &self.author {
            document_info.with_author(author)
        } else {
            document_info
        }
    }
}
impl VolumeInfo {
    fn is_image_lossless(&self, image_path: &Path) -> bool {
        self.lossless_images.iter()
            .any(|lossless_image| crate::utils::compare_file_name(image_path, lossless_image))
    }
}

/// This the the chapter mapping info
#[derive(Deserialize)]
pub struct ChapterInfo {
    pub chapter_name: String,
    pub file_name: String,
    pub children: Vec<ChapterInfo>,
}

#[derive(Clone, Deserialize)]
struct PageInfo {
    /// The percentage gap between each page in a wide page (1 is 100% of the total original width)
    image_gap: f64,
    /// Have a list of tupled image names that need to be combined (0: left -> len: right) together for an extra wide page (見開き)
    images: Vec<PathBuf>,
}
pub struct PageImageInfo {
    image_gap: f64,
    images: Vec<(PathBuf, bool)>,
}
impl PageImageInfo {
    pub fn image_gap(&self) -> f64 { self.image_gap }
    pub fn has_image(&self, file_name: &str) -> bool {
        self.images.iter()
            .any(|(image, _)| crate::utils::compare_file_name(image, file_name))
    }
    pub fn make_pdf_images(&self) -> Result<Vec<PDFImage>, String> {
        let mut pdf_images = Vec::new();
        for (image_path, lossless) in self.images.iter() {
            let pdf_image = PDFImage::from_path(&image_path, *lossless)
                .map_err(|e| format!("Failed to make the image: {:?}", e))?;
            pdf_images.push(pdf_image);
        }
        Ok(pdf_images)
    }
}
