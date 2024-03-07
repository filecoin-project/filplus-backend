use serde::Deserialize;
use uuid::Uuid;

use crate::error::LDNError;

#[derive(Deserialize)]
pub struct GenerateNonceQueryParams {
    pub owner: String,
    pub repo: String,
    pub wallet: String,
    pub multisig_address: String,
}


pub fn generate_nonce(wallet: String, multisig_address: String, owner: String, repo: String) -> Result<String, LDNError> {
  // Example verification: check if any of the parameters are empty
    if wallet.trim().is_empty() || multisig_address.trim().is_empty() || owner.trim().is_empty() || repo.trim().is_empty() {
        return Err("All query parameters must be provided and non-empty.");
    }

    // Generate a nonce (UUID)
    let id = Uuid::new_v4();

    // Return the generated nonce
    Ok(nonce)
}