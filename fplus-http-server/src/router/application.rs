use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::{
    CompleteGovernanceReviewInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo,
    LDNApplication, RefillInfo, ValidationPullRequestData, ValidationIssueData
};

#[post("/application")]
pub async fn create(info: web::Json<CreateApplicationInfo>) -> impl Responder {
    match LDNApplication::new_from_issue(info.into_inner()).await {
        Ok(app) => HttpResponse::Ok().body(format!(
            "Created new application for issue: {}",
            app.application_id.clone()
        )),
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    }
}

#[get("/application/{id}")]
pub async fn single(id: web::Path<String>) -> impl Responder {
    let app = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    if let Ok(app_file) = app.file().await {
        HttpResponse::Ok().body(serde_json::to_string_pretty(&app_file).unwrap())
    } else {
        HttpResponse::BadRequest().body("Application not found")
    }
}

#[post("/application/{id}/trigger")]
pub async fn trigger(
    id: web::Path<String>,
    info: web::Json<CompleteGovernanceReviewInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    dbg!(&ldn_application);
    match ldn_application
        .complete_governance_review(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(e) => {
            return HttpResponse::BadRequest()
                .body(format!("Application is not in the correct state {}", e));
        }
    }
}

#[post("/application/{id}/propose")]
pub async fn propose(
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
pub async fn approve(
    id: web::Path<String>,
    info: web::Json<CompleteNewApplicationProposalInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    dbg!(&ldn_application);
    match ldn_application
        .complete_new_application_approval(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

#[get("/application/active")]
pub async fn active() -> impl Responder {
    let apps = match LDNApplication::active(None).await {
        Ok(app) => app,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    HttpResponse::Ok().body(serde_json::to_string_pretty(&apps).unwrap())
}

#[get("/application/merged")]
pub async fn merged() -> actix_web::Result<impl Responder> {
    match LDNApplication::merged().await {
        Ok(apps) => Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&apps).unwrap())),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().body(e.to_string()));
        }
    }
}

#[post("/application/{id}/refill")]
pub async fn refill(data: web::Json<RefillInfo>) -> actix_web::Result<impl Responder> {
    match LDNApplication::refill(data.into_inner()).await {
        Ok(applications) => Ok(HttpResponse::Ok().json(applications)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[post("/application/{id}/totaldcreached")]
pub async fn total_dc_reached(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    match LDNApplication::total_dc_reached(id.into_inner()).await {
        Ok(applications) => Ok(HttpResponse::Ok().json(applications)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[post("application/trigger/validate")]
pub async fn validate_application_trigger(info: web::Json<ValidationPullRequestData>) -> impl Responder {
    let pr_number = info.pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::validate_trigger(pr_number, &info.user_handle).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/proposal/validate")]
pub async fn validate_application_proposal(info: web::Json<ValidationPullRequestData>) -> impl Responder {
    let pr_number = info.pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::validate_proposal(pr_number).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/approval/validate")]
pub async fn validate_application_approval(info: web::Json<ValidationPullRequestData>) -> impl Responder {
    let pr_number = info.pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::validate_approval(pr_number).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
