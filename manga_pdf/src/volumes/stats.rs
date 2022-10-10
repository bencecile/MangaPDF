use std::{
    io::{Write},
    path::{Path, PathBuf},
    fs::{self},
    time::{Instant},
};

pub struct Stats {
    images: Vec<ImageStats>,
    total_pdf_size: u64,
    start_time: Instant,
}
impl Stats {
    pub fn new() -> Stats {
        Stats {
            images: Vec::new(),
            total_pdf_size: 0,
            start_time: Instant::now(),
        }
    }

    pub fn add_image_stats(&mut self, image_stats: ImageStats) {
        self.images.push(image_stats);
    }
    pub fn set_total_pdf_size(&mut self, total_pdf_size: u64) {
        self.total_pdf_size = total_pdf_size;
    }

    pub fn write_stats<W: Write>(&self, writer: &mut W) -> Result<(), String> {
        let total_image_files_size = self.images.iter()
            .map(|image_stats| image_stats.file_size)
            .sum::<u64>();
        let total_image_in_pdf_size = self.images.iter()
            .map(|image_stats| image_stats.size_in_pdf)
            .sum::<u64>();

        writeln!(writer, "Time Spent:              {:?}", self.start_time.elapsed())
            .map_err(|e| format!("Failed to write the time spent ({:?})", e))?;
        writeln!(writer, "Total PDF Size:          {}",
            crate::utils::byte_size_string(self.total_pdf_size))
            .map_err(|e| format!("Failed to write the total pdf size ({:?})", e))?;
        writeln!(writer, "Total Image File Size:   {}",
            crate::utils::byte_size_string(total_image_files_size))
            .map_err(|e| format!("Failed to write the total image file size ({:?})", e))?;
        writeln!(writer, "Total Image Size in PDF: {}",
            crate::utils::byte_size_string(total_image_in_pdf_size))
            .map_err(|e| format!("Failed to write the total image size in PDF ({:?})", e))?;

        for image_stats in &self.images {
            if image_stats.pdf_to_file_ratio > 1.01 || image_stats.pdf_to_file_ratio < 0.99 {
                image_stats.write_stats(writer)?;
            }
        }
        Ok(())
    }
}

pub struct ImageStats {
    path: PathBuf,
    file_size: u64,
    size_in_pdf: u64,
    pdf_to_file_ratio: f64,
}
impl ImageStats {
    pub fn new(path: impl AsRef<Path>, size_in_pdf: u64) -> Result<ImageStats, String> {
        let path = path.as_ref().to_owned();
        let metadata = fs::metadata(&path)
            .map_err(|e| format!("Failed to get the metadata for {:?} ({:?})", &path, e))?;
        let file_size = metadata.len();
        let pdf_to_file_ratio = (size_in_pdf as f64) / (file_size as f64);
        Ok(ImageStats { path, file_size, size_in_pdf, pdf_to_file_ratio })
    }
}
impl ImageStats {
    fn write_stats<W: Write>(&self, writer: &mut W) -> Result<(), String> {
        let file_name = self.path.file_name().unwrap();
        let o_bytes = crate::utils::byte_size_string(self.file_size);
        let n_bytes = crate::utils::byte_size_string(self.size_in_pdf);
        let ratio = self.pdf_to_file_ratio;
        writeln!(writer, "{:?} (Original {}, In-PDF {}, {:.3}x)", file_name, o_bytes, n_bytes, ratio)
            .map_err(|e| format!("Failed to write the image stats ({:?})", e))?;
        Ok(())
    }
}
