mod pages;
mod stats;

use std::{
    iter,
    path::{Path},
};
use lopdf::{
    dictionary, Document, Dictionary, Stream, Object, ObjectId,
    content::{Content, Operation},
};
use self::{
    pages::{PageFiller},
    stats::{BuilderStats, PageStat},
};
use crate::{
    PDFImage, PDFImageColourType,
    utils,
};

pub struct VolumeBuilder {
    /// This is the PDF document
    doc: Document,
    /// This is the ID of the document root
    page_tree_id: ObjectId,
    /// These are the IDs of all the added pages
    pages: Vec<ObjectId>,
    page_number: usize,
    /// The id of the last page added
    last_page_id: Option<ObjectId>,
    /// This is a vector of chapter_name -> ObjectId (id of the page)
    outline: Vec<(String, ObjectId)>,

    stats: BuilderStats,
}

impl VolumeBuilder {
    pub fn new() -> VolumeBuilder {
        let mut doc = Document::with_version("1.7");
        let page_tree_id = doc.new_object_id();
        VolumeBuilder {
            doc,
            page_tree_id,
            pages: Vec::new(),
            // The first page will need to be 1
            page_number: 1,
            last_page_id: None,
            outline: Vec::new(),
            stats: BuilderStats::new(),
        }
    }

    pub fn print_stats(&self, name: &str) { self.stats.print_with_name(name) }

    /// The width and height should be in PDF dimensions to use for the page size
    pub fn add_image_page(&mut self, image: PDFImage, mut width: f64, height: f64) {
        // Get all the stats first before we start dropping things
        self.stats.add_page(match image.colour_type() {
            PDFImageColourType::RGB => PageStat::Colour(image.raw_data_size() as u64),
            PDFImageColourType::Gray => PageStat::Gray(image.raw_data_size() as u64),
        });

        let (image_width, image_height) = image.dimensions();
        // We will want a double wide page if the image is in landscape
        if image_width > image_height {
            width *= 2.0;
        }

        // Add the image to the document and grab the ID to reference it later
        let image_id = self.doc.add_object(Stream::new(dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => image_width,
            "Height" => image_height,
            "ColorSpace" => image.colour_space(),
            "Filter" => image.pdf_filter(),
            "BitsPerComponent" => 8,
        }, image.raw_data()));
        let resource_id = self.doc.add_object(dictionary! {
            "XObject" => dictionary! {
                "image" => image_id,
            },
        });

        // We want to keep the image at the same ratio so it won't get stretched
        let (scale_width, scale_height, x, y) = {
            let pdf_page_ratio = width / height;
            let image_ratio = (image_width as f64) / (image_height as f64);

            // When a ratio is larger, that means that it is wider and we will have to make
            //  bars on the sides to make sure the image ratio stays the same
            if pdf_page_ratio > image_ratio {
                // Get the width where new_width:height will make the same image_ratio
                let new_width = height * image_ratio;
                // This is the difference that the width has to change to fit the images
                let fit_width_diff = width - new_width;
                // Once the image gets smaller, we have to center it
                (new_width, height, fit_width_diff / 2.0, 0.0)
            } else {
                // Do the same thing as the other branch, but with the height
                // We divide here because we need to inverse the ratio to make it height:width
                let new_height = width / image_ratio;
                let fit_height_diff = height - new_height;
                (width, new_height, 0.0, fit_height_diff / 2.0)
            }
        };
        let image_instructions = Content {
            operations: vec![
                // Make a new graphics frame so that we can easily change the view matrix
                Operation::new("q", Vec::new()),
                // Translate it first
                Operation::new("cm", vec![
                    1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()
                ]),
                // Scale the image to fit the page
                Operation::new("cm", vec![
                    scale_width.into(), 0.into(), 0.into(), scale_height.into(), 0.into(), 0.into()
                ]),
                Operation::new("Do", vec!["image".into()]),
                // Pop off the graphics frame we created
                Operation::new("Q", Vec::new()),
            ],
        };

