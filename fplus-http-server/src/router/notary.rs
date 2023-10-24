use std::env;

use actix_web::{get, post, HttpResponse, web};
use reqwest;
use serde::{Deserialize, Serialize};

#[get("/notary")]
pub async fn get() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

#[post("/notary")]
pub async fn post() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

#[derive(Serialize, Deserialize)]
struct RemoveDatacapForClientRequest {
    datacap_to_remove: u64,
    client: String,
}

#[derive(Serialize, Deserialize)]
struct RemoveDatacapForClientResponse {
    signature1: String,
    signature2: String,
}

#[post("/notary/sign-datacap-removal")]
async fn sign_datacap_removal(
    json: web::Json<RemoveDatacapForClientRequest>,
) -> HttpResponse {
    let request_body = json.into_inner();
    
    let url = env::var("SIGNER_URL").unwrap_or_else(|_| "http://localhost:3000/notary/sign-datacap-removal".to_string());
    let client = reqwest::Client::new();
    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<RemoveDatacapForClientResponse>().await {
                    Ok(response_body) => {
                        HttpResponse::Ok().json(response_body)
                    }
                    Err(err) => {
                        eprintln!("Failed to parse response: {:?}", err);
                        HttpResponse::InternalServerError().finish()
                    }
                }
            } else {
                eprintln!("Failed to remove datacap for client: {:?}", response.text().await);
                HttpResponse::InternalServerError().finish()
            }
        }
        Err(err) => {
            eprintln!("Failed to send request: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
