use std::io::Cursor;

use base64;

use crate::core::application::file::{ApplicationFile, ValidNotaryList, ValidGovTeamList};

pub fn decode(i: &str) -> Option<ApplicationFile> {
    let mut binding = Cursor::new(i);
    let decoder = base64::read::DecoderReader::new(&mut binding, base64::STANDARD);
    let app_file: ApplicationFile = match serde_json::from_reader(decoder) {
        Ok(f) => f,
        Err(_) => return None,
    };
    Some(app_file)
}

pub fn decode_notary(i: &str) -> Option<ValidNotaryList> {
    let mut binding = Cursor::new(i);
    let decoder = base64::read::DecoderReader::new(&mut binding, base64::STANDARD);
    let notaries: ValidNotaryList = match serde_json::from_reader(decoder) {
        Ok(f) => f,
        Err(_) => return None,
    };
    Some(notaries)
}

pub fn decode_gov_team(i: &str) -> Option<ValidGovTeamList> {
    let mut binding = Cursor::new(i);
    let decoder = base64::read::DecoderReader::new(&mut binding, base64::STANDARD);
    let gov_team: ValidGovTeamList = match serde_json::from_reader(decoder) {
        Ok(f) => f,
        Err(_) => return None,
    };
    Some(gov_team)
}
