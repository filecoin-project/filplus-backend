use fplus_database::database::rkh::{create_or_update_rkh, get_nonce_for_rkh};
use serde::Deserialize;
use uuid::Uuid;
use std::str::FromStr;
use crate::error::LDNError;
use ethers::core::types::Signature; // Add this import statement

#[derive(Deserialize)]
pub struct GenerateNonceQueryParams {
    pub wallet_address: String,
    pub multisig_address: String,
}

#[derive(Deserialize)]
pub struct TestSignaturePayload {
    pub wallet_address: String,
    pub signature: String
}

pub async fn verify_signature(
    public_address: &str,
    signature: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let nonce = get_nonce_for_rkh(public_address).await?.unwrap();
    println!("====================0");
    let message_hash = ethers::utils::keccak256(nonce.as_bytes());
    println!("====================1");
    println!("====================");
    println!("Message hash: {:?}", message_hash);
    println!("====================");
    println!("====================");
    let signature = Signature::from_str(signature)?;
    println!("====================2");
    let recovered_address = signature.recover(message_hash)?;
    println!("====================3");

    Ok(recovered_address == public_address.parse()?)
}

pub async fn generate_nonce(wallet_address: String, multisig_address: String) -> Result<String, LDNError> {
    if wallet_address.trim().is_empty() {
        return Err(LDNError::New("The wallet address must be provided and non-empty.".to_owned()));
    }

    // to-do check if addresss is part of multisig

    let nonce = Uuid::new_v4().to_string();

    let result = create_or_update_rkh(wallet_address.as_str(), nonce.as_str()).await;
    if let Err(err) = result {
        // Handle the error here
        println!("Error: {}", err);
        return Err(LDNError::New("Failed to update nonce in database".to_owned()));
    }

    Ok(nonce)
}