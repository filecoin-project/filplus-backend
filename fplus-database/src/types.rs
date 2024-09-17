use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct DbConnectParams {
    password: String,
    dbname: String,
    engine: String,
    port: u16,
    host: String,
    username: String,
}

impl DbConnectParams {
    pub fn to_url(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}?{}",
            self.engine,
            self.username,
            urlencoding::encode(&self.password),
            self.host,
            self.port,
            self.dbname,
            std::env::var("DB_OPTIONS").unwrap_or_default(),
        )
    }
}
