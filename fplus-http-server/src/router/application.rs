use actix_web::{get, post, web, HttpResponse, Responder};
use fplus_lib::core::{
    application::file::{StorageProviderChangeVerifier, VerifierInput},
    ApplicationQueryParams, BranchDeleteInfo, CompleteGovernanceReviewInfo,
    CompleteNewApplicationApprovalInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo,
    DcReachedInfo, GithubQueryParams, LDNApplication, MoreInfoNeeded, NotifyRefillInfo,
    StorageProvidersChangeApprovalInfo, StorageProvidersChangeProposalInfo, SubmitKYCInfo,
    TriggerSSAInfo, ValidationPullRequestData, VerifierActionsQueryParams,
};

#[post("/application")]
pub async fn create(info: web::Json<CreateApplicationInfo>) -> impl Responder {
    match LDNApplication::new_from_issue(info.into_inner()).await {
        Ok(app) => HttpResponse::Ok().body(format!(
            "Created new application for issue: {}",
            app.application_id.clone()
        )),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[get("/application")]
pub async fn single(query: web::Query<ApplicationQueryParams>) -> impl Responder {
    let ApplicationQueryParams { id, owner, repo } = query.into_inner();
    match LDNApplication::load_from_db(id, owner, repo).await {
        Ok(app_file) => match serde_json::to_string_pretty(&app_file) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[get("/application/with-allocation-amount")]
pub async fn application_with_allocation_amount_handler(
    query: web::Query<ApplicationQueryParams>,
) -> impl Responder {
    let ApplicationQueryParams { id, owner, repo } = query.into_inner();

    match LDNApplication::application_with_allocation_amount(id, owner, repo).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/application/trigger")]
pub async fn trigger(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<CompleteGovernanceReviewInfo>,
) -> impl Responder {
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    dbg!(&ldn_application);
    let CompleteGovernanceReviewInfo {
        allocation_amount,
        client_contract_address,
    } = info.into_inner();
    match ldn_application
        .complete_governance_review(
            query.github_username.clone(),
            query.owner.clone(),
            query.repo.clone(),
            allocation_amount,
            client_contract_address,
        )
        .await
    {
        Ok(app) => match serde_json::to_string_pretty(&app) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest()
            .body(format!("Application is not in the correct state {}", e)),
    }
}

#[post("/application/approve_changes")]
pub async fn approve_changes(query: web::Query<VerifierActionsQueryParams>) -> impl Responder {
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };

    match ldn_application
        .approve_changes(query.owner.clone(), query.repo.clone())
        .await
    {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
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
        new_allocation_amount,
    } = info.into_inner();
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    let updated_signer = VerifierInput {
        github_username: query.github_username.clone(), // Use the provided `github_username` parameter
        signing_address: signer.signing_address,
        created_at: signer.created_at,
        message_cid: signer.message_cids.message_cid,
        increase_allowance_cid: signer.message_cids.increase_allowance_cid,
    };
    match ldn_application
        .complete_new_application_proposal(
            updated_signer,
            request_id,
            query.owner.clone(),
            query.repo.clone(),
            new_allocation_amount,
        )
        .await
    {
        Ok(app) => match serde_json::to_string_pretty(&app) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/application/propose_storage_providers")]
pub async fn propose_storage_providers(
    info: web::Json<StorageProvidersChangeProposalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> impl Responder {
    let StorageProvidersChangeProposalInfo {
        signer,
        allowed_sps,
        max_deviation,
    } = info.into_inner();
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    let verifier = StorageProviderChangeVerifier {
        github_username: query.github_username.clone(),
        signing_address: signer.signing_address.clone(),
        max_deviation_cid: signer.max_deviation_cid.clone(),
        add_allowed_sps_cids: signer.allowed_sps_cids.clone(),
        remove_allowed_sps_cids: signer.removed_allowed_sps_cids.clone(),
    };

    match ldn_application
        .complete_sps_change_proposal(
            verifier,
            query.owner.clone(),
            query.repo.clone(),
            allowed_sps,
            max_deviation,
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/application/approve_storage_providers")]
pub async fn approve_storage_providers(
    info: web::Json<StorageProvidersChangeApprovalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> impl Responder {
    let StorageProvidersChangeApprovalInfo { signer, request_id } = info.into_inner();
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    let verifier = StorageProviderChangeVerifier {
        github_username: query.github_username.clone(),
        signing_address: signer.signing_address.clone(),
        max_deviation_cid: signer.max_deviation_cid.clone(),
        add_allowed_sps_cids: signer.allowed_sps_cids.clone(),
        remove_allowed_sps_cids: signer.removed_allowed_sps_cids.clone(),
    };
    match ldn_application
        .complete_sps_change_approval(
            verifier,
            query.owner.clone(),
            query.repo.clone(),
            request_id,
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/application/approve")]
pub async fn approve(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<CompleteNewApplicationApprovalInfo>,
) -> impl Responder {
    let CompleteNewApplicationApprovalInfo { signer, request_id } = info.into_inner();
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    let updated_signer = VerifierInput {
        github_username: query.github_username.clone(), // Use the provided `github_username` parameter
        signing_address: signer.signing_address,
        created_at: signer.created_at,
        message_cid: signer.message_cids.message_cid,
        increase_allowance_cid: signer.message_cids.increase_allowance_cid,
    };
    match ldn_application
        .complete_new_application_approval(
            updated_signer,
            request_id,
            query.owner.clone(),
            query.repo.clone(),
            None,
        )
        .await
    {
        Ok(app) => match serde_json::to_string_pretty(&app) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/application/decline")]
pub async fn decline(query: web::Query<VerifierActionsQueryParams>) -> impl Responder {
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    match ldn_application
        .decline_application(query.owner.clone(), query.repo.clone())
        .await
    {
        Ok(app) => match serde_json::to_string_pretty(&app) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

#[post("/application/additional_info_required")]
pub async fn additional_info_required(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<MoreInfoNeeded>,
) -> impl Responder {
    let MoreInfoNeeded { verifier_message } = info.into_inner();
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => {
                return HttpResponse::BadRequest().body(e.to_string());
            }
        };
    match ldn_application
        .additional_info_required(query.owner.clone(), query.repo.clone(), verifier_message)
        .await
    {
        Ok(app) => match serde_json::to_string_pretty(&app) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

#[get("/applications")]
pub async fn all_applications() -> impl Responder {
    match LDNApplication::all_applications().await {
        Ok(apps) => match serde_json::to_string_pretty(&apps) {
            Ok(json) => HttpResponse::Ok()
                .content_type("application/json")
                .body(json),
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Failed to serialize applications: {}", e)),
        },
        Err(errors) => match serde_json::to_string_pretty(&errors) {
            Ok(json) => HttpResponse::BadRequest()
                .content_type("application/json")
                .body(json),
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Failed to serialize errors: {}", e)),
        },
    }
}

#[get("/application/active")]
pub async fn active(query: web::Query<GithubQueryParams>) -> impl Responder {
    let GithubQueryParams { owner, repo } = query.into_inner();
    match LDNApplication::active(owner, repo, None).await {
        Ok(app) => match serde_json::to_string_pretty(&app) {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => {
                HttpResponse::InternalServerError().body("Failed to serialize success message")
            }
        },
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[get("/application/merged")]
pub async fn merged(query: web::Query<GithubQueryParams>) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();
    match LDNApplication::merged(owner, repo).await {
        Ok(apps) => match serde_json::to_string_pretty(&apps) {
            Ok(response) => Ok(HttpResponse::Ok().body(response)),
            Err(_) => {
                Ok(HttpResponse::InternalServerError().body("Failed to serialize success message"))
            }
        },
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[post("/application/notify_refill")]
pub async fn notify_refill(info: web::Json<NotifyRefillInfo>) -> impl Responder {
    match LDNApplication::notify_refill(info.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/application/totaldcreached")]
pub async fn total_dc_reached(data: web::Json<DcReachedInfo>) -> actix_web::Result<impl Responder> {
    let DcReachedInfo { id, owner, repo } = data.into_inner();
    match LDNApplication::total_dc_reached(id, owner, repo).await {
        Ok(applications) => Ok(HttpResponse::Ok().json(applications)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[post("application/flow/validate")]
pub async fn validate_application_flow(
    info: web::Json<ValidationPullRequestData>,
) -> impl Responder {
    let ValidationPullRequestData {
        pr_number,
        user_handle,
        owner,
        repo,
    } = info.into_inner();
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
    let ValidationPullRequestData {
        pr_number,
        user_handle,
        owner,
        repo,
    } = info.into_inner();
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
    let ValidationPullRequestData {
        pr_number,
        user_handle: _,
        owner,
        repo,
    } = info.into_inner();
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
    let ValidationPullRequestData {
        pr_number,
        user_handle: _,
        owner,
        repo,
    } = info.into_inner();
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
    let ValidationPullRequestData {
        pr_number,
        user_handle: _,
        owner,
        repo,
    } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::validate_merge_application(pr_number, owner, repo).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("/application/branch/delete")]
pub async fn delete_branch(data: web::Json<BranchDeleteInfo>) -> actix_web::Result<impl Responder> {
    let info = data.into_inner();
    match LDNApplication::delete_branch(info.owner, info.repo, info.branch_name).await {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[post("application/cache/renewal")]
pub async fn cache_renewal(info: web::Json<GithubQueryParams>) -> impl Responder {
    let GithubQueryParams { owner, repo } = info.into_inner();
    let active_result = LDNApplication::cache_renewal_active(owner.clone(), repo.clone()).await;

    let merged_result = LDNApplication::cache_renewal_merged(owner, repo).await;

    match (active_result, merged_result) {
        (Ok(_), Ok(_)) => {
            HttpResponse::Ok().json("Cache renewal for active and merged applications succeeded")
        }
        (Err(e), _) | (_, Err(e)) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("application/update-from-issue")]
pub async fn update_from_issue(info: web::Json<CreateApplicationInfo>) -> impl Responder {
    match LDNApplication::update_from_issue(info.into_inner()).await {
        Ok(app) => HttpResponse::Ok().body(format!(
            "Updated application for issue: {}",
            app.application_id.clone()
        )),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("application/check_for_changes")]
pub async fn check_for_changes(info: web::Json<ValidationPullRequestData>) -> impl Responder {
    let ValidationPullRequestData {
        pr_number,
        user_handle,
        owner,
        repo,
    } = info.into_inner();
    let pr_number = pr_number.trim_matches('"').parse::<u64>();

    match pr_number {
        Ok(pr_number) => {
            match LDNApplication::check_for_changes(pr_number, &user_handle, owner, repo).await {
                Ok(result) => HttpResponse::Ok().json(result),
                Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
            }
        }
        Err(_) => HttpResponse::BadRequest().json("Invalid PR Number"),
    }
}

#[post("application/submit_kyc")]
pub async fn submit_kyc(info: web::Json<SubmitKYCInfo>) -> impl Responder {
    let ldn_application = match LDNApplication::load(
        info.message.client_id.clone(),
        info.message.allocator_repo_owner.clone(),
        info.message.allocator_repo_name.clone(),
    )
    .await
    {
        Ok(app) => app,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };
    match ldn_application.submit_kyc(&info.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Address verified with score")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[post("application/request_kyc")]
pub async fn request_kyc(query: web::Query<VerifierActionsQueryParams>) -> impl Responder {
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
        };
    match ldn_application
        .request_kyc(&query.id, &query.owner, &query.repo)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("application/trigger_ssa")]
pub async fn trigger_ssa(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<TriggerSSAInfo>,
) -> impl Responder {
    match LDNApplication::trigger_ssa(
        &query.id,
        &query.owner,
        &query.repo,
        &query.github_username,
        info.into_inner(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("application/remove_pending_allocation")]
pub async fn remove_pending_allocation(
    query: web::Query<VerifierActionsQueryParams>,
) -> impl Responder {
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
        };
    match ldn_application
        .remove_pending_allocation(&query.id, &query.owner, &query.repo)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("application/allocation_failed")]
pub async fn allocation_failed(query: web::Query<VerifierActionsQueryParams>) -> impl Responder {
    let ldn_application =
        match LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone()).await
        {
            Ok(app) => app,
            Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
        };
    match ldn_application
        .revert_to_ready_to_sign(&query.id, &query.owner, &query.repo)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(
            serde_json::to_string_pretty("Success")
                .expect("Serialization of static string should succeed"),
        ),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
