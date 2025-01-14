use actix_web::{error::ErrorInternalServerError, get, web, HttpResponse, Responder};
use fplus_lib::external_services::{
    blockchain::BlockchainData, filecoin::get_allowance_for_address,
};

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
pub async fn address_allowance(address: web::Path<String>) -> actix_web::Result<impl Responder> {
    let res = get_allowance_for_address(&address.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(res))
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
pub async fn verified_clients() -> actix_web::Result<impl Responder> {
    let blockchain = BlockchainData::new();
    let res = blockchain
        .get_verified_clients()
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(res))
}
