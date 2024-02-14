use actix_web::{get, web, HttpResponse, Responder};
use fplus_lib::core::{LDNApplication, GithubQueryParams};

#[get("/gov-team-members")]
pub async fn gov_team_members(query: web::Query<GithubQueryParams>) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();

    match LDNApplication::fetch_verifiers(owner, repo).await {
        Ok(notaries) => {
            Ok(HttpResponse::Ok().json(notaries))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().body(e.to_string()))
        }
    }
}


