mod common_types;
mod objects;
mod page;
mod pdf_image;
mod utils;
pub use crate::{
    common_types::{Justify},
    pdf_image::{PDFImage},
    page::{PDFPage},
};

use std::{
    io::{
        BufWriter, Error as IOError, SeekFrom,
        prelude::*,
    },
    fs::{File},
    path::{Path, PathBuf},
};
use image::{ImageError};
use crate::{
    objects::{
        Object, ObjectId, ObjectIdGenerator,
        Dictionary, Name, Stream,
    },
};

pub type PDFResult<T> = Result<T, PDFError>;
#[derive(Debug)]
pub enum PDFError {
    BadImageColourType(String),
    ByteIndexTooLarge,
    FileAlreadyExists(PathBuf),

    IOError(IOError),
    ImageError(ImageError),
}
impl From<IOError> for PDFError {
    fn from(error: IOError) -> Self { Self::IOError(error) }
}
impl From<ImageError> for PDFError {
    fn from(error: ImageError) -> Self { Self::ImageError(error) }
}

pub struct DocumentWriter {
    file: BufWriter<File>,
    id_generator: ObjectIdGenerator,
    written_objects: Vec<WrittenObject>,
    pages_root_id: ObjectId,
    pages: Vec<PageRef>,
}
impl DocumentWriter {
    pub fn stream_to_file(path: impl AsRef<Path>, overwrite: bool) -> PDFResult<DocumentWriter> {
        let path = path.as_ref();
        if !overwrite && path.exists() {
            return Err(PDFError::FileAlreadyExists(path.to_path_buf()));
        }
        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(b"%PDF-1.7")?;

        let mut id_generator = ObjectIdGenerator::new();
        // The xref table needs to start with this object
        let written_objects = vec![WrittenObject::new(
            id_generator.next(u16::max_value()), 0, true
        )];
        let pages_root_id = id_generator.next(0);

        Ok(DocumentWriter {
            file,
            id_generator,
            written_objects,
            pages_root_id,
            pages: Vec::new(),
        })
    }

    pub fn add_image(&mut self, image: PDFImage) -> PDFResult<ImageRef> {
        let image_id = self.id_generator.next(0);
        let image_ref = crate::pdf_image::ref_from_image(image_id, &image);
        let image_stream = crate::pdf_image::make_image_stream(image);
        self.write_object_with_ref(image_id, image_stream)?;
        Ok(image_ref)
    }
    pub fn add_page(&mut self, page: PDFPage) -> PDFResult<PageRef> {
        let page_id = self.id_generator.next(0);
        let page_ref = crate::page::ref_from_page(page_id, &page);
        let content_stream_ref = self.write_object_ref(page.make_content_stream()?)?;
        let page_dictionary = crate::page::make_page_dictionary(
            self.pages_root_id, page, content_stream_ref);
        self.write_object_with_ref(page_id, page_dictionary)?;
        self.pages.push(page_ref.clone());
        Ok(page_ref)
    }

