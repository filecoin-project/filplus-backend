use crate::external_services::blockchain::BlockchainData;
use actix_web::{get, web, HttpResponse, Responder};

/// Address Allowance
/// Returns the allowance for a given address
/// Example:
/// curl http://localhost:8080/blockchain/address_allowance/0x1234
/// Returns:
/// {
/// "address": "0x1234",
/// "allowance": 10000
/// }
#[get("/blockchain/address_allowance/{address}")]
pub async fn address_allowance(address: web::Path<String>) -> impl Responder {
    let blockchain = BlockchainData::new();
    match blockchain
        .get_allowance_for_address(&address.into_inner())
        .await
    {
        Ok(res) => return HttpResponse::Ok().body(res),
        Err(_) => {
            return HttpResponse::InternalServerError().body("SOMETHING IS WRONG WITH DEMOB SETUP!");
        }
    }
}

/// Verified Clients
/// Returns the list of verified clients
/// Example:
/// curl http://localhost:8080/blockchain/verified_clients
/// Returns:
/// [
/// "0x1234",
/// "0x5678"
/// ]
#[get("/blockchain/verified_clients")]
pub async fn verified_clients() -> impl Responder {
    let blockchain = BlockchainData::new();
    match blockchain.get_verified_clients().await {
        Ok(res) => return HttpResponse::Ok().body(res),
        Err(_) => {
            return HttpResponse::InternalServerError().body("SOMETHING IS WRONG WITH DEMOB SETUP!");
        }
    }
}
