use actix_web::{get, HttpResponse, Responder};

pub mod application;
pub mod blockchain;

/// Return server health status
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
