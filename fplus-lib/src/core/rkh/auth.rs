use fplus_database::database::rkh::create_or_update_rkh;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::LDNError;

#[derive(Deserialize)]
pub struct GenerateNonceQueryParams {
    pub wallet_address: String,
}


pub async fn generate_nonce(wallet_address: String) -> Result<String, LDNError> {
    if wallet_address.trim().is_empty() {
        return Err(LDNError::New("The wallet address must be provided and non-empty.".to_owned()));
    }

    let nonce = Uuid::new_v4().to_string();

    let result = create_or_update_rkh(wallet_address.as_str(), nonce.as_str()).await;
    if let Err(err) = result {
        // Handle the error here
        println!("Error: {}", err);
        return Err(LDNError::New("Failed to update nonce in database".to_owned()));
    }

    Ok(nonce)
}