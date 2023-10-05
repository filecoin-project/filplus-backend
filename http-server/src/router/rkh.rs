use crate::db;
use actix_web::{get, http::header::ContentType, web, HttpResponse};
use mongodb::Client;
use std::sync::Mutex;

#[get("/rkh")]
pub async fn get(db_connection: web::Data<Mutex<Client>>) -> HttpResponse {
    let items = match db::collections::rkh::find(db_connection).await {
        Ok(items) => items,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    return HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&items).unwrap());
}
