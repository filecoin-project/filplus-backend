use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::{
    application::file::VerifierInput, ApplicationQueryParams, CompleteGovernanceReviewInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo, DcReachedInfo, GithubQueryParams, LDNApplication, RefillInfo, ValidationPullRequestData, VerifierActionsQueryParams
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

    match LDNApplication::load_from_db(id, owner, repo).await {
        Ok(app_file) => return HttpResponse::Ok().body(serde_json::to_string_pretty(&app_file).unwrap()),
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
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

#[post("/application/propose")]
pub async fn propose(
    info: web::Json<CompleteNewApplicationProposalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> impl Responder {
    let CompleteNewApplicationProposalInfo {
        signer,
        request_id, 
    } = info.into_inner();
    let ldn_application = match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    let updated_signer = VerifierInput {
        github_username: query.github_username.clone(), // Use the provided `github_username` parameter
        signing_address: signer.signing_address,
        created_at: signer.created_at,
        message_cid: signer.message_cid,
    };
    match ldn_application
        .complete_new_application_proposal(updated_signer, request_id, query.owner.clone(), query.repo.clone())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => {
            return HttpResponse::BadRequest().body("Application is not in the correct state");
        }
    }
}

#[post("/application/approve")]
pub async fn approve(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<CompleteNewApplicationProposalInfo>,
) -> impl Responder {
    let CompleteNewApplicationProposalInfo {
        signer,
        request_id, 
    } = info.into_inner();
    let ldn_application = match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    let updated_signer = VerifierInput {
        github_username: query.github_username.clone(), // Use the provided `github_username` parameter
        signing_address: signer.signing_address,
        created_at: signer.created_at,
        message_cid: signer.message_cid,
    };
    match ldn_application
        .complete_new_application_approval(updated_signer, request_id, query.owner.clone(), query.repo.clone())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

#[get("/applications")]
pub async fn all_applications() -> impl Responder {
    match LDNApplication::all_applications().await {
        Ok(apps) => {
            match serde_json::to_string_pretty(&apps) {
                Ok(json) => HttpResponse::Ok().content_type("application/json").body(json),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to serialize applications: {}", e)),
            }
        },
        Err(errors) => {
            match serde_json::to_string_pretty(&errors) {
                Ok(json) => HttpResponse::BadRequest().content_type("application/json").body(json),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to serialize errors: {}", e)),
            }
        },
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

#[post("/application/refill")]
pub async fn refill(data: web::Json<RefillInfo>) -> actix_web::Result<impl Responder> {
    match LDNApplication::refill(data.into_inner()).await {
        Ok(applications) => Ok(HttpResponse::Ok().json(applications)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[post("/application/totaldcreached")]
pub async fn total_dc_reached(data: web::Json<DcReachedInfo>) -> actix_web::Result<impl Responder> {
    let DcReachedInfo {id, owner, repo} = data.into_inner();
    match LDNApplication::total_dc_reached(id, owner, repo).await {
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
    let ValidationPullRequestData { pr_number, user_handle: _, owner, repo } = info.into_inner();
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
    let ValidationPullRequestData { pr_number, user_handle: _, owner, repo } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => match LDNApplication::validate_approval(pr_number, owner, repo).await {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/merge/validate")]
pub async fn validate_application_merge(
    info: web::Json<ValidationPullRequestData>,
) -> impl Responder {
    let ValidationPullRequestData { pr_number, user_handle: _, owner, repo } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => match LDNApplication::validate_merge_application(pr_number, owner, repo).await {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/cache/renewal")]
pub async fn cache_renewal(
    info: web::Json<GithubQueryParams>,
) -> impl Responder {
    let GithubQueryParams { owner, repo } = info.into_inner();
    let active_result = LDNApplication::cache_renewal_active(owner.clone(), repo.clone()).await;

    let merged_result = LDNApplication::cache_renewal_merged(owner, repo).await;

    match (active_result, merged_result) {
        (Ok(_), Ok(_)) => HttpResponse::Ok().json("Cache renewal for active and merged applications succeeded"),
        (Err(e), _) | (_, Err(e)) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
