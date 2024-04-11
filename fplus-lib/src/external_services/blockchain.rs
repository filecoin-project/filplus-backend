const BASE_URL: &str = "https://api.filplus.d.interplanetary.one/public/api";

/// BlockchainData is a client for the Fil+ blockchain data API.
pub struct BlockchainData {
    client: reqwest::Client,
    base_url: String,
}

/// BlockchainDataError is an error type for BlockchainData.
#[derive(Debug)]
pub enum BlockchainDataError {
   Err(String),
}

impl BlockchainData {
    /// Setup new BlockchainData client.
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

    /// Get Verified Clients
    pub async fn get_verified_clients(&self) -> Result<String, BlockchainDataError> {
        let query = "getVerifiedClients";
        let url = self.build_url(query);
        let res = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => {
                println!("Error: {}", e);
                return Err(BlockchainDataError::Err(e.to_string()))
            }
        };
        let body = match res.text().await {
            Ok(body) => body,
            Err(e) => {
                log::error!("Error: {}", e);
                return Err(BlockchainDataError::Err(e.to_string()))
            }
        };
        return Ok(body);
    }

    /// Get Allowance For Address
    pub async fn get_allowance_for_address(
        &self,
        address: &str,
    ) -> Result<String, BlockchainDataError> {
        let query = format!("getAllowanceForAddress/{}", address);
        let url = self.build_url(&query);
        let res = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Error: {}", e);
                return Err(BlockchainDataError::Err(e.to_string()))
            }
        };
        let body = res.text().await.unwrap();

        //Body json structure is {"type": "verifiedClient" | "error", ["allowance"]: string value, ["message"]: string value}
        // Let's parse the json and return the allowance value if the type is verifiedClient
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        match json["type"].as_str() {
            Some("verifiedClient") => {
                let allowance = json["allowance"].as_str().unwrap_or("");
                Ok(allowance.to_string())
            }
            Some("error") => {
                let message = json["message"].as_str().unwrap_or("");
                Err(BlockchainDataError::Err(message.to_string()))
            }
            _ => {
                Err(BlockchainDataError::Err("Unknown error".to_string()))
            }
        }

    }

    /// Build URL
    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path)
    }
}