        let page_content_id = self.doc.add_object(
            Stream::new(dictionary! {}, image_instructions.encode().unwrap())
        );
        let page_id = self.doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => self.page_tree_id,
            "Resources" => resource_id,
            "Contents" => vec![page_content_id.into()],
            "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()],
        });

        self.pages.push(page_id);
        self.last_page_id = Some(page_id);
    }

    pub fn add_page(&mut self, page: PDFPage) {
        let page_dictionary = {
            if let Some( (other_content, reading_direction) ) = page.other_content() {
                let dict = PageFiller::new(
                    &mut self.doc, page.width(), page.height(), self.page_number)
                    .fill_half_page_each(page.content(), other_content, reading_direction)
                    .make_page_dictionary(self.page_tree_id);
                self.page_number += 2;
                dict
            } else {
                let dict = PageFiller::new(
                    &mut self.doc, page.width(), page.height(), self.page_number)
                    .fill_page(page.content())
                    .make_page_dictionary(self.page_tree_id);
                self.page_number += 1;
                dict
            }
        };

        let page_id = self.doc.add_object(page_dictionary);
        self.pages.push(page_id);
        self.last_page_id = Some(page_id);
    }

    /// Marks the last added page as the start of a new chapter
    /// Panics if there hasn't been an added page yet
    pub fn mark_as_chapter(&mut self, chapter_name: &str) {
        if let Some(page_id) = self.last_page_id {
            self.outline.push(
                (chapter_name.to_string(), page_id)
            );
        } else {
            panic!("No page has been added yet!");
        }
    }

    pub fn save(&mut self, save_file: impl AsRef<Path>) {
        // Create the top-most page tree dictionary
        let page_tree = dictionary! {
            "Type" => "Pages",
            "Kids" => self.pages.iter().map(|p| p.clone().into()).collect::<Vec<Object>>(),
            "Count" => self.pages.len() as i64,
            "Resources" => dictionary! {},
        };
        // Insert the page tree dictionary with the page tree id that we got at document creation
        self.doc.objects.insert(self.page_tree_id, page_tree.into());

        let mut catalog = dictionary! {
            "Type" => "Catalog",
            "Pages" => self.page_tree_id,
        };

        if self.outline.len() > 0 {
            let outline_tree_id = self.doc.new_object_id();

            let outline_len = self.outline.len();
            let ids: Vec<ObjectId> = iter::repeat_with(|| self.doc.new_object_id())
                .take(outline_len)
                .collect();

            for (i, (chapter_name, page_id)) in self.outline.iter().enumerate() {
                // Get the height of the page we're working with
                let height = self.doc.get_dictionary(*page_id).unwrap()
                    .get(b"MediaBox").unwrap()
                    .as_array().unwrap()[3].clone();
                let mut dict = dictionary! {
                    "Title" => Object::string_literal(utils::to_utf16(&chapter_name)),
                    "Parent" => outline_tree_id,
                    // Set the destination to the page with nothing special
                    // It will just go to the top of the page without changing zoom level
                    "Dest" => vec![
                        (*page_id).into(),
                        // The slash for the start of the name isn't needed
                        "XYZ".into(), 0.into(), height, Object::Null,
                    ],
                };

                // Only set these references if there is more than chapter
                if ids.len() > 1 {
                    // Set the next and previous references for each entry
                    if i == 0 {
                        // This is the first one, and there will be one next
                        dict.set("Next", ids[i + 1]);
                    } else if i == ids.len() - 1 {
                        dict.set("Prev", ids[i - 1]);
                    } else {
                        dict.set("Prev", ids[i - 1]);
                        dict.set("Next", ids[i + 1]);
                    }
                }

                self.doc.objects.insert(ids[i], dict.into());
            }

            let outline_tree = dictionary! {
                "Type" => "Outlines",
                "First" => ids[0],
                "Last" => *ids.last().unwrap(),
            };
            self.doc.objects.insert(outline_tree_id, outline_tree.into());

            catalog.set("Outlines", outline_tree_id);
        }

        let catalog_id = self.doc.add_object(catalog);
        self.doc.trailer.set("Root", catalog_id);

        let save_file = save_file.as_ref();

        // Parse the save file name to get some metadata to use for the PDF
        let pdf_title = find_title(save_file);
        let author = find_author(save_file).unwrap_or_else(|| {
            println!("Failed to find an author for {}", save_file.display());
            "Unknown"
        });

        let document_info_id = self.doc.add_object(dictionary! {
            "Title" => Object::string_literal(utils::to_utf16(pdf_title)),
            "Author" => Object::string_literal(utils::to_utf16(author)),
        });
        self.doc.trailer.set("Info", document_info_id);

        self.doc.save(save_file)
            .expect(&format!("Failed to save {}", save_file.display()));
    }
}

/// Takes the stem of a file to figure out the PDF title
fn find_title(file_path: &Path) -> &str {
    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
    file_stem.split(" [").next().unwrap_or_else(|| {
        // Now search for just the bracket
        file_stem.split('[').next().unwrap_or_else(|| {
            // Just return the full file stem if there's no author
            file_stem
        })
    })
}
/// Finds the author in the save file name.
/// Example: "Here is some book title [by me].pdf" would grab the "by me" part.
fn find_author(file_path: &Path) -> Option<&str> {
    let file_name = utils::file_name(file_path);
    if let Some(left_bracket) = file_name.rfind('[') {
        if let Some(right_bracket) = file_name.rfind(']') {
            // This must be on the right side of the left bracket
            if left_bracket < right_bracket {
                return Some(&file_name[(left_bracket + 1)..right_bracket]);
            }
        }
    }
    None
}
