use super::{PDFPage};
use crate::{FontRef, FontDirection, Justify};

pub fn new_text_layout<'a>(text_rect: (f64, f64, f64, f64), metrics: TextMetrics,
page: &'a mut PDFPage) -> TextLayout<'a> {
    // Start the text object
    page.add_instruction("BT", Vec::new());
    let cursor = match metrics.font_ref.direction() {
        // Top-left corner for horizontal fonts
        FontDirection::Horizontal => (text_rect.0, text_rect.3),
        // Top-right corner for vertical fonts
        FontDirection::Vertical => (text_rect.2, text_rect.3),
    };
    TextLayout {
        text_rect,
        metrics,
        page,
        cursor,
    }
}
pub struct TextLayout<'a> {
    text_rect: (f64, f64, f64, f64),
    metrics: TextMetrics,
    page: &'a mut PDFPage,
    cursor: (f64, f64),
}
impl <'a> TextLayout<'a> {
    /// Returns any remaining text that couldn't fit on the line
    pub fn println(&mut self, text_contents: Vec<TextContent>) -> Option< Vec<TextContent> > {
for content in text_contents {
    match content {
        TextContent::Text(text) |
        TextContent::Ruby { base: text, .. } => {
            for c in text.chars() {
                println!("{:?} {:?}", c, self.metrics.font_ref.bounds_for_char(c));
            }
        },
    }
}
None
        // TODO
    }
    /// Returns any remaining text that couldn't fit in the text area
    pub fn paragraph(&mut self, text_contents: Vec<TextContent>) -> Option< Vec<TextContent> > {
        // Use the paragraph indent first
        match self.metrics.font_direction() {
            FontDirection::Horizontal => { self.cursor.0 += self.metrics.paragraph_indent; },
            FontDirection::Vertical => { self.cursor.1 -= self.metrics.paragraph_indent; },
        }
        let mut leftover_contents = Some(text_contents);
        while self.is_within_rect() {
            leftover_contents = self.println(leftover_contents.take().unwrap());
            if leftover_contents.is_none() { break; }
        }
        leftover_contents
    }
    pub fn new_line(&mut self) {
        match self.metrics.font_direction() {
            FontDirection::Horizontal => {
                // Move down and start at the left
                self.cursor.0 = self.text_rect.0;
                self.cursor.1 -= self.metrics.line_height();
            },
            FontDirection::Vertical => {
                // Move left and start at the top
                self.cursor.0 -= self.metrics.line_height();
                self.cursor.1 = self.text_rect.3;
            },
        }
    }
}
impl <'a> TextLayout<'a> {
    fn is_within_rect(&self) -> bool {
        let within_x = self.text_rect.0 < self.cursor.0 && self.cursor.0 < self.text_rect.2;
        let within_y = self.text_rect.1 < self.cursor.1 && self.cursor.1 < self.text_rect.3;
        within_x && within_y
    }
}
impl <'a> Drop for TextLayout<'a> {
    fn drop(&mut self) {
        self.page.add_instruction("ET", Vec::new());
    }
}

pub struct TextMetrics {
    font_ref: FontRef,
    text_height: f64,
    justify: Justify,
    line_gap: f64,
    paragraph_indent: f64,
}
impl TextMetrics {
    pub fn new(font_ref: &FontRef, text_height: f64, justify: Justify) -> TextMetrics {
        TextMetrics {
            font_ref: font_ref.clone(),
            text_height,
            justify,
            line_gap: 0.0,
            paragraph_indent: 0.0,
        }
    }
    pub fn with_line_gap(mut self, line_gap: f64) -> TextMetrics {
        self.line_gap = line_gap;
        self
    }
    pub fn with_paragraph_indent(mut self, paragraph_indent: f64) -> TextMetrics {
        self.paragraph_indent = paragraph_indent;
        self
    }
}
impl TextMetrics {
    fn font_direction(&self) -> FontDirection { self.font_ref.direction() }
    fn line_height(&self) -> f64 { self.text_height + self.line_gap }
}

pub enum TextContent {
    Text(String),
    Ruby {
        base: String,
        above: String,
    },
}
impl TextContent {
    pub fn text(string: &str) -> TextContent { TextContent::Text(string.to_string()) }
    pub fn ruby(base: &str, above: &str) -> TextContent {
        TextContent::Ruby{ base: base.to_string(), above: above.to_string() }
    }
}
