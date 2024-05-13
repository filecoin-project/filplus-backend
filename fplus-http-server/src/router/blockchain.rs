use std::str::FromStr;

use actix_web::{get, post, web, HttpResponse, Responder};
use alloy::signers::Signature;
use fplus_lib::external_services::blockchain::BlockchainData;
use serde::Deserialize;

use crate::middleware::SignatureReader;

/// Address Allowance.
///
/// # Returns
/// Returns the allowance for a given address.
///
/// # Example
/// ```plaintext
/// curl http://localhost:8080/blockchain/address_allowance/0x1234
/// ```
///
/// # Response
/// ```
/// {
/// "address": "0x1234",
/// "allowance": 10000
/// }
/// ```

#[get("/blockchain/address_allowance/{address}")]
pub async fn address_allowance(address: web::Path<String>) -> impl Responder {
    let blockchain = BlockchainData::new();
    match blockchain
        .get_allowance_for_address(&address.into_inner())
        .await
    {
        Ok(res) => return HttpResponse::Ok().body(res),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("SOMETHING IS WRONG WITH DEMOB SETUP!");
        }
    }
}

/// Verified Clients.
///
/// # Returns
/// Returns the list of verified clients.
///
/// # Example
/// ```plaintext
/// curl http://localhost:8080/blockchain/verified_clients
/// ```
///
/// # Response
/// ```
/// [
/// "0x1234",
/// "0x5678"
/// ]
/// ```

#[get("/blockchain/verified_clients")]
pub async fn verified_clients() -> impl Responder {
    let blockchain = BlockchainData::new();
    match blockchain.get_verified_clients().await {
        Ok(res) => return HttpResponse::Ok().body(res),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("SOMETHING IS WRONG WITH DEMOB SETUP!");
        }
    }
}

#[post("/blockchain/verify")]
pub async fn verify(signature: web::Json<SignatureRequest>) -> impl Responder {
    let signature_reader = SignatureReader;

    let signature = match Signature::from_str(&signature.signature) {
        Ok(signature) => signature,
        Err(_) => return HttpResponse::BadRequest(),
    };

    // todo move message to .env
    let _address = signature_reader
        .get_address_from_signature(&signature, b"Hello world2")
        .await
        .unwrap();

    // todo add call to gitcoin

    HttpResponse::Ok()
}

#[derive(Deserialize)]
struct SignatureRequest {
    signature: String,
}
