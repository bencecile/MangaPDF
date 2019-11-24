mod pdf_image;
mod text_page;
pub use pdf_image::{PDFImage};
pub use text_page::*;

/// Reading Direction is for page numbering.
pub struct PDFPage {
    content: PageContent,
    // Other content will always be rendered to the right of the original content
    other_content: Option<(PageContent, ReadingDirection)>,
    // Just the width and height of a single page
    width: u32,
    height: u32,
}
impl PDFPage {
    pub fn new_single_page(content: impl Into<PageContent>, width: u32, height: u32) -> PDFPage {
        PDFPage {
            content,
            other_content: None,
            width,
            height,
        }
    }
    pub fn new_double_page(left_side: impl Into<PageContent>, right_side: impl Into<PageContent>,
    width: u32, height: u32, direction: ReadingDirection) -> PDFPage {
        PDFPage {
            content: left_side,
            other_content: Some( (right_side, direction) ),
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn content(&self) -> &PageContent { &self.content }
    pub fn other_content(&self) -> Option<(&PageContent, ReadingDirection)> {
        self.other_content.as_ref().map(|(page_content, direction)| (page_content, *direction))
    }
}

pub enum PageContent {
    Image(PDFImage),
    Text(TextPage),
}
impl From<PDFImage> for PageContent {
    fn from(image: PDFImage) -> Self { Self::Image(image) }
}
impl From<TextPage> for PageContent {
    fn from(text: TextPage) -> Self { Self::Text(text) }
}

pub enum ReadingDirection {
    RightToLeft,
    LeftToRight,
}
