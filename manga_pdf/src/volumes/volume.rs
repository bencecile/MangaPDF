use std::{
    fs,
    path::{Path},
};
use lib_stream_pdf::{
    DocumentWriter, PDFPage, ImageRef, Justify,
    PageRef, OutlineItem,
};
use super::{
    info::{ChapterInfo, VolumeInfo, PageImageInfo},
    stats::{Stats, ImageStats},
};

pub fn make_volume(info: VolumeInfo, out_dir: impl AsRef<Path>) -> Result<(), String> {
    let save_path = info.save_path(out_dir);
    let (page_width, page_height) = info.dimensions_in_device_space();
    let mut outline_holders = OutlineItemHolder::from_chapter_infos(info.chapter_list());

    let mut stats = Stats::new();

    // Create any missing directories
    fs::create_dir_all(save_path.parent().unwrap())
        .map_err(|e| format!("Failed to mkdirs for {}. {}", save_path.display(), e))?;
    let mut doc_writer = DocumentWriter::stream_to_file(&save_path, true)
        .map_err(|e| format!("Failed to open the document writer: {:?}", e))?;

    for page_image_info in info.page_image_infos() {
        let mut pdf_image_refs = Vec::new();
        for (pdf_image, image_path) in page_image_info.make_pdf_images()? {
            let pdf_start_size = doc_writer.file_position()
                .map_err(|e| format!("Failed to get the starting file position ({:?}", e))?;

                let pdf_image_ref = doc_writer.add_image(pdf_image)
                .map_err(|e| format!("Failed to add the image: {:?}", e))?;
            pdf_image_refs.push(pdf_image_ref);

            let pdf_end_size = doc_writer.file_position()
                .map_err(|e| format!("Failed to get the ending file position ({:?})", e))?;
            stats.add_image_stats(
                ImageStats::new(image_path, pdf_end_size - pdf_start_size)
                    .map_err(|e| format!("Failed to make new image stats for {:?} ({:?})",
                        image_path, e))?
            );
        }
        if pdf_image_refs.is_empty() {
            return Err("A page can't be empty (aka. without images)".to_string());
        }

        let pdf_page = layout_page(
            pdf_image_refs, page_image_info.image_gap(), page_width, page_height
        );
        let page_ref = doc_writer.add_page(pdf_page)
            .map_err(|e| format!("Failed to add a page: {:?}", e))?;

        apply_to_holders_if_matching_page(&mut outline_holders, &page_image_info, page_ref);
    }

    let mut missed_outline_items = Vec::new();
    let mut outline_items = Vec::new();
    for outline_holder in outline_holders {
        match outline_holder.into_outline_item() {
            Ok(outline_item) => outline_items.push(outline_item),
            Err(missed_err) => missed_outline_items.push(missed_err),
        }
    }

    if missed_outline_items.len() > 0 {
        for missed_outline_item in missed_outline_items {
            println!("{}", missed_outline_item);
        }
        return Err("The outline tree is incomplete".to_string());
    }

    let document_info = info.make_document_info();
    doc_writer.finish_writing(outline_items, document_info)
        .map_err(|e| format!("Failed to finish writing: {:?}", e))?;

    stats.set_total_pdf_size(fs::metadata(&save_path).unwrap().len());
    stats.write_stats(&mut std::io::stdout())
        .map_err(|e| format!("Failed to write the stats ({:?})", e))?;

    Ok(())
}

struct OutlineItemHolder {
    name: String,
    file_name: String,
    page_ref: Option<PageRef>,
    children: Vec<OutlineItemHolder>,
}
impl OutlineItemHolder {
    fn from_chapter_infos(chapter_infos: &[ChapterInfo]) -> Vec<OutlineItemHolder> {
        chapter_infos.iter().map(|chapter_info| {
            let children = Self::from_chapter_infos(&chapter_info.children);
            OutlineItemHolder {
                name: chapter_info.chapter_name.clone(),
                file_name: chapter_info.file_name.clone(),
                page_ref: None,
                children,
            }
        }).collect()
    }

    /// Explicitly match any and all children without returning early
    fn apply_if_matching_page(&mut self, page_info: &PageImageInfo, page_ref: PageRef) {
        if page_info.has_image(&self.file_name) {
            self.page_ref = Some(page_ref);
        }
        apply_to_holders_if_matching_page(&mut self.children, page_info, page_ref);
    }

    fn into_outline_item(self) -> Result<OutlineItem, String> {
        let page_ref = match self.page_ref {
            Some(page_ref) => page_ref,
            None => return Err(
                format!("Failed to find a page with {} ({})", self.file_name, self.name)
            ),
        };
        let mut outline_item = OutlineItem::new(self.name, page_ref);
        for holder_child in self.children {
            outline_item.add_child(holder_child.into_outline_item()?);
        }
        Ok(outline_item)
    }
}
fn apply_to_holders_if_matching_page(holders: &mut [OutlineItemHolder], page_info: &PageImageInfo,
page_ref: PageRef) {
    for holder in holders {
        holder.apply_if_matching_page(page_info, page_ref);
    }
}

fn layout_page(image_refs: Vec<ImageRef>, image_gap: f64, mut page_width: f64, page_height: f64)
-> PDFPage {
    let num_images = image_refs.len();
    let total_image_width = image_refs.iter()
        .map(|image_ref| image_ref.dimensions().0)
        .sum::<u32>() as f64;
    let mut image_width_ratios: Vec<f64> = image_refs.iter()
        .map(|image_ref| image_ref.dimensions().0 as f64 / total_image_width)
        .collect();
    let total_gap_width_percent = (num_images - 1) as f64 * image_gap;

    // We'll want a double wide page to fit the extra image width (if any)
    let largest_height = image_refs.iter()
        .map(|image_ref| image_ref.dimensions().1)
        .max().unwrap() as f64;
    if total_image_width > largest_height {
        page_width *= 2.0;
    }

    let mut x_progress = if total_gap_width_percent.is_sign_negative() {
        // Since we will pull the images inwards from both sides (and only the 2 sides)
        // This will keep the image ratios to add up correctly
        total_gap_width_percent.abs() * 0.5
    } else if total_gap_width_percent > 1e-5 {
        // We'll need to fix the image ratios since we'll need more width than just the raw images
        // Each image will have to split how much extra width we'll gain from the gaps
        let width_loss_per_image = total_gap_width_percent / (num_images as f64);
        for ratio in image_width_ratios.iter_mut() {
            *ratio -= width_loss_per_image;
        }
        0.0
    } else {
        0.0
    };

    let mut pdf_page = PDFPage::new(page_width, page_height);
    let image_iterator = image_refs.into_iter().zip(image_width_ratios).enumerate();
    for (i, (image_ref, image_width_ratio)) in image_iterator {
        let justify = if num_images == 1 {
            Justify::Center
        } else {
            // Squish it towards the center
            if i < (num_images / 2) {
                Justify::End
            } else {
                Justify::Start
            }
        };
        pdf_page.add_image(image_ref, x_progress, x_progress + image_width_ratio, justify);
        x_progress += image_width_ratio + image_gap;
    }
    pdf_page
}
