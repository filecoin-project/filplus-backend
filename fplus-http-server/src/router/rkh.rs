use actix_web::{get, HttpResponse, Responder};
use fplus_lib::core::LDNApplication;

#[get("/rkhs")]
pub async fn rkhs() -> actix_web::Result<impl Responder> {
    match LDNApplication::fetch_rkh().await {
        Ok(notaries) => Ok(HttpResponse::Ok().json(notaries)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}
