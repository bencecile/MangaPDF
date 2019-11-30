use std::{
    io::{Write},
};
use flate2::{
    Compression,
    write::{ZlibEncoder},
};
use crate::{PDFResult};

pub fn flate_compress(to_compress: &[u8], size_hint: Option<usize>) -> PDFResult< Vec<u8> > {
    let compress_vec = {
        if let Some(size_hint) = size_hint { Vec::with_capacity(size_hint) }
        else { Vec::new() }
    };
    let mut encoder = ZlibEncoder::new(compress_vec, Compression::best());
    encoder.write_all(to_compress)?;
    Ok(encoder.finish()?)
}

pub fn to_utf16(string: &str) -> Vec<u8> {
    // Write the Big endian UTF-16 identifier bytes
    let mut utf_bytes: Vec<u8> = vec![0xFE, 0xFF];
    string.encode_utf16().for_each(|utf16_part| {
        utf_bytes.extend_from_slice(&utf16_part.to_be_bytes());
    });
    utf_bytes
}
