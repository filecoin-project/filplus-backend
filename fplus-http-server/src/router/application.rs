use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::{
    CompleteGovernanceReviewInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo,
    LDNApplication, RemoveDatacapRequest,
};
use reqwest::StatusCode;

#[post("/application")]
pub async fn create_application(info: web::Json<CreateApplicationInfo>) -> impl Responder {
    match LDNApplication::new(info.into_inner()).await {
        Ok(app) => HttpResponse::Ok().body(format!(
            "Created new application for issue: {}",
            app.application_id.clone()
        )),
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    }
}

#[post("/application/{id}/trigger")]
pub async fn trigger_application(
    id: web::Path<String>,
    info: web::Json<CompleteGovernanceReviewInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_governance_review(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => {
            return HttpResponse::BadRequest().body("Application is not in the correct state");
        }
    }
}

#[post("/application/{id}/propose")]
pub async fn propose_application(
    id: web::Path<String>,
    info: web::Json<CompleteNewApplicationProposalInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_new_application_proposal(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => {
            return HttpResponse::BadRequest().body("Application is not in the correct state");
        }
    }
}

#[post("/application/{id}/approve")]
pub async fn approve_application(
    id: web::Path<String>,
    info: web::Json<CompleteNewApplicationProposalInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_new_application_approval(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

#[get("/application/{id}")]
pub async fn get_application(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let app = match LDNApplication::app_file_without_load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().body(e.to_string()));
        }
    };
    Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()))
}

#[get("/application")]
pub async fn get_all_applications() -> actix_web::Result<impl Responder> {
    let apps = match LDNApplication::get_all_active_applications().await {
        Ok(app) => app,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().body(e.to_string()));
        }
    };
    Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&apps).unwrap()))
}

#[get("/applications/merged")]
pub async fn get_merged_applications() -> actix_web::Result<impl Responder> {
    match LDNApplication::get_merged_applications().await {
        Ok(apps) => Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&apps).unwrap())),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().body(e.to_string()));
        }
    }
}

#[post("/application/{id}/remove")]
pub async fn remove(
    id: web::Path<String>,
    info: web::Json<RemoveDatacapRequest>,
) -> actix_web::Result<HttpResponse> {
    match LDNApplication::remove_datacap(id.into_inner(), info.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok().status(StatusCode::OK).finish()),
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(e.to_string()))
        }
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
