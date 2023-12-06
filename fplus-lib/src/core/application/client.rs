use super::file::Client;

impl Client {
    fn new(i: Client) -> Self {
        Self { ..i }
    }
    //TODO: Check that none of the values are not default values
    // Used after finished the parsing
    pub fn validate(&self) -> bool {
        let Client {
            applicant,
            name,
            region,
            industry,
            website,
            social_media,
            social_media_type,
            role,
        } = self;
        applicant.len() > 0
            && name.len() > 0
            && region.len() > 0
            && industry.len() > 0
            && website.len() > 0
            && social_media.len() > 0
            && social_media_type.len() > 0
            && role.len() > 0
    }
}
