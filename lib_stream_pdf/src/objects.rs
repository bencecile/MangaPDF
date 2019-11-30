use std::{
    collections::{BTreeMap},
    io::{Write},
};
use crate::{PDFResult};

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Bool(bool),
    Int(i64),
    Real(f64),
    Name(Name),
    Str(String),
    Array(Vec<Object>),
    Dictionary(Dictionary),
    Stream(Stream),
    Ref(ObjectId),
}
impl From<bool> for Object { fn from(boolean: bool) -> Self { Self::Bool(boolean) } }
impl From<u8> for Object { fn from(int: u8) -> Self { Self::Int(int as i64) } }
impl From<u16> for Object { fn from(int: u16) -> Self { Self::Int(int as i64) } }
impl From<u32> for Object { fn from(int: u32) -> Self { Self::Int(int as i64) } }
impl From<usize> for Object { fn from(int: usize) -> Self { Self::Int(int as i64) } }
impl From<i8> for Object { fn from(int: i8) -> Self { Self::Int(int as i64) } }
impl From<i16> for Object { fn from(int: i16) -> Self { Self::Int(int as i64) } }
impl From<i32> for Object { fn from(int: i32) -> Self { Self::Int(int as i64) } }
impl From<i64> for Object { fn from(int: i64) -> Self { Self::Int(int) } }
impl From<f32> for Object { fn from(real: f32) -> Self { Self::Real(real as f64) } }
impl From<f64> for Object { fn from(real: f64) -> Self { Self::Real(real) } }
impl From<Name> for Object { fn from(name: Name) -> Self { Self::Name(name) } }
impl From<String> for Object { fn from(string: String) -> Self { Self::Str(string) } }
impl <'a> From<&'a str> for Object {
    fn from(string: &'a str) -> Self { Self::Str(string.to_string()) }
}
impl <T: Into<Object>> From< Vec<T> > for Object {
    fn from(array: Vec<T>) -> Self {
        let object_vec = array.into_iter()
            .map(|could_be_object| could_be_object.into()).collect();
        Self::Array(object_vec)
    }
}
impl From<Dictionary> for Object {
    fn from(dictionary: Dictionary) -> Self { Self::Dictionary(dictionary) }
}
impl From<Stream> for Object { fn from(stream: Stream) -> Self { Self::Stream(stream) } }
impl From<ObjectId> for Object { fn from(id: ObjectId) -> Self { Self::Ref(id) } }
impl <T: Into<Object>> From< Option<T> > for Object {
    fn from(option: Option<T>) -> Self { option.map_or(Object::Null, |object| object.into()) }
}

impl Object {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> PDFResult<()> {
        match self {
            Self::Null => writer.write_all(b"null")?,
            Self::Bool(boolean) => write!(writer, "{}", boolean)?,
            Self::Int(integer) => write!(writer, "{}", integer)?,
            // The PDF standard says to have up to 5 significant digits
            Self::Real(real) => write!(writer, "{:.5}", real)?,
            Self::Name(name) => name.write_to(writer)?,
            Self::Str(string) => {
                let mut write_string_bytes = |string_bytes: &[u8]| -> PDFResult<()> {
                    writer.write(b"(")?;
                    for &byte in string_bytes {
                        // Escape 0x28 `(`, 0x29 `)`, 0x5C `\` with 0x5C `\` in the PDF text
                        match byte {
                            b'(' | b')' | b'\\' => {
                                writer.write(b"\\")?;
                                writer.write(&[byte])?;
                            },
                            _ => { writer.write(&[byte])?; },
                        };
                    }
                    writer.write(b")")?;
                    Ok(())
                };
                if string.is_ascii() {
                    write_string_bytes(string.as_bytes())?
                } else {
                    let utf16_bytes = crate::utils::to_utf16(&string);
                    write_string_bytes(&utf16_bytes)?;
                }
            },
            Self::Array(array) => {
                writer.write(b"[")?;
                for object in array {
                    object.write_to(writer)?;
                    writer.write(b" ")?;
                }
                writer.write(b"]")?;
            },
            Self::Dictionary(dictionary) => dictionary.write_to(writer)?,
            Self::Stream(stream) => stream.write_to(writer)?,
            Self::Ref(object_id) => {
                object_id.write_to(writer)?;
                writer.write_all(b" R")?;
            },
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(u32, u16);
impl ObjectId {
    /// Makes the generation number into a string. Will fail if it's more than 5 digits.
    pub fn gen_string(&self) -> String { format!("{:05}", self.1) }
    pub fn object_num(&self) -> u32 { self.0 }
    pub fn write_to<W: Write>(&self, writer: &mut W) -> PDFResult<()> {
        write!(writer, "{} {}", self.0, self.1)?;
        Ok(())
    }
}
pub struct ObjectIdGenerator {
    next_id: u32,
}
impl ObjectIdGenerator {
    pub fn new() -> ObjectIdGenerator {
        ObjectIdGenerator { next_id: 0 }
    }

    pub fn next(&mut self, generation_num: u16) -> ObjectId {
        let given_id = self.next_id;
        self.next_id += 1;
        ObjectId(given_id, generation_num)
    }
}

/// A Name MUST NOT have a byte with a value of 0 (nul).
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Name(String);
impl Name {
    pub fn author() -> Name { Name::new("Author") }
    pub fn bits_per_component() -> Name { Name::new("BitsPerComponent") }
    pub fn catalog() -> Name { Name::new("Catalog") }
    pub fn color_space() -> Name { Name::new("ColorSpace") }
    pub fn contents() -> Name { Name::new("Contents") }
    pub fn count() -> Name { Name::new("Count") }
    pub fn dct_decode() -> Name { Name::new("DCTDecode") }
    pub fn dest() -> Name { Name::new("Dest") }
    pub fn device_gray() -> Name { Name::new("DeviceGray") }
    pub fn device_rgb() -> Name { Name::new("DeviceRGB") }
    pub fn filter() -> Name { Name::new("Filter") }
    pub fn first() -> Name { Name::new("First") }
    pub fn flate_decode() -> Name { Name::new("FlateDecode") }
    pub fn image() -> Name { Name::new("Image") }
    pub fn info() -> Name { Name::new("Info") }
    pub fn height() -> Name { Name::new("Height") }
    pub fn kids() -> Name { Name::new("Kids") }
    pub fn last() -> Name { Name::new("Last") }
    pub fn length() -> Name { Name::new("Length") }
    pub fn media_box() -> Name { Name::new("MediaBox") }
    pub fn next() -> Name { Name::new("Next") }
    pub fn outlines() -> Name { Name::new("Outlines") }
    pub fn page() -> Name { Name::new("Page") }
    pub fn pages() -> Name { Name::new("Pages") }
    pub fn parent() -> Name { Name::new("Parent") }
    pub fn prev() -> Name { Name::new("Prev") }
    pub fn resources() -> Name { Name::new("Resources") }
    pub fn root() -> Name { Name::new("Root") }
    pub fn size() -> Name { Name::new("Size") }
    pub fn subtype() -> Name { Name::new("Subtype") }
    pub fn title() -> Name { Name::new("Title") }
    pub fn type_name() -> Name { Name::new("Type") }
    pub fn width() -> Name { Name::new("Width") }
    pub fn xobject() -> Name { Name::new("XObject") }
}
impl Name {
    pub fn new(string: impl ToString) -> Name { Name(string.to_string()) }

    fn write_to<W: Write>(&self, writer: &mut W) -> PDFResult<()> {
        let mut converted = vec![b'/'];
        for byte in self.0.bytes() {
            if byte == b'#' {
                converted.write_all(b"#23")?;
            } else if (0x21 <= byte && byte <= 0x7E) && !super::DELIMITER_CHARS.contains(&byte) {
                converted.push(byte);
            } else {
                write!(&mut converted, "#{:X}", byte)?;
            }
        }

        writer.write_all(&converted)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Dictionary(BTreeMap<Name, Object>);
impl Dictionary {
    pub fn new() -> Dictionary { Dictionary(BTreeMap::new()) }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = (&Name, &Object)> { self.0.iter() }
    pub fn insert<T: Into<Object>>(&mut self, name: Name, value: T) {
        self.0.insert(name, value.into());
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> PDFResult<()> {
        writer.write_all(b"<<")?;
        for (name, object) in self.iter() {
            name.write_to(writer)?;
            writer.write(b" ")?;
            object.write_to(writer)?;
        }
        writer.write_all(b">>")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Stream(Dictionary, Vec<u8>);
impl Stream {
    pub fn new(mut dictionary: Dictionary, contents: Vec<u8>) -> Stream {
        dictionary.insert(Name::length(), contents.len());
        Stream(dictionary, contents)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> PDFResult<()> {
        self.0.write_to(writer)?;
        writer.write_all(b"stream\n")?;
        writer.write_all(&self.1)?;
        writer.write_all(b"\nendstream")?;
        Ok(())
    }
}
