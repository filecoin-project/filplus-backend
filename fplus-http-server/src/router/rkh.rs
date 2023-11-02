use actix_web::{get, HttpResponse,};


#[get("/rkh")]
pub async fn get() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}




