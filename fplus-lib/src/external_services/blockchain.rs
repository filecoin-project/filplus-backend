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

//Implement Display for BlockchainDataError
impl std::fmt::Display for BlockchainDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BlockchainDataError::Err(e) => write!(f, "Error: {}", e),
        }
    }
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
        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Error: {}", e);
                return Err(BlockchainDataError::Err("Error accessing DMOB api".to_string()))
            }
        };
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


fn parse_size_to_bytes(size: &str) -> Option<u64> {
    let parts = size.trim().splitn(2, |c: char| !c.is_ascii_digit()).collect::<Vec<_>>();
    if parts.len() != 2 {
        return None; // Incorrect format
    }

    let number = parts[0].parse::<u64>().ok()?;
    let unit = parts[1].trim();

    // Normalize the unit by removing any trailing 's' and converting to upper case
    let unit = unit.trim_end_matches('s').to_uppercase();

    match unit.as_str() {
        "KIB" => Some(number * 1024),                             // 2^10
        "MIB" => Some(number * 1024 * 1024),                      // 2^20
        "GIB" => Some(number * 1024 * 1024 * 1024),               // 2^30
        "TIB" => Some(number * 1024 * 1024 * 1024 * 1024),        // 2^40
        "PIB" => Some(number * 1024 * 1024 * 1024 * 1024 * 1024), // 2^50
        "KB"  => Some(number * 1000),                             // 10^3
        "MB"  => Some(number * 1000 * 1000),                      // 10^6
        "GB"  => Some(number * 1000 * 1000 * 1000),               // 10^9
        "TB"  => Some(number * 1000 * 1000 * 1000 * 1000),        // 10^12
        "PB"  => Some(number * 1000 * 1000 * 1000 * 1000 * 1000), // 10^15
        _ => None, // Unsupported unit
    }
}

pub fn compare_allowance_and_allocation(allowance: &str, new_allocation_amount: Option<String>) -> Option<bool> {
    let allowance_bytes = parse_size_to_bytes(allowance)?;
    let allocation_bytes = match new_allocation_amount {
        Some(amount) => parse_size_to_bytes(&amount)?,
        None => return None, 
    };

    Some(allowance_bytes >= allocation_bytes)
}