mod text;
pub use self::{
    text::{TextContent, TextLayout, TextMetrics},
};

use std::{
    io::{Write},
};
use crate::{
    PDFResult,
    Name, Dictionary, Stream, Object, ObjectId, ImageRef, PageRef,
    Justify,
};

pub struct PDFPage {
    width: f64,
    height: f64,
    xobject_dictionary: Dictionary,
    instructions: Vec<(String, Vec<Object>)>,
}
impl PDFPage {
    pub fn new(width: f64, height: f64) -> PDFPage {
        PDFPage {
            width, height,
            xobject_dictionary: Dictionary::new(),
            instructions: Vec::new(),
        }
    }

    pub fn add_full_width_image(&mut self, image_ref: ImageRef) {
        self.add_image(image_ref, 0.0, 1.0, Justify::Center);
    }
    /// Justify will use the left as the start.
    pub fn add_image(&mut self, image_ref: ImageRef, start_x_percent: f64, end_x_percent: f64,
    justify: Justify) {
        let start_x = self.width * start_x_percent;
        let end_x = self.width * end_x_percent;
        let image_width_on_page = end_x - start_x;
        // We want to keep the image at the same ratio so it won't get stretched
        let (scale_width, scale_height, mut x, y) = {
            let pdf_page_ratio = image_width_on_page / self.height;
            let image_ratio = (image_ref.width as f64) / (image_ref.height as f64);

            // There we will empty space on the sides if the image is too tall
            if pdf_page_ratio > image_ratio {
                // Get the width where new_width:height will make the same image_ratio
                let new_width = self.height * image_ratio;
                // This is the difference that the width has to change to fit the images
                let leftover_width = image_width_on_page - new_width;
                // Once the image gets smaller, we have to center it
                (new_width, self.height, start_x + leftover_width / 2.0, 0.0)
            } else {
                // Do the same thing as the other branch, but with the height
                // We divide here because we need to inverse the ratio to make it height:width
                let new_height = image_width_on_page / image_ratio;
                let leftover_height = self.height - new_height;
                (image_width_on_page, new_height, start_x, leftover_height / 2.0)
            }
        };
        match justify {
            Justify::Start => { x = start_x; },
            Justify::End => { x = end_x - scale_width; },
            // It's already centered and space between is the same as center here
            _ => (),
        }
        // Make a new graphics frame so that we can easily change the view matrix
        self.add_instruction("q", Vec::new());
        // Translate it first
        self.add_instruction("cm", vec![
            1.into(), 0.into(), 0.into(), 1.into(), x.into(), y.into()
        ]);
        // Scale the image to fit the page
        self.add_instruction("cm", vec![
            scale_width.into(), 0.into(), 0.into(), scale_height.into(), 0.into(), 0.into()
        ]);
        self.add_instruction("Do", vec![image_ref.ref_name.clone().into()]);
        // Pop off the graphics frame we created
        self.add_instruction("Q", Vec::new());

        self.xobject_dictionary.insert(image_ref.ref_name, image_ref.id);
    }

    pub fn text_layout<'a>(&'a mut self,
    text_rect: (f64, f64, f64, f64), metrics: TextMetrics) -> TextLayout<'a> {
        self::text::new_text_layout(text_rect, metrics, self)
    }

    pub fn make_content_stream(&self) -> PDFResult<Stream> {
        let mut encoded_instructions: Vec<u8> = Vec::new();
        for (operator, arguments) in &self.instructions {
            for argument in arguments {
                argument.write_to(&mut encoded_instructions)?;
                encoded_instructions.push(b' ');
            }
            encoded_instructions.write_all(operator.as_bytes())?;
            encoded_instructions.push(b'\n');
        }

        let compressed_content = crate::utils::flate_compress(&encoded_instructions, None)?;
        let mut stream_dictionary = Dictionary::new();
        stream_dictionary.insert(Name::filter(), Name::flate_decode());
        Ok(Stream::new(stream_dictionary, compressed_content))
        // Ok(Stream::new(Dictionary::new(), encoded_instructions))
    }
}
impl PDFPage {
    fn add_instruction(&mut self, operator: impl ToString, arguments: Vec<Object>) {
        self.instructions.push( (operator.to_string(), arguments) );
    }
}

pub fn ref_from_page(id: ObjectId, page: &PDFPage) -> PageRef {
    PageRef::new(id, page.height)
}
pub fn make_page_dictionary(parent_id: ObjectId, page: PDFPage, content_stream_ref: ObjectId)
-> Dictionary {
    let mut page_dictionary = Dictionary::new();
    page_dictionary.insert(Name::type_name(), Name::page());
    page_dictionary.insert(Name::parent(), parent_id);
    page_dictionary.insert(Name::contents(), content_stream_ref);
    page_dictionary.insert(Name::media_box(), vec![0.0, 0.0, page.width, page.height]);

    let mut resource_dictionary = Dictionary::new();
    if !page.xobject_dictionary.is_empty() {
        resource_dictionary.insert(Name::xobject(), page.xobject_dictionary);
    }
    if !resource_dictionary.is_empty() {
        page_dictionary.insert(Name::resources(), resource_dictionary);
    }
    page_dictionary
}
