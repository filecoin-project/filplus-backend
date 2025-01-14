use actix_web::error::ErrorInternalServerError;
use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_database::database::autoallocations as autoallocations_db;
use fplus_lib::core::autoallocator;
use fplus_lib::core::{LastAutoallocationQueryParams, TriggerAutoallocationInfo};
#[get("/autoallocator/last_client_allocation")]
pub async fn last_client_allocation(
    query: web::Query<LastAutoallocationQueryParams>,
) -> actix_web::Result<impl Responder> {
    let last_client_allocation =
        autoallocations_db::get_last_client_autoallocation(query.evm_wallet_address)
            .await
            .map_err(ErrorInternalServerError)?;

    let serialized_last_client_allocation =
        serde_json::to_string_pretty(&last_client_allocation).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(serialized_last_client_allocation))
}

#[post("autoallocator/trigger_autoallocation")]
pub async fn trigger_autoallocation(
    info: web::Json<TriggerAutoallocationInfo>,
) -> actix_web::Result<impl Responder> {
    autoallocator::trigger_autoallocation(&info.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}
