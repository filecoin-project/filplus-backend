use base64::read::DecoderReader;
use serde_json::from_reader;
use base64;

use crate::core::{allocator::file::AllocatorModel, application::file::ApplicationFile};

pub fn decode_application_file(i: &str) -> Option<ApplicationFile> {
    let mut binding = Cursor::new(i);
    let decoder = DecoderReader::new(&mut binding, base64::STANDARD);
    let app_file: ApplicationFile = match serde_json::from_reader(decoder) {
        Ok(f) => f,
        Err(_) => return None,
    };
    Some(app_file)
}

use std::io::Cursor;


pub fn decode_allocator_model(encoded_str: &str) -> Option<AllocatorModel> {
    let mut binding = Cursor::new(encoded_str);
    let decoder = DecoderReader::new(&mut binding, base64::STANDARD);

    match from_reader(decoder) {
        Ok(model) => Some(model),
        Err(_) => None,
    }
}