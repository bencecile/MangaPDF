use std::{
    collections::{BTreeMap},
    fs,
    path::{Path, PathBuf},
};
use image::{DynamicImage, GenericImage, RgbImage};
use lib_pdf::{
    PDFImage, VolumeBuilder,
    utils as pdf_utils,
};
use super::info::{ChapterInfo, VolumeInfo, WidePageInfo};

type WideImagePath = (f64, Vec<PathBuf>);

pub fn make_volume(info: VolumeInfo, out_dir: impl AsRef<Path>) -> Result<(), String> {
    let image_paths = info.find_images()?;
    let save_path = info.save_path(out_dir);
    let (width, height) = info.dimensions_in_device_space();
    let mut chapter_map = create_chapter_map(&image_paths, info.chapter_list())?;
    let wide_page_infos = info.wide_page_info();
    let wide_image_paths = create_wide_page_list(&image_paths, wide_page_infos)?;

    let mut builder = VolumeBuilder::new();

    for wide_page_paths in wide_image_paths {
        let wide_image = create_wide_image(&wide_page_paths.1, wide_page_paths.0);
        builder.add_image_page(wide_image, width, height);

        let found_chapter = wide_page_paths.1.into_iter()
            .find_map(|path| chapter_map.remove(&path));
        if let Some(chapter_name) = found_chapter {
            builder.mark_as_chapter(&chapter_name);
        }
    }

    // Print out all of the chapters that are missing
    // This is good for debugging and ensuring the correctness of these volumes
    for (file_path, chapter) in chapter_map {
        println!("Failed to find {} for the chapter {}", file_path.display(), chapter);
    }

    // Create any missing directories
    fs::create_dir_all(save_path.parent().unwrap())
        .map_err(|e| format!("Failed to mkdirs for {}. {}", save_path.display(), e))?;
    builder.save(&save_path);

    builder.print_stats(pdf_utils::file_name(&save_path));

    Ok(())
}

fn create_chapter_map(image_paths: &[PathBuf], chapter_list: &[ChapterInfo])
-> Result<BTreeMap<PathBuf, String>, String> {
    let mut chapter_map = BTreeMap::new();
    for chapter_info in chapter_list {
        // Find the path to the real image
        let image_path = image_paths.into_iter()
            .find(|image_path| pdf_utils::compare_file_name(image_path, &chapter_info.file_name))
            .ok_or_else(|| format!(
                "Failed to find the file ({}) for chapter ({})",
                &chapter_info.file_name, &chapter_info.chapter_name
            ))?;
        // Fail on a duplicate key insertion (using the same file to start 2 different chapters)
        let key_before = chapter_map.insert(
            image_path.clone(),
            chapter_info.chapter_name.to_string()
        );
        if let Some(chapter_name) = key_before {
            return Err(format!(
                "The same file ({}) starts 2 different chapters ({} and {})",
                pdf_utils::file_name(image_path), chapter_name, &chapter_info.chapter_name
            ));
        }
    }
    Ok(chapter_map)
}

