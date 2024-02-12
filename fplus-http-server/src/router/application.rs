use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::{
    CompleteGovernanceReviewInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo,
    LDNApplication, RefillInfo, DcReachedInfo, ValidationPullRequestData, GithubQueryParams, ApplicationQueryParams
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

#[get("/application")]
pub async fn single(query: web::Query<ApplicationQueryParams>) -> impl Responder {
    let ApplicationQueryParams { id, owner, repo } = query.into_inner();

    let app = match LDNApplication::load(id, owner, repo).await {
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

    let CompleteGovernanceReviewInfo { actor, owner, repo } = info.into_inner();
    let ldn_application = match LDNApplication::load(id.into_inner(), owner.clone(), repo.clone()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    dbg!(&ldn_application);
    match ldn_application
        .complete_governance_review(actor, owner, repo)
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
    let CompleteNewApplicationProposalInfo {
        signer,
        request_id, 
        owner, 
        repo
    } = info.into_inner();
    let ldn_application = match LDNApplication::load(id.into_inner(), owner.clone(), repo.clone()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_new_application_proposal(signer, request_id, owner, repo)
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
    let CompleteNewApplicationProposalInfo {
        signer,
        request_id, 
        owner, 
        repo
    } = info.into_inner();
    let ldn_application = match LDNApplication::load(id.into_inner(), owner.clone(), repo.clone()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    dbg!(&ldn_application);
    match ldn_application
        .complete_new_application_approval(signer, request_id, owner, repo)
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

#[get("/application/active")]
pub async fn active(query: web::Query<GithubQueryParams>) -> impl Responder {
    let GithubQueryParams { owner, repo } = query.into_inner();
    let apps = match LDNApplication::active(owner, repo, None).await {
        Ok(app) => app,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    HttpResponse::Ok().body(serde_json::to_string_pretty(&apps).unwrap())
}

#[get("/application/merged")]
pub async fn merged(query: web::Query<GithubQueryParams>) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();
    match LDNApplication::merged(owner, repo).await {
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
pub async fn total_dc_reached(id: web::Path<String>, data: web::Json<DcReachedInfo>) -> actix_web::Result<impl Responder> {
    let DcReachedInfo {owner, repo} = data.into_inner();
    match LDNApplication::total_dc_reached(id.into_inner(), owner, repo).await {
        Ok(applications) => Ok(HttpResponse::Ok().json(applications)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[post("application/flow/validate")]
pub async fn validate_application_flow(
    info: web::Json<ValidationPullRequestData>,
) -> impl Responder {
    let ValidationPullRequestData { pr_number, user_handle, owner, repo } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();
    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::validate_flow(pr_number, &user_handle, owner, repo).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/trigger/validate")]
pub async fn validate_application_trigger(
    info: web::Json<ValidationPullRequestData>,
) -> impl Responder {
    let ValidationPullRequestData { pr_number, user_handle, owner, repo } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::validate_trigger(pr_number, &user_handle, owner, repo).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/proposal/validate")]
pub async fn validate_application_proposal(
    info: web::Json<ValidationPullRequestData>,
) -> impl Responder {
    let ValidationPullRequestData { pr_number, user_handle, owner, repo } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => match LDNApplication::validate_proposal(pr_number, owner, repo).await {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/approval/validate")]
pub async fn validate_application_approval(
    info: web::Json<ValidationPullRequestData>,
) -> impl Responder {
    let ValidationPullRequestData { pr_number, user_handle, owner, repo } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => match LDNApplication::validate_approval(pr_number, owner, repo).await {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
