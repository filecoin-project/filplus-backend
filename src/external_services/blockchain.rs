const BASE_URL: &str = "https://api.filplus.d.interplanetary.one/public/api";

pub struct BlockchainData {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug)]
pub enum BlockchainDataError {
    ReqwestError(reqwest::Error),
}

impl BlockchainData {
    pub fn new() -> Self {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-api-key",
            header::HeaderValue::from_static("5c993a17-7b18-4ead-a8a8-89dad981d87e"),
        );
        let client = reqwest::Client::builder()
            .user_agent("FP-CORE/0.1.0")
            .default_headers(headers)
            .connection_verbose(true)
            .build()
            .expect("Failed to build client");

        BlockchainData {
            client,
            base_url: BASE_URL.to_string(),
        }
    }

    pub async fn get_verified_clients(&self) -> Result<String, BlockchainDataError> {
        let query = "getVerifiedClients";
        let url = self.build_url(query);
        let res = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => {
                println!("Error: {}", e);
                return Err(BlockchainDataError::ReqwestError(e));
            }
        };
        let body = match res.text().await {
            Ok(body) => body,
            Err(e) => {
                println!("Error: {}", e);
                return Err(BlockchainDataError::ReqwestError(e));
            }
        };
        return Ok(body);
    }

    pub async fn get_allowance_for_address(
        &self,
        address: &str,
    ) -> Result<String, BlockchainDataError> {
        let query = format!("getAllowanceForAddress/{}", address);
        let url = self.build_url(&query);
        let res = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => {
                println!("Error: {}", e);
                return Err(BlockchainDataError::ReqwestError(e));
            }
        };
        let body = res.text().await.unwrap();
        return Ok(body);
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path)
    }
}
