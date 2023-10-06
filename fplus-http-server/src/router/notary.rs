use actix_web::{get, http::header::ContentType, web, HttpResponse, post};
use mongodb::Client;
use std::sync::Mutex;

#[get("/notary")]
pub async fn get(db_connection: web::Data<Mutex<Client>>) -> HttpResponse {
    match fplus_database::core::collections::notary::find(db_connection).await {
        Ok(i) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&i).unwrap()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/notary")]
pub async fn post(
    db_connection: web::Data<Mutex<Client>>,
    rkh: web::Json<fplus_database::core::collections::notary::Notary>,
) -> HttpResponse {
    match fplus_database::core::collections::notary::insert(db_connection, rkh.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
