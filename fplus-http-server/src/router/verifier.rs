use actix_web::{error::ErrorInternalServerError, get, web, HttpResponse, Responder};
use fplus_lib::core::{GithubQueryParams, LDNApplication};

#[get("/verifiers")]
pub async fn verifiers(query: web::Query<GithubQueryParams>) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();

    let notaries = LDNApplication::fetch_verifiers(owner, repo)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(notaries))
}
