mod utils;
mod volumes;

use std::path::{PathBuf};

use rayon::prelude::*;
use serde::{Deserialize};

fn main() {
    let run_info: RunInfo = utils::read_json_file("run_info.json").unwrap();

    let volume_json_files: Vec<PathBuf> = run_info.json_files.iter()
        .map(|json_file| run_info.info_folder.join(json_file))
        .collect();
    // Make PDFs from all of the JSON files
    volume_json_files.par_iter()
        .for_each(|json_file| volumes::create_pdf(json_file, &run_info.out_folder));
}

#[derive(Deserialize)]
struct RunInfo {
    out_folder: PathBuf,
    info_folder: PathBuf,
    json_files: Vec<String>,
}
// use lib_stream_pdf::{DocumentWriter, PDFPage, PDFResult, OutlineItem, DocumentInfo};
// fn main() -> PDFResult<()> {
//     let pdf_file = "C:/Manga/!BooksToCopy/temp.pdf";
//     let mut document_writer = DocumentWriter::stream_to_file(pdf_file, true)?;
//     let page_ref = document_writer.add_page(PDFPage::new(600.0, 600.0))?;

//     let outline_item = OutlineItem::new("SomeOutline１２３", page_ref);
//     let document_info = DocumentInfo::new()
//         .with_title("SomeBigTitleなん(")
//         .with_author("Somebodyだれ()");
//     document_writer.finish_writing(vec![outline_item], document_info)?;

//     Ok(())
// }
