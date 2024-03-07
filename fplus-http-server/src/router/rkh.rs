use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::{
    ApplicationQueryParams, CompleteGovernanceReviewInfo, LDNApplication, rkh::auth::{GenerateNonceQueryParams, generate_nonce}
};

#[get("/generate-nonce")]
pub async fn fetch_nonce(query: web::Query<GenerateNonceQueryParams>) -> impl Responder {
    let GenerateNonceQueryParams { wallet, multisig_address, owner, repo } = query.into_inner();

    generate_nonce(wallet, multisig_address, owner, repo)
        .map(|nonce| HttpResponse::Ok().body(nonce))
        .unwrap_or_else(|e| HttpResponse::BadRequest().body(e.to_string())) as HttpResponse
}

#[post("/application/trigger")]
pub async fn trigger(
    query: web::Query<ApplicationQueryParams>,
    info: web::Json<CompleteGovernanceReviewInfo>,
) -> impl Responder {
    let CompleteGovernanceReviewInfo { actor} = info.into_inner();
    let ldn_application = match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    dbg!(&ldn_application);
    match ldn_application
        .complete_governance_review(actor, query.owner.clone(), query.repo.clone())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(e) => {
            return HttpResponse::BadRequest()
                .body(format!("Application is not in the correct state {}", e));
        }
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