    pub fn finish_writing(mut self, outline_tree: Vec<OutlineItem>, document_info: DocumentInfo)
    -> PDFResult<()> {
        let mut pages = Dictionary::new();
        pages.insert(Name::type_name(), Name::pages());
        pages.insert(Name::count(), self.pages.len());
        let kids: Vec<Object> = self.pages.iter()
            .map(|page_ref| page_ref.id.into())
            .collect();
        pages.insert(Name::kids(), kids);
        self.write_object_with_ref(self.pages_root_id, pages)?;

        let outline_dictionary_ref = {
            let outline_root_id = self.id_generator.next(0);
            let outline_ids = self.write_outline_tree(outline_root_id, outline_tree)?;
            if outline_ids.len() == 0 {
                None
            } else {
                let mut outline_dictionary = Dictionary::new();
                outline_dictionary.insert(Name::type_name(), Name::outlines());
                outline_dictionary.insert(Name::first(), outline_ids[0]);
                outline_dictionary.insert(Name::last(), outline_ids[outline_ids.len() - 1]);
                self.write_object_with_ref(outline_root_id, outline_dictionary)?;
                Some(outline_root_id)
            }
        };
        let document_catalog_ref = {
            let mut catalog = Dictionary::new();
            catalog.insert(Name::type_name(), Name::catalog());
            catalog.insert(Name::pages(), self.pages_root_id);
            catalog.insert(Name::outlines(), outline_dictionary_ref);
            self.write_object_ref(catalog)?
        };
        let document_info_ref = {
            let info_dictionary = document_info.into_dictionary();
            self.write_object_ref(info_dictionary)?
        };
        let xref_table_start = self.file_position()?;
        self.write_xref_table()?;
        self.write_trailer(document_catalog_ref, document_info_ref)?;
        write!(&mut self.file, "\nstartxref\n{}\n%%EOF", xref_table_start)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn file_position(&mut self) -> PDFResult<u64> {
        let current_position = self.file.seek(SeekFrom::Current(0))?;
        Ok(current_position)
    }
}
impl DocumentWriter {
    fn write_object_ref<T: Into<Object>>(&mut self, object: T) -> PDFResult<ObjectId> {
        let new_id = self.id_generator.next(0);
        self.write_object_with_ref(new_id, object)?;
        Ok(new_id)
    }
    fn write_object_with_ref<T: Into<Object>>(&mut self, id: ObjectId, object: T) -> PDFResult<()> {
        // Start with a new line to guarantee no symantic collisions
        self.file.write(b"\n")?;
        let object_start = self.file_position()?;
        id.write_to(&mut self.file)?;
        self.file.write_all(b" obj\n")?;
        object.into().write_to(&mut self.file)?;
        self.file.write_all(b"\nendobj")?;

        self.written_objects.push(WrittenObject::new(id, object_start, false));
        Ok(())
    }

    fn write_outline_tree(&mut self, parent_id: ObjectId, outline_tree: Vec<OutlineItem>)
    -> PDFResult< Vec<ObjectId> > {
        let outline_ids: Vec<ObjectId> = std::iter::repeat_with(|| self.id_generator.next(0))
            .take(outline_tree.len())
            .collect();
        if outline_ids.len() != 0 {
            let max_index = outline_ids.len() - 1;
            for (i, outline_item) in outline_tree.into_iter().enumerate() {
                let mut item_dictionary = Dictionary::new();
                item_dictionary.insert(Name::title(), outline_item.name);
                item_dictionary.insert(Name::parent(), parent_id);
                if i > 0 {
                    item_dictionary.insert(Name::prev(), outline_ids[i - 1]);
                }
                if i < max_index {
                    item_dictionary.insert(Name::next(), outline_ids[i + 1]);
                }
                let dest_array: Vec<Object> = vec![
                    outline_item.page.id.into(),
                    Name::new("XYZ").into(),
                    0.into(), outline_item.page.height.into(), Object::Null,
                ];
                item_dictionary.insert(Name::dest(), dest_array);

                let item_id = outline_ids[i];
                let child_ids = self.write_outline_tree(item_id, outline_item.children)?;
                if child_ids.len() > 0 {
                    item_dictionary.insert(Name::first(), child_ids[0]);
                    item_dictionary.insert(Name::last(), child_ids[child_ids.len() - 1]);
                    // Make sure that all the children are closed (negative length of children)
                    item_dictionary.insert(Name::count(), -(child_ids.len() as i64));
                }
                self.write_object_with_ref(item_id, item_dictionary)?;
            }
        }
        Ok(outline_ids)
    }

