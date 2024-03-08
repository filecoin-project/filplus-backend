use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::rkh::auth::{GenerateNonceQueryParams, TestSignaturePayload, generate_nonce, verify_signature};
use serde_json::json;

#[get("/nonce")]
pub async fn fetch_nonce(query: web::Query<GenerateNonceQueryParams>) -> impl Responder {
    let GenerateNonceQueryParams { wallet_address, multisig_address } = query.into_inner();

    match generate_nonce(wallet_address, multisig_address)
        .await {
            Ok(nonce) => HttpResponse::Ok().json(json!({ "nonce": nonce.to_string() })),
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        }
}

#[post("/test-signature")]
pub async fn test_signature(data: web::Json<TestSignaturePayload>) -> impl Responder {
    let TestSignaturePayload {wallet_address, signature} = data.into_inner();

    match verify_signature(&wallet_address, &signature)
        .await {
            Ok(is_valid) => HttpResponse::Ok().json(json!({ "isValid": is_valid })),
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
