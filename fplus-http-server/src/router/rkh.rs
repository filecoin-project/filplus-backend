use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use fplus_lib::core::LDNApplication;


#[derive(Deserialize)]
pub struct PullRequestData {
    pr_number: String,
    user_handle: String,
}

#[get("/rkh")]
pub async fn get() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

#[post("/pr")]
pub async fn receive_pr(info: web::Json<PullRequestData>) -> impl Responder {
    let pr_number = info.pr_number.trim_matches('"').parse::<u64>();
    
    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::get_by_pr_number(pr_number).await {
                Ok(application_file) => {
                    println!("Received PR data - PR Number: {}, User Handle: {}", pr_number, info.user_handle);
                    HttpResponse::Ok().json(application_file)
                },
                Err(error) => HttpResponse::InternalServerError().json(error.to_string()),
            }
        }
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}



