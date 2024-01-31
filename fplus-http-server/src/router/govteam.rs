use actix_web::{get, HttpResponse, Responder};
use fplus_lib::core::LDNApplication;

#[get("/gov-team-members")]
pub async fn gov_team_members() -> actix_web::Result<impl Responder> {
    match LDNApplication::fetch_gov().await {
        Ok(notaries) => {
            Ok(HttpResponse::Ok().json(notaries))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().body(e.to_string()))
        }
    }
}


