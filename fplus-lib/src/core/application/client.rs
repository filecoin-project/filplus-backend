use super::file::Client;

impl Client {
    fn new(i: Client) -> Self {
        Self { ..i }
    }

    fn validate(&self) -> bool {
        let Client {
            name,
            region,
            industry,
            website,
            social_media,
            role,
        } = self;
        name.len() > 0
            && region.len() > 0
            && industry.len() > 0
            && website.len() > 0
            && social_media.len() > 0
            && role.len() > 0
    }
}
