use actix_web::{get, post, HttpResponse, Responder};
use fplus_lib::core::LDNApplication;

#[get("/notaries")]
pub async fn notaries() -> actix_web::Result<impl Responder> {
    match LDNApplication::fetch_notaries().await {
        Ok(notaries) => {
            Ok(HttpResponse::Ok().json(notaries))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().body(e.to_string()))
        }
    }
}

#[post("/notary")]
pub async fn post() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}
