use lib_stream_pdf::{
    DocumentWriter, PDFFont, PDFPage, PDFResult, DocumentInfo,
    FontLang, TextContent, TextMetrics, TextLayout, TextJustify,
};

fn main() -> PDFResult<()> {
    let temp_path = "C:/Manga/!BooksToCopy/TempText.pdf";
    let mut doc_writer = DocumentWriter::stream_to_file(temp_path, true)?;

    let font = PDFFont::new_truetype("MS-Mincho", FontLang::En)?;
    let font_ref = doc_writer.add_font(font)?;

    let mut page = PDFPage::new(600.0, 600.0);
    let title_metrics = TextMetrics::new(&font_ref, 30.0, TextJustify::Center)
        .with_line_gap(5.0);
    let mut text_layout = page.text_layout((10.0, 10.0, 590.0, 590.0), title_metrics);
    text_layout.println(vec![TextContent::text("This is a title")]);

    doc_writer.finish_writing(Vec::new(), DocumentInfo::new())?;

    Ok(())
}
