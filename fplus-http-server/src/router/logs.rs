use actix_web::{get, http::header::ContentType, post, web, HttpResponse};
use mongodb::Client;
use std::sync::Mutex;

#[get("/logs")]
pub async fn get() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

#[post("/logs")]
pub async fn post() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}
