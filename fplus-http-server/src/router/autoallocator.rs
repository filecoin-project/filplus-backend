use actix_web::{get, web, HttpResponse, Responder};
use fplus_database::database::autoallocations as autoallocations_db;
use fplus_lib::core::LastAutoallocationQueryParams;

#[get("/autoallocator/last_client_allocation")]
pub async fn last_client_allocation(
    query: web::Query<LastAutoallocationQueryParams>,
) -> impl Responder {
    match autoallocations_db::get_last_client_autoallocation(query.evm_wallet_address).await {
        Ok(last_client_allocation) => {
            HttpResponse::Ok().body(serde_json::to_string_pretty(&last_client_allocation).unwrap())
        }
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
