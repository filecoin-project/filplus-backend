use actix_web::{get, post, HttpResponse, Responder};
use fplus_lib::core::{LDNApplication, application::file::LDNActorsResponse};

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

#[get("/ldn-actors")]
pub async fn ldn_actors() -> actix_web::Result<impl Responder> {
    match LDNApplication::fetch_rkh_and_notary_gh_users().await {
        Ok((governance_gh_handles, notary_gh_handles)) => {
            let response = LDNActorsResponse { governance_gh_handles, notary_gh_handles };
            Ok(HttpResponse::Ok().json(response))
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
