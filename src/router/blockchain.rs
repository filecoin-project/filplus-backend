use crate::external_services::blockchain::BlockchainData;
use actix_web::{get, web, HttpResponse, Responder};

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