    fn write_xref_table(&mut self) -> PDFResult<()> {
        self.file.write_all(b"\nxref\n")?;

        self.written_objects.sort_by(|object1, object2| object1.id.cmp(&object2.id));
        let mut adjacent_object_lists: Vec< Vec<&WrittenObject> > = Vec::new();
        for written_object in &self.written_objects {
            if let Some(last_object_list) = adjacent_object_lists.last_mut() {
                let last_object = last_object_list.last().unwrap();
                if last_object.object_num() == (written_object.object_num() - 1) {
                    last_object_list.push(written_object);
                    continue;
                }
            }
            adjacent_object_lists.push(vec![written_object]);
        }

        for adjacent_objects in adjacent_object_lists {
            let start_object_num = adjacent_objects[0].object_num();
            write!(&mut self.file, "{} {}\n", start_object_num, adjacent_objects.len())?;
            for written_object in adjacent_objects {
                written_object.write_xref_line(&mut self.file)?;
            }
        }
        Ok(())
    }

    fn write_trailer(&mut self, root_id: ObjectId, info_id: ObjectId) -> PDFResult<()> {
        self.file.write_all(b"\ntrailer\n")?;

        let mut trailer = Dictionary::new();
        trailer.insert(Name::size(), self.written_objects.len());
        trailer.insert(Name::root(), root_id);
        trailer.insert(Name::info(), info_id);
        trailer.write_to(&mut self.file)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ImageRef {
    id: ObjectId,
    ref_name: Name,
    width: u32,
    height: u32,
}
impl ImageRef {
    pub fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
}
impl ImageRef {
    fn new(id: ObjectId, width: u32, height: u32) -> ImageRef {
        let ref_name = Name::new(format!("Image{}", id.object_num()));
        ImageRef { id, ref_name, width, height }
    }
}


#[derive(Copy, Clone)]
pub struct PageRef {
    id: ObjectId,
    height: f64,
}
impl PageRef {
    fn new(id: ObjectId, height: f64) -> PageRef {
        PageRef { id, height }
    }
}

pub struct OutlineItem {
    name: String,
    page: PageRef,
    children: Vec<OutlineItem>,
}
impl OutlineItem {
    pub fn new(name: impl ToString, page: PageRef) -> OutlineItem {
        OutlineItem {
            name: name.to_string(),
            page,
            children: Vec::new(),
        }
    }
    pub fn add_child(&mut self, outline_item: OutlineItem) {
        self.children.push(outline_item);
    }
}

pub struct DocumentInfo {
    title: Option<String>,
    author: Option<String>,
}
impl DocumentInfo {
    pub fn new() -> DocumentInfo {
        DocumentInfo {
            title: None,
            author: None,
        }
    }
    pub fn with_title(mut self, title: impl ToString) -> DocumentInfo {
        self.title = Some(title.to_string());
        self
    }
    pub fn with_author(mut self, author: impl ToString) -> DocumentInfo {
        self.author = Some(author.to_string());
        self
    }

    fn into_dictionary(self) -> Dictionary {
        let mut info_dictionary = Dictionary::new();
        info_dictionary.insert(Name::title(), self.title);
        info_dictionary.insert(Name::author(), self.author);
        info_dictionary
    }
}

struct WrittenObject {
    id: ObjectId,
    byte_offset: u64,
    is_free: bool,
}
impl WrittenObject {
    fn new(id: ObjectId, byte_offset: u64, is_free: bool) -> WrittenObject {
        WrittenObject { id, byte_offset, is_free }
    }
    fn object_num(&self) -> u32 { self.id.object_num() }

    fn write_xref_line<W: Write>(&self, writer: &mut W) -> PDFResult<()> {
        let byte_index_string = format!("{:010}", self.byte_offset);
        if byte_index_string.len() > 10 {
            return Err(PDFError::ByteIndexTooLarge);
        }
        let gen_number = self.id.gen_string();
        let object_type = if self.is_free { "f" } else { "n" };
        write!(writer, "{} {} {}\r\n", byte_index_string, gen_number, object_type)?;
        Ok(())
    }
}
