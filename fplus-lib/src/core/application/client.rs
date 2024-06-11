use super::file::Client;

#[warn(dead_code)]
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
            social_media_type,
            role,
        } = self;
        !name.is_empty()
            && !region.is_empty()
            && !industry.is_empty()
            && !website.is_empty()
            && !social_media.is_empty()
            && !social_media_type.is_empty()
            && !role.is_empty()
    }
}
