extern crate regex;
use std::str::FromStr;

use alloy::{
    network::TransactionBuilder,
    primitives::{Address, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    sol,
    sol_types::SolCall,
};

use crate::{config::get_env_var_or_default, error::LDNError};

use super::filecoin::filecoin_address_to_evm_address;

sol! {
  #[allow(missing_docs)]
  function allowance(address allocator) view returns (uint256);
}
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

// TODO: Change new function to get api_key and base_url as arguments
#[allow(clippy::new_without_default)]
impl BlockchainData {
    /// Setup new BlockchainData client.
    pub fn new() -> Self {
        use crate::config::get_env_var_or_default;
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        let api_key = get_env_var_or_default("DMOB_API_KEY");
        let header = header::HeaderValue::from_str(&api_key)
            .expect("Env DMOB_API_KEY should be a valid HTTP header value");
        headers.insert("X-api-key", header);
        let client = reqwest::Client::builder()
            .user_agent("FP-CORE/0.1.0")
            .default_headers(headers)
            .connection_verbose(true)
            .build()
            .expect("Failed to build client");

        BlockchainData {
            client,
            base_url: format!(
                "{}{}",
                get_env_var_or_default("DMOB_API_URL"),
                "/public/api"
            ),
        }
    }

    /// Get Verified Clients
    pub async fn get_verified_clients(&self) -> Result<String, BlockchainDataError> {
        let query = "getVerifiedClients";
        let url = self.build_url(query);

        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| BlockchainDataError::Err(e.to_string()))?;

        let body = res
            .text()
            .await
            .map_err(|e| BlockchainDataError::Err(e.to_string()))?;

        Ok(body)
    }

    /// Build URL
    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path)
    }
}

pub async fn get_allowance_for_address_contract(
    user_address: &str,
    contract_address: &str,
) -> Result<u64, LDNError> {
    let rpc_url = get_env_var_or_default("GLIF_NODE_URL");
    let provider = ProviderBuilder::new()
        .on_builtin(&rpc_url)
        .await
        .map_err(|e| LDNError::New(format!("Invalid RPC URL: {e:?}")))?;
    let evm_user_address: Address = filecoin_address_to_evm_address(user_address)
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Failed to get evm address from filecoin address: {e:?}"
            ))
        })?
        .parse()
        .map_err(|e| {
            LDNError::New(format!(
                "Failed to get evm address from filecoin address: {e:?}"
            ))
        })?;

    let evm_contract_address = filecoin_address_to_evm_address(contract_address)
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Failed to get evm address from filecoin address: {e:?}"
            ))
        })?
        .parse()
        .map_err(|e| {
            LDNError::New(format!(
                "Failed to get evm address from filecoin address: {e:?}"
            ))
        })?;

    let call = allowanceCall {
        allocator: evm_user_address,
    }
    .abi_encode();
    let input = Bytes::from(call);
    let tx = TransactionRequest::default()
        .with_to(evm_contract_address)
        .with_input(input);

    let response = provider
        .call(&tx)
        .block(BlockId::latest())
        .await
        .map_err(|e| LDNError::New(format!("Transaction failed: {e:?}")))?;

    let parsed_response = U256::from_str(&response.to_string())
        .map_err(|e| LDNError::Load(format!("Failed to parse response to U256: {}", e)))?
        .to::<u64>();

    Ok(parsed_response)
}
