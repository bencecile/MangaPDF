use std::{
    io::prelude::*,
    path::{Path},
    fs::{File},
};
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat, ImageOutputFormat};

pub struct PDFImage {
    raw_data: Vec<u8>,
    width: u32,
    height: u32,
    image_type: PDFImageType,
    colour_type: PDFImageColourType,
}
impl PDFImage {
    /// Gets the (width, height) of the image
    pub fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
    pub fn colour_type(&self) -> PDFImageColourType { self.colour_type }
    /// Gets the PDF definition for this image's colour space
    pub fn colour_space(&self) -> &'static str { self.colour_type.colour_space() }
    pub fn pdf_filter(&self) -> &'static str { self.image_type.pdf_filter() }
    pub fn raw_data(self) -> Vec<u8> { self.raw_data }
    pub fn raw_data_size(&self) -> usize { self.raw_data.len() }

    pub fn from_path(path: impl AsRef<Path>) -> PDFImage {
        let mut raw_data = Vec::new();
        File::open(path).unwrap().read_to_end(&mut raw_data).unwrap();
        Self::from_bytes(raw_data)
    }

    pub fn from_bytes(raw_data: Vec<u8>) -> PDFImage {
        let image = image::load_from_memory(&raw_data)
            .expect(&format!("Failed to open the bytes as an image"));

        // Check if this is already a JPG image that we can use directly
        // We can unwrap it because we already know that we can read in the image
        match image::guess_format(&raw_data).unwrap() {
            ImageFormat::JPEG => PDFImage {
                raw_data,
                width: image.width(),
                height: image.height(),
                image_type: PDFImageType::JPG,
                colour_type: PDFImageColourType::from_image_colour_type(image.color())
                    .expect(&format!(
                        "Failed to use the JPEG's color type ({:?}) directly",
                        image.color()
                    )),
            },
            _ => Self::from_image(image),
        }
    }

    pub fn from_image(image: DynamicImage) -> PDFImage {
        let try_to_convert_to_grayscale = |image, old_colour_type| {
            // Sample the image first to see if it can be turned to grayscale
            // We will only grayscale non-colour images
            //  This should save some space for all of the images that are just black and white
            if image_can_be_grayscale(&image) {
                (DynamicImage::ImageLuma8(image.to_luma()), PDFImageColourType::Gray)
            } else {
                (image, old_colour_type)
            }
        };

        // We need to make sure that the PDF can handle the image
        //  We can make it not use any weird color channels to accomplish this
        let (image, colour_type) = match image.color() {
            ColorType::Gray(_) => (image, PDFImageColourType::Gray),
            ColorType::RGB(_) => try_to_convert_to_grayscale(image, PDFImageColourType::RGB),

            ColorType::GrayA(_) => (
                DynamicImage::ImageLuma8(image.to_luma()),
                PDFImageColourType::Gray
            ),

            ColorType::Palette(_) |
            ColorType::RGBA(_) |
            ColorType::BGR(_) |
            ColorType::BGRA(_) => try_to_convert_to_grayscale(
                DynamicImage::ImageRgb8(image.to_rgb()),
                PDFImageColourType::RGB
            ),
        };

        // Make a rough estimate for the compressed image size so it's not quite so inefficient
        let rough_size = (image.width() * image.height()) as usize;

        // Convert a png into a jpg for virtually lossless
        //  But still have a decent file size reduction
        let mut jpg_data = Vec::with_capacity(rough_size);
        image.write_to(&mut jpg_data, ImageOutputFormat::JPEG(90))
            .expect("Failed to encode into a jpg");

        PDFImage {
            raw_data: jpg_data,
            width: image.width(),
            height: image.height(),
            image_type: PDFImageType::JPG,
            colour_type,
        }
    }
}

#[derive(Copy, Clone)]
pub enum PDFImageColourType {
    RGB,
    Gray,
}
impl PDFImageColourType {
    fn from_image_colour_type(colour_type: ColorType) -> Option<PDFImageColourType> {
        match colour_type {
            ColorType::Gray(_) => Some(Self::Gray),
            ColorType::RGB(_) => Some(Self::RGB),
            _ => None,
        }
    }

    fn colour_space(&self) -> &'static str {
        match self {
            Self::RGB => "DeviceRGB",
            Self::Gray => "DeviceGray",
        }
    }
}

enum PDFImageType {
    JPG,
}
impl PDFImageType {
    fn pdf_filter(&self) -> &'static str {
        match self {
            Self::JPG => "DCTDecode",
        }
    }
}

// Sample the image at intervals (instead of looking at every single pixel)
// This will speed things up and won't sacrifice accuracy (if it's not a super weird image)
fn image_can_be_grayscale(image: &DynamicImage) -> bool {
    const STEP_COUNT: u32 = 20;

    let (image_width, image_height) = (image.width(), image.height());
    let x_steps = (0..STEP_COUNT).map(move |count| image_width * count / STEP_COUNT);
    let y_steps = (0..STEP_COUNT).map(move |count| image_height * count / STEP_COUNT);

    for x in x_steps {
        for y in y_steps.clone() {
            let pixel = image.get_pixel(x, y);

            let first_component = pixel[0];
            // Each component (except alpha) will be the same if it's grayscale
            if pixel[1] != first_component || pixel[2] != first_component {
                return false;
            }
        }
    }
    true
}