/// Returns the wide page format so that we can just use the same definition for every page.
fn create_wide_page_list(image_paths: &[PathBuf], wide_page_infos: Vec<WidePageInfo>)
-> Result<Vec<WideImagePath>, String> {
    let find_image_index = |wide_page_path_string: &str| {
        image_paths.iter().enumerate().find_map(|(i, path)| {
            if pdf_utils::compare_file_name(path, wide_page_path_string) {
                Some(i)
            } else {
                None
            }
        })
    };

    // We will want to mark each image when we find them
    let mut found_images = vec![false; image_paths.len()];
    // To make sure that we find all of them and can raise an error if we don't
    let mut found_wide_pages = vec![false; wide_page_infos.len()];
    // We know that we wil be able to (at least) hold all of the wide pages
    let mut wide_pages = Vec::with_capacity(image_paths.len());
    for (i, image) in image_paths.iter().enumerate() {
        // Make sure to skip the ones we've found by wide pages so that we don't double-up
        if found_images[i] { continue; }

        let is_part_of_wide_page = wide_page_infos.iter().enumerate()
            .find(|(_, wide_page_info)| {
                wide_page_info.pages.iter()
                    .any(|wide_page_string| pdf_utils::compare_file_name(image, wide_page_string))
            });
        if let Some((wide_index, wide_page_info)) = is_part_of_wide_page {
            let mut wide_page = Vec::with_capacity(wide_page_info.pages.len());
            // Find the other pages that go with it
            for wide_page_string in wide_page_info.pages.iter() {
                let image_index = match find_image_index(wide_page_string.as_str()) {
                    Some(index) => index,
                    None => return Err(
                        format!(
                            "Failed to find the image from the wide page part {}",
                            wide_page_string
                        )
                    ),
                };
                if found_images[image_index] {
                    return Err(format!(
                        "We've already used this image in a page ({}). Fix the wide pages",
                        wide_page_string
                    ));
                }
                found_images[image_index] = true;
                wide_page.push(image_paths[image_index].clone());
            }
            found_wide_pages[wide_index] = true;
            wide_pages.push( (wide_page_info.page_gap, wide_page) );
        } else {
            found_images[i] = true;
            wide_pages.push(
                (0.0, vec![image.clone()])
            );
        }
    }

    if !found_wide_pages.iter().all(|found_wide_page| *found_wide_page) {
        let not_found_wide_pages: Vec<&[String]> = wide_page_infos.iter()
            .zip(found_wide_pages.iter())
            .filter_map(|(wide_page_info, &found_wide_page)| if !found_wide_page {
                Some(wide_page_info.pages.as_slice())
            } else {
                None
            }).collect();
        return Err(format!("Failed to find all the wide pages {:?}", not_found_wide_pages));
    }

    Ok(wide_pages)
}

/// Creates an image from a series to images to collate into a single wide page.
/// Gap is the gap of whitespace between each page, in a percentage width of the total width.
fn create_wide_image(image_paths: &[PathBuf], gap: f64) -> PDFImage {
    // If there's only 1 image, we just need to read in that path
    if image_paths.len() == 1 {
        return PDFImage::from_path(&image_paths[0]);
    }

    // Read in the images from the paths
    let images: Vec<RgbImage> = image_paths.iter()
        .map(|image_path| image::open(image_path)
            .expect(&format!("Failed to open the image '{}'", image_path.display()))
            .to_rgb())
        .collect();
    // Find the new max height so that all images can fit
    let total_height = images.iter().map(|image| image.height()).max().unwrap();
    // Find the new max width for all the images side-by-side
    let total_width: u32 = images.iter().map(|image| image.width()).sum();
    let actual_gap_width = ((total_width as f64) * gap) as u32;
    // Find the width needed for the gaps
    let total_width = total_width + actual_gap_width * (image_paths.len() as u32 - 1);

    // Create an empty white canvas
    let mut new_image = RgbImage::from_pixel(
        total_width, total_height, image::Rgb([255, 255, 255])
    );
    // Create a handy function to add the images to the new image
    let mut add_image = |image: &RgbImage, x| {
        // We will want to vertically center the image that is smaller in height
        let y = (total_height - image.height()) / 2;
        if !new_image.copy_from(image, x, y) {
            // This is a programming bug if this ever happens
            panic!(
                "Failed to add an image to the new, wide image. x={} y={} oldWidth={} oldHeight={} totalWidth={} totalHeight={}",
                x, y, image.width(), image.height(), total_width, total_height
            );
        }
    };

    // We need to keep track of the progress on the image
    let mut width_progress = 0;
    for image in images {
        add_image(&image, width_progress);
        // | Added Image | -> Gap -> | Next Image|
        width_progress += image.width() + actual_gap_width;
    }

    PDFImage::from_image(DynamicImage::ImageRgb8(new_image))
}
