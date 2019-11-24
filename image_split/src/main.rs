use std::fs::{File};
use std::path::{PathBuf};

use image::{GenericImageView};
use serde::{Deserialize};
use rayon::prelude::*;

fn main() {
    let mut info_file = File::open("split_info.json")
        .expect("Failed to open the info file");
    let info: Info = serde_json::from_reader(&mut info_file)
        .expect("Failed to read the struct from the info file");

    info.split_info().into_par_iter().for_each(|split_info| {
        let mut image = image::open(&split_info.file)
            .expect(&format!("Failed to open {}", split_info.file.display()));
        let image_width = image.width();
        let image_height = image.height();

        let split_iterator = {
            let splits = split_info.splits;
            let mut index = 0;
            std::iter::from_fn(move || {
                let iter_item = if index < splits.len() - 1 {
                    let start = (splits[index] * image_width as f64) as u32;
                    let end = (splits[index + 1] * image_width as f64) as u32;
                    Some( (start, end) )
                } else {
                    None
                };
                index += 1;
                iter_item
            })
        };

        for ((start_x, end_x), new_file) in split_iterator.zip(split_info.new_files) {
            let new_image = image.crop(start_x, 0, end_x - start_x, image_height);
            new_image.save(&new_file)
                .expect(&format!("Failed to {}", new_file.display()));
        }
    });
}

#[derive(Deserialize)]
struct Info {
    image_folder: PathBuf,
    split_info: Vec<SplitInfo>,
}
impl Info {
    fn split_info(self) -> Vec<SplitItem> {
        let image_folder = self.image_folder;
        let items = self.split_info.into_iter().map(|split_info| {
            let file = image_folder.join(split_info.file);
            let mut splits = split_info.splits;
            splits.sort_by(|split1, split2| {
                if split1 < split2 {
                    std::cmp::Ordering::Less
                } else if split1 > split2 {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            // Add the start and end to make our iteration easier
            splits.insert(0, 0.0);
            splits.push(1.0);
            let new_files: Vec<PathBuf> = split_info.new_names.into_iter().map(|new_name| {
                image_folder.join(format!("{}.png", new_name))
            }).collect();

            if splits.len() - 1 != new_files.len() {
                panic!("Didn't get a matching len for {:?}", new_files);
            }

            SplitItem {
                file,
                splits,
                new_files,
            }
        }).collect();
        items
    }
}

#[derive(Deserialize)]
struct SplitInfo {
    /// The file to split
    file: String,
    /// The horizontal points (in percent 0 to 1) to make a split at.
    splits: Vec<f64>,
    /// The new names of the files from the split sections.
    /// Must be splits.len + 1.
    /// Must only be a name (no extension) since they will always be saved as PNG.
    /// Matches it up left to right across the image.
    new_names: Vec<String>,
}

struct SplitItem {
    file: PathBuf,
    /// Will have [0, some, splits, 1.0]
    splits: Vec<f64>,
    new_files: Vec<PathBuf>,
}
