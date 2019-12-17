use font_kit::{
    font::{Font},
    hinting::{HintingOptions},
    source::{SystemSource},
};
use crate::{
    PDFError, PDFResult,
    Name, Dictionary, ObjectId, FontRef,
};

pub struct PDFFont {
    font: Font,
    lang: FontLang,
}
impl PDFFont {
    pub fn new_truetype(font_name: &str, lang: FontLang) -> PDFResult<PDFFont> {
        let font = SystemSource::new().select_by_postscript_name(font_name)
            .map_err(|_| PDFError::FontMissing)?
            .load()?;
        match lang.direction() {
            FontDirection::Vertical => {
                if !font.supports_hinting_options(HintingOptions::Vertical(1.0), true) {
                    return Err(PDFError::FontCantBeVertical);
                }
            },
            _ => (),
        }
        Ok(PDFFont { font, lang })
    }
}

#[derive(Copy, Clone)]
pub enum FontDirection {
    Horizontal,
    Vertical,
}
#[derive(Copy, Clone)]
pub enum FontLang {
    En,
    Ja(FontDirection),
}
impl FontLang {
    fn direction(&self) -> FontDirection {
        match self {
            Self::En => FontDirection::Horizontal,
            Self::Ja(direction) => *direction,
        }
    }
}

pub fn ref_from_font(id: ObjectId, pdf_font: &PDFFont) -> FontRef {
    FontRef {
        id,
        font: pdf_font.font.clone(),
        direction: pdf_font.lang.direction(),
    }
}
pub fn make_font_object(font: PDFFont) -> Dictionary {
    let mut font_dictionary = Dictionary::new();
    font_dictionary.insert(Name::type_name(), Name::font());

    font_dictionary
    // TODO
}
