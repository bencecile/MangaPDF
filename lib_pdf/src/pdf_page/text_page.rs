use super::{ReadingDirection};

pub struct TextPage {
    direction: ReadingDirection,
    orientation: TextOrientation,
    margins: PageMargins,
    top_ribbon: Option<TextRibbon>,
    bottom_ribbon: Option<TextRibbon>,
    page_number: Option<PageNumber>,
    text_lines: Vec<Text>,
}
impl TextPage {
    pub fn new(direction: ReadingDirection, orientation: TextOrientation, margins: PageMargins)
    -> TextPage {
        TextPage {
            direction,
            orientation,
            margins,
            top_ribbon: None,
            bottom_ribbon: None,
            page_number: None,
            text_lines: Vec::new(),
        }
    }

    pub fn add_top_ribbon(&mut self, top_ribbon: TextRibbon) {
        self.top_ribbon = Some(top_ribbon);
    }
    pub fn add_bottom_ribbon(&mut self, bottom_ribbon: TextRibbon) {
        self.bottom_ribbon = Some(bottom_ribbon);
    }
    pub fn add_page_number(&mut self, page_number: PageNumber) {
        self.page_number = Some(page_number);
    }

    /// Uses the page size to format the pargraph onto this page.
    /// Returns any text that couldn't fit inside the page.
    pub fn format_paragraph(&mut self, paragraph: Text, _page_width: u32, _page_height: u32)
    -> Option<Text> {
        // TODO Actually format it
        self.text_lines.push(paragraph);
    }
}

#[derive(Copy, Clone)]
pub enum TextOrientation {
    Horizontal,
    Vertical,
}
/// Actual PDF units (same units as page width and height)
#[derive(Copy, Clone)]
pub struct PageMargins {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}
impl PageMargins {
    pub fn new(top: u32, right: u32, bottom: u32, left: u32) -> PageMargins {
        PageMargins { top, right, bottom, left }
    }
}

/// A page number will always be hugging the outside of the margin.
#[derive(Copy, Clone)]
pub struct PageNumber {
    top: bool,
    justify: TextJustify,
}
impl PageNumber {
    pub fn new(top: bool, justify: TextJustify) -> PageNumber {
        PageNumber { top, justify }
    }
}

/// Font size is in the PDF units.
pub struct Text {
    text: Vec<TextContent>,
    justify: TextJustify,
    font_name: String,
    font_size: u32,
}
impl Text {
    pub fn new(justify: TextJustify, font_name: String, font_size: u32) -> Text {
        Text { text: Vec::new(), justify, font_name, font_size }
    }

    pub fn add_text(&mut self, text: String) {
        self.text.push(TextContent::Span(text));
    }
    pub fn add_ruby(&mut self, base: String, above: String) {
        self.text.push(TextContent::Ruby { base, above });
    }
}

pub struct TextRibbon {
    text: String,
    align: TextAlign,
    justify: TextJustify,
}
impl TextRibbon {
    pub fn new(text: &str, align: TextAlign, justify: TextJustify) -> TextRibbon {
        TextRibbon { text: text.to_string(), align, justify }
    }
}

#[derive(Copy, Clone)]
pub enum TextAlign {
    Top,
    Center,
    Bottom,
}
/// Justify a line of text.
///
/// Space between: if there's and spaces in the text (0x20 ' '), the spaces between words is
/// extended to make the start and end justified. If there are no spaces, extra spacing
/// will be between each character.
#[derive(Copy, Clone)]
pub enum TextJustify {
    Start,
    End,
    Center,
    SpaceBetween,
}
pub enum PagePosition {
    Top,
    Bottom,
}
enum TextContent {
    Span(String),
    Ruby {
        base: String,
        above: String,
    },
}
