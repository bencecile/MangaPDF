use std::{
    path::{Path},
    fs,
};
use image::{ColorType, DynamicImage, ImageFormat, ImageOutputFormat, GenericImageView};
use crate::{
    PDFError, PDFResult, ImageRef,
    Name, Dictionary, Stream, ObjectId,
};

pub struct PDFImage {
    image_bytes: Vec<u8>,
    width: u32,
    height: u32,
    image_type: ImageType,
    colour_type: ColourType,
}
impl PDFImage {
    pub fn from_path(image_path: impl AsRef<Path>, lossless: bool) -> PDFResult<PDFImage> {
        let image_bytes = fs::read(image_path)?;
        Self::from_bytes(image_bytes, lossless)
    }
    pub fn from_bytes(image_bytes: Vec<u8>, lossless: bool) -> PDFResult<PDFImage> {
        let image = image::load_from_memory(&image_bytes)?;
        let width = image.width();
        let height = image.height();
        let colour_type = ColourType::from_image_colour_type(image.color())
            .ok_or(PDFError::BadImageColourType)?;

        // Check if this is already a JPG image that we can use directly
        // We can unwrap it because we already know that we can read in the image
        match image::guess_format(&image_bytes)? {
            ImageFormat::JPEG => Ok(PDFImage {
                image_bytes,
                width,
                height,
                image_type: ImageType::Jpg,
                colour_type,
            }),
            _ => Self::from_image(image, lossless),
        }
    }
    pub fn from_image(image: DynamicImage, lossless: bool) -> PDFResult<PDFImage> {
        let try_to_convert_to_grayscale = |image, old_colour_type| {
            // Sample the image first to see if it can be turned to grayscale
            // We will only grayscale non-colour images
            //  This should save some space for all of the images that are just black and white
            if image_can_be_grayscale(&image) {
                (DynamicImage::ImageLuma8(image.to_luma()), ColourType::Gray)
            } else {
                (image, old_colour_type)
            }
        };

        // We need to make sure that the PDF can handle the image
        //  We can make it not use any weird color channels to accomplish this
        let (image, colour_type) = match image.color() {
            ColorType::Gray(_) => (image, ColourType::Gray),
            ColorType::RGB(_) => try_to_convert_to_grayscale(image, ColourType::RGB),

            ColorType::GrayA(_) => (
                DynamicImage::ImageLuma8(image.to_luma()),
                ColourType::Gray
            ),

            ColorType::Palette(_) |
            ColorType::RGBA(_) |
            ColorType::BGR(_) |
            ColorType::BGRA(_) => try_to_convert_to_grayscale(
                DynamicImage::ImageRgb8(image.to_rgb()),
                ColourType::RGB
            ),
        };

        let width = image.width();
        let height = image.height();
        // Make a rough estimate for the compressed image size so it's not quite so inefficient
        let rough_size = (width * height) as usize;

        let (image_bytes, image_type) = if lossless {
            let image_bytes = crate::utils::flate_compress(&image.raw_pixels(), Some(rough_size))?;
            (image_bytes, ImageType::FlateLossless)
        } else {
            // Convert a JPG for virtually lossless
            //  But still have a decent file size reduction
            let mut image_bytes = Vec::with_capacity(rough_size);
            image.write_to(&mut image_bytes, ImageOutputFormat::JPEG(90))?;
            (image_bytes, ImageType::Jpg)
        };
        Ok(PDFImage { image_bytes, width, height, image_type, colour_type })
    }
}

pub fn ref_from_image(id: ObjectId, image: &PDFImage) -> ImageRef {
    ImageRef::new(id, image.width, image.height)
}
pub fn make_image_stream(image: PDFImage) -> Stream {
    let mut image_dictionary = Dictionary::new();
    image_dictionary.insert(Name::type_name(), Name::xobject());
    image_dictionary.insert(Name::subtype(), Name::image());
    image_dictionary.insert(Name::width(), image.width);
    image_dictionary.insert(Name::height(), image.height);
    image_dictionary.insert(Name::color_space(), image.colour_type.pdf_colour_space());
    image_dictionary.insert(Name::bits_per_component(), 8);
    image_dictionary.insert(Name::filter(), image.image_type.pdf_filter());
    Stream::new(image_dictionary, image.image_bytes)
}

enum ImageType {
    FlateLossless,
    Jpg,
}
impl ImageType {
    fn pdf_filter(&self) -> Name {
        match self {
            Self::FlateLossless => Name::flate_decode(),
            Self::Jpg => Name::dct_decode(),
        }
    }
}
enum ColourType {
    Gray,
    RGB,
}
impl ColourType {
    fn from_image_colour_type(color_type: ColorType) -> Option<ColourType> {
        match color_type {
            ColorType::Gray(_) => Some(Self::Gray),
            ColorType::RGB(_) => Some(Self::RGB),
            _ => None,
        }
    }
    fn pdf_colour_space(&self) -> Name {
        match self {
            Self::Gray => Name::device_gray(),
            Self::RGB => Name::device_rgb(),
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
