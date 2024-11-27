use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_database::database::autoallocations as autoallocations_db;
use fplus_lib::core::autoallocator;
use fplus_lib::core::{LastAutoallocationQueryParams, TriggerAutoallocationInfo};
#[get("/autoallocator/last_client_allocation")]
pub async fn last_client_allocation(
    query: web::Query<LastAutoallocationQueryParams>,
) -> impl Responder {
    match autoallocations_db::get_last_client_autoallocation(query.evm_wallet_address).await {
        Ok(last_client_allocation) => match serde_json::to_string_pretty(&last_client_allocation) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("autoallocator/trigger_autoallocation")]
pub async fn trigger_autoallocation(info: web::Json<TriggerAutoallocationInfo>) -> impl Responder {
    match autoallocator::trigger_autoallocation(&info.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
