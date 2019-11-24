use lopdf::{
    dictionary, Document, Dictionary, Stream, Object, ObjectId,
};
use crate::{
    PageContent, ReadingDirection,
};

pub struct PageFiller<'a> {
    document: &'a mut Document,
    operations: Vec<Operation>,
    resource_dictionary: Dictionary,
    page_width: u32,
    page_height: u32,
    start_page_number: usize,
}
impl <'a> PageFiller<'a> {
    pub fn new(document: &'a mut Document, page_width: u32, page_height: u32,
    start_page_number: usize) -> PageFiller<'a> {
        PageFiller {
            document,
            operations: Vec::new(),
            resource_dictionary: Dictionary::new(),
            page_width,
            page_height,
            start_page_number,
        }
    }

    pub fn make_page_dictionary(self, page_tree_id: ObjectId) -> Dictionary {
        let page_content_id = self.document.add_object(Stream::new(
            Dictionary::new(), Content { operations: operations }.encode().unwrap()
        ));
        let mut page_dictonary = dictionary! {
            "Type" => "Page",
            "Parent" => page_tree_id,
            "Contents" => page_content_id,
            "MediaBox" => vec![0.into(), 0.into(), self.page_width.into(), self.page_height.into()],
        };
        if !self.resource_dictionary.is_empty() {
            let resource_id = self.document.add_object(self.resource_dictionary));
            page_dictionary.set("Resources", resource_id);
        }
        page_dictonary
    }

    pub fn fill_page(mut self, page_content: &PageContent) -> Self {
        self.make_page_content(page_content, 0, self.page_width, self.start_page_number);
        self
    }
    pub fn fill_half_page_each(mut self, left_content: &PageContent, right_content: &PageContent,
    reading_direction: ReadingDirection) -> Self {
        let half_width = self.page_width / 2;
        match reading_direction {
            ReadingDirection::RightToLeft => {
                self.make_page_content(left_content, 0, half_width, self.start_page_number);
                self.make_page_content(right_content, half_width, half_width,
                    self.start_page_number + 1);
            },
            ReadingDirection::LeftToRight => {
                self.make_page_content(left_content, 0, half_width, self.start_page_number + 1);
                self.make_page_content(right_content half_width, half_width,
                    self.start_page_number);
            },
        }
        self
    }

    fn make_page_content(&mut self, page_content: &PageContent, start_x: u32, usable_width: u32,
    page_number: usize) {
        compile_error!("TODO");
    }
}
