use std::io::Cursor;

use base64;

use crate::core::application::ApplicationFile;

pub fn decode(i: &str) -> Option<ApplicationFile> {
    let mut binding = Cursor::new(i);
    let decoder = base64::read::DecoderReader::new(&mut binding, base64::STANDARD);
    let app_file: ApplicationFile = match serde_json::from_reader(decoder) {
        Ok(f) => f,
        Err(_) => return None,
    };
    Some(app_file)
}
