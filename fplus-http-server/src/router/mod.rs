use actix_web::{get, HttpResponse, Responder};

pub mod allocator;
pub mod application;
pub mod autoallocator;
pub mod blockchain;
pub mod verifier;

/// Return server health status
#[get("/health")]
pub async fn health() -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok().body("OK"))
}
