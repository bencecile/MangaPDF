use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    fs::{self, File},
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

    pub fn write_stats_to_file(&self, stat_path: impl AsRef<Path>) -> Result<(), String> {
        let file = File::create(stat_path)
            .map_err(|e| format!("Failed to open the file ({:?})", e))?;
        self.write_stats(&mut BufWriter::new(file))
    }
    pub fn write_stats<W: Write>(&self, writer: &mut W) -> Result<(), String> {
        let total_image_files_size = self.images.iter()
            .map(|image_stats| image_stats.file_size)
            .sum::<u64>();
        let total_image_in_pdf_size = self.images.iter()
            .map(|image_stats| image_stats.size_in_pdf)
            .sum::<u64>();

        writer.write_all(b"----- Totals -----\n")
            .map_err(|e| format!("Failed to write the totals header ({:?})", e))?;
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

        writer.write_all(b"----- Images -----\n")
            .map_err(|e| format!("Failed to write the images header ({:?})", e))?;
        for image_stats in &self.images {
            image_stats.write_stats(writer)?;
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
        writeln!(writer, "{}", self.path.display())
            .map_err(|e| format!("Failed to write the path ({:?})", e))?;
        writeln!(writer, "  File Size:              {}",
            crate::utils::byte_size_string(self.file_size))
            .map_err(|e| format!("Failed to write the file size ({:?})", e))?;
        writeln!(writer, "  Size in PDF:            {}",
            crate::utils::byte_size_string(self.size_in_pdf))
            .map_err(|e| format!("Failed to write the size in the PDF ({:?})", e))?;
        writeln!(writer, "  PDF-to-File Size Ratio: {:.3}", self.pdf_to_file_ratio)
            .map_err(|e| format!("Failed to write the ratio ({:?})", e))?;
        Ok(())
    }
}
