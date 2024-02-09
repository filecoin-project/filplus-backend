use actix_web::{get, post, put, HttpResponse, Responder};
use fplus_database::database::get_allocators;

#[get("/allocators")]
pub async fn allocators() -> impl Responder {
    let allocators = get_allocators().await;
    match allocators {
        Ok(allocators) => HttpResponse::Ok().json(allocators),
        Err(e) => {
          log::error!("Failed to fetch allocators: {}", e);
          HttpResponse::InternalServerError().finish()
        },
    }
}

#[post("/allocator/")]
pub async fn create_allocator() -> impl Responder {
    HttpResponse::Ok().body("create_allocator")
}

#[put("/allocator/{owner}/{repo}")]
pub async fn update_allocator() -> impl Responder {
    HttpResponse::Ok().body("update_allocator")
}