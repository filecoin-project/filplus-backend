use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound},
    get, post, web, HttpResponse, Responder,
};

use fplus_lib::core::{
    application::file::{
        DecreaseClientAllowanceVerifier, StorageProviderChangeVerifier, VerifierInput,
    },
    ApplicationQueryParams, BranchDeleteInfo, CompleteGovernanceReviewInfo,
    CompleteNewApplicationApprovalInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo,
    DcReachedInfo, DecreaseAllowanceApprovalInfo, DecreaseAllowanceProposalInfo,
    GetApplicationsByClientContractAddressQueryParams, GithubQueryParams, LDNApplication,
    MoreInfoNeeded, NotifyRefillInfo, StorageProvidersChangeApprovalInfo,
    StorageProvidersChangeProposalInfo, SubmitKYCInfo, TriggerSSAInfo, ValidationPullRequestData,
    VerifierActionsQueryParams,
};

#[post("/application")]
pub async fn create(info: web::Json<CreateApplicationInfo>) -> actix_web::Result<impl Responder> {
    let app = LDNApplication::new_from_issue(info.into_inner())
        .await
        .map_err(ErrorBadRequest)?;
    Ok(HttpResponse::Ok().body(format!(
        "Created new application for issue: {}",
        app.application_id.clone()
    )))
}

#[get("/application")]
pub async fn single(
    query: web::Query<ApplicationQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ApplicationQueryParams { id, owner, repo } = query.into_inner();
    let app_file = LDNApplication::load_from_db(id, owner, repo)
        .await
        .map_err(ErrorNotFound)?;
    let body = serde_json::to_string_pretty(&app_file).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(body))
}

#[get("/applications/closed")]
pub async fn closed_applications() -> actix_web::Result<impl Responder> {
    let apps = LDNApplication::get_closed_applications()
        .await
        .map_err(ErrorInternalServerError)?;

    let parsed = serde_json::to_string_pretty(&apps).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(parsed))
}

#[get("/applications/by_contract_address")]
pub async fn get_applications_by_contract_address(
    query: web::Query<GetApplicationsByClientContractAddressQueryParams>,
) -> actix_web::Result<impl Responder> {
    let applications =
        LDNApplication::get_applications_by_client_contract_address(&query.client_contract_address)
            .await
            .map_err(ErrorNotFound)?;
    let parsed = serde_json::to_string_pretty(&applications).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(parsed))
}

#[get("/applications/closed/allocator")]
pub async fn closed_allocator_applications(
    query: web::Query<GithubQueryParams>,
) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();
    let apps = LDNApplication::get_allocator_closed_applications(&owner, &repo)
        .await
        .map_err(ErrorInternalServerError)?;

    let parsed = serde_json::to_string_pretty(&apps).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(parsed))
}

#[get("/application/with-allocation-amount")]
pub async fn application_with_allocation_amount_handler(
    query: web::Query<ApplicationQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ApplicationQueryParams { id, owner, repo } = query.into_inner();
    let application = LDNApplication::application_with_allocation_amount(id, owner, repo)
        .await
        .map_err(ErrorNotFound)?;
    Ok(HttpResponse::Ok().json(application))
}

#[post("/application/trigger")]
pub async fn trigger(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<CompleteGovernanceReviewInfo>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;

    dbg!(&ldn_application);
    let CompleteGovernanceReviewInfo {
        allocation_amount,
        client_contract_address,
        reason_for_not_using_client_smart_contract,
    } = info.into_inner();
    let response = ldn_application
        .complete_governance_review(
            query.github_username.clone(),
            query.owner.clone(),
            query.repo.clone(),
            allocation_amount,
            client_contract_address,
            reason_for_not_using_client_smart_contract,
        )
        .await
        .map_err(ErrorBadRequest)?;

    let serialized_app = serde_json::to_string_pretty(&response)
        .map_err(|_| ErrorInternalServerError("Failed to serialize success message".to_string()))?;

    Ok(HttpResponse::Ok().body(serialized_app))
}

#[post("/application/approve_changes")]
pub async fn approve_changes(
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;

    let response = ldn_application
        .approve_changes(query.owner.clone(), query.repo.clone())
        .await
        .map_err(ErrorNotFound)?;

    let serialized_app = serde_json::to_string_pretty(&response)
        .map_err(|_| ErrorInternalServerError("Failed to serialize success message".to_string()))?;
    Ok(HttpResponse::Ok().body(serialized_app))
}

#[post("/application/propose")]
pub async fn propose(
    info: web::Json<CompleteNewApplicationProposalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let CompleteNewApplicationProposalInfo {
        signer,
        request_id,
        new_allocation_amount,
        amount_of_datacap_sent_to_contract,
    } = info.into_inner();
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let updated_signer = VerifierInput {
        github_username: query.github_username.clone(), // Use the provided `github_username` parameter
        signing_address: signer.signing_address,
        created_at: signer.created_at,
        message_cid: signer.message_cids.message_cid,
        increase_allowance_cid: signer.message_cids.increase_allowance_cid,
    };
    let response = ldn_application
        .complete_new_application_proposal(
            updated_signer,
            request_id,
            query.owner.clone(),
            query.repo.clone(),
            new_allocation_amount,
            amount_of_datacap_sent_to_contract,
        )
        .await
        .map_err(ErrorInternalServerError)?;
    let serialized_app = serde_json::to_string_pretty(&response)
        .map_err(|_| ErrorInternalServerError("Failed to serialize success message".to_string()))?;
    Ok(HttpResponse::Ok().body(serialized_app))
}

#[post("/application/propose_storage_providers")]
pub async fn propose_storage_providers(
    info: web::Json<StorageProvidersChangeProposalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let StorageProvidersChangeProposalInfo {
        signer,
        allowed_sps,
        max_deviation,
    } = info.into_inner();
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let verifier = StorageProviderChangeVerifier {
        github_username: query.github_username.clone(),
        signing_address: signer.signing_address.clone(),
        max_deviation_cid: signer.max_deviation_cid.clone(),
        add_allowed_sps_cids: signer.allowed_sps_cids.clone(),
        remove_allowed_sps_cids: signer.removed_allowed_sps_cids.clone(),
    };

    ldn_application
        .complete_sps_change_proposal(
            verifier,
            query.owner.clone(),
            query.repo.clone(),
            allowed_sps,
            max_deviation,
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("/application/approve_storage_providers")]
pub async fn approve_storage_providers(
    info: web::Json<StorageProvidersChangeApprovalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let StorageProvidersChangeApprovalInfo { signer, request_id } = info.into_inner();
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let verifier = StorageProviderChangeVerifier {
        github_username: query.github_username.clone(),
        signing_address: signer.signing_address.clone(),
        max_deviation_cid: signer.max_deviation_cid.clone(),
        add_allowed_sps_cids: signer.allowed_sps_cids.clone(),
        remove_allowed_sps_cids: signer.removed_allowed_sps_cids.clone(),
    };
    ldn_application
        .complete_sps_change_approval(
            verifier,
            query.owner.clone(),
            query.repo.clone(),
            request_id,
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("/application/propose_decrease_allowance")]
pub async fn propose_decrease_allowance(
    info: web::Json<DecreaseAllowanceProposalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let verifier = DecreaseClientAllowanceVerifier {
        github_username: query.github_username.clone(),
        signing_address: info.signer.signing_address.clone(),
        decrease_allowance_cid: info.signer.decrease_allowance_cid.clone(),
    };

    ldn_application
        .propose_decrease_allowance(
            &verifier,
            &query.owner,
            &query.repo,
            &info.amount_to_decrease,
            &info.reason_for_decrease,
        )
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("/application/approve_decrease_allowance")]
pub async fn approve_decrease_allowance(
    info: web::Json<DecreaseAllowanceApprovalInfo>,
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let verifier = DecreaseClientAllowanceVerifier {
        github_username: query.github_username.clone(),
        signing_address: info.signer.signing_address.clone(),
        decrease_allowance_cid: info.signer.decrease_allowance_cid.clone(),
    };

    ldn_application
        .approve_decrease_allowance(&verifier, &query.owner, &query.repo, &info.request_id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("/application/approve")]
pub async fn approve(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<CompleteNewApplicationApprovalInfo>,
) -> actix_web::Result<impl Responder> {
    let CompleteNewApplicationApprovalInfo { signer, request_id } = info.into_inner();
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let updated_signer = VerifierInput {
        github_username: query.github_username.clone(), // Use the provided `github_username` parameter
        signing_address: signer.signing_address,
        created_at: signer.created_at,
        message_cid: signer.message_cids.message_cid,
        increase_allowance_cid: signer.message_cids.increase_allowance_cid,
    };
    let app = ldn_application
        .complete_new_application_approval(
            updated_signer,
            request_id,
            query.owner.clone(),
            query.repo.clone(),
            None,
            None,
        )
        .await
        .map_err(ErrorInternalServerError)?;
    let serialized_app = serde_json::to_string_pretty(&app).map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(serialized_app))
}

#[post("/application/decline")]
pub async fn decline(
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    ldn_application
        .decline_application(query.owner.clone(), query.repo.clone())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(()))
}

#[post("/application/reopen_declined_application")]
pub async fn reopen_declined_application(
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    LDNApplication::reopen_declined_application(
        &query.owner,
        &query.repo,
        &query.github_username,
        &query.id,
    )
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(()))
}

#[post("/application/additional_info_required")]
pub async fn additional_info_required(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<MoreInfoNeeded>,
) -> actix_web::Result<impl Responder> {
    let MoreInfoNeeded { verifier_message } = info.into_inner();
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    let app = ldn_application
        .additional_info_required(query.owner.clone(), query.repo.clone(), verifier_message)
        .await
        .map_err(ErrorInternalServerError)?;
    let serialized_app = serde_json::to_string_pretty(&app).map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(serialized_app))
}

#[get("/applications/active")]
pub async fn all_applications() -> actix_web::Result<impl Responder> {
    let apps = LDNApplication::all_applications()
        .await
        .map_err(ErrorNotFound)?;

    let parsed = serde_json::to_string_pretty(&apps).map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(parsed))
}

#[get("/applications/open_pull_request")]
pub async fn active(query: web::Query<GithubQueryParams>) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();
    let app = LDNApplication::active(owner, repo, None)
        .await
        .map_err(ErrorInternalServerError)?;
    let serialized_app = serde_json::to_string_pretty(&app).map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(serialized_app))
}

#[get("/application/merged")]
pub async fn merged(query: web::Query<GithubQueryParams>) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = query.into_inner();
    let apps = LDNApplication::merged(owner, repo)
        .await
        .map_err(ErrorInternalServerError)?;
    let serialized_apps = serde_json::to_string_pretty(&apps).map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(serialized_apps))
}

#[post("/application/notify_refill")]
pub async fn notify_refill(info: web::Json<NotifyRefillInfo>) -> actix_web::Result<impl Responder> {
    LDNApplication::notify_refill(info.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("/application/totaldcreached")]
pub async fn total_dc_reached(data: web::Json<DcReachedInfo>) -> actix_web::Result<impl Responder> {
    let DcReachedInfo { id, owner, repo } = data.into_inner();
    let ldn_application = LDNApplication::load(id.clone(), owner.clone(), repo.clone())
        .await
        .map_err(ErrorNotFound)?;
    let applications = ldn_application
        .total_dc_reached()
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(applications))
}

#[post("application/flow/validate")]
pub async fn validate_application_flow(
    info: web::Json<ValidationPullRequestData>,
) -> actix_web::Result<impl Responder> {
    let ValidationPullRequestData {
        pr_number,
        user_handle,
        owner,
        repo,
    } = info.into_inner();
    if let Ok(pr_number) = pr_number.trim_matches('"').parse::<u64>() {
        let result = LDNApplication::validate_flow(pr_number, &user_handle, owner, repo)
            .await
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(result))
    } else {
        Err(ErrorBadRequest("Invalid PR Number"))
    }
}

#[post("application/trigger/validate")]
pub async fn validate_application_trigger(
    info: web::Json<ValidationPullRequestData>,
) -> actix_web::Result<impl Responder> {
    let ValidationPullRequestData {
        pr_number,
        user_handle,
        owner,
        repo,
    } = info.into_inner();

    if let Ok(pr_number) = pr_number.trim_matches('"').parse::<u64>() {
        let result = LDNApplication::validate_trigger(pr_number, &user_handle, owner, repo)
            .await
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(result))
    } else {
        Err(ErrorBadRequest("Invalid PR Number"))
    }
}

#[post("application/proposal/validate")]
pub async fn validate_application_proposal(
    info: web::Json<ValidationPullRequestData>,
) -> actix_web::Result<impl Responder> {
    let ValidationPullRequestData {
        pr_number,
        user_handle: _,
        owner,
        repo,
    } = info.into_inner();

    if let Ok(pr_number) = pr_number.trim_matches('"').parse::<u64>() {
        let result = LDNApplication::validate_proposal(pr_number, owner, repo)
            .await
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(result))
    } else {
        Err(ErrorBadRequest("Invalid PR Number"))
    }
}

#[post("application/approval/validate")]
pub async fn validate_application_approval(
    info: web::Json<ValidationPullRequestData>,
) -> actix_web::Result<impl Responder> {
    let ValidationPullRequestData {
        pr_number,
        user_handle: _,
        owner,
        repo,
    } = info.into_inner();

    if let Ok(pr_number) = pr_number.trim_matches('"').parse::<u64>() {
        let result = LDNApplication::validate_approval(pr_number, owner, repo)
            .await
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(result))
    } else {
        Err(ErrorBadRequest("Invalid PR Number"))
    }
}

#[post("application/merge/validate")]
pub async fn validate_application_merge(
    info: web::Json<ValidationPullRequestData>,
) -> actix_web::Result<impl Responder> {
    let ValidationPullRequestData {
        pr_number,
        user_handle: _,
        owner,
        repo,
    } = info.into_inner();

    if let Ok(pr_number) = pr_number.trim_matches('"').parse::<u64>() {
        let result = LDNApplication::validate_merge_application(pr_number, owner, repo)
            .await
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(result))
    } else {
        Err(ErrorBadRequest("Invalid PR Number"))
    }
}

#[post("/application/branch/delete")]
pub async fn delete_branch(data: web::Json<BranchDeleteInfo>) -> actix_web::Result<impl Responder> {
    let info = data.into_inner();
    let result = LDNApplication::delete_branch(info.owner, info.repo, info.branch_name)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(result))
}

#[post("application/cache/renewal")]
pub async fn cache_renewal(
    info: web::Json<GithubQueryParams>,
) -> actix_web::Result<impl Responder> {
    let GithubQueryParams { owner, repo } = info.into_inner();
    LDNApplication::cache_renewal_active(owner.clone(), repo.clone())
        .await
        .map_err(ErrorInternalServerError)?;

    LDNApplication::cache_renewal_merged(owner, repo)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json("Cache renewal for active and merged applications succeeded"))
}

#[post("application/update-from-issue")]
pub async fn update_from_issue(
    info: web::Json<CreateApplicationInfo>,
) -> actix_web::Result<impl Responder> {
    let app = LDNApplication::update_from_issue(info.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(format!(
        "Updated application for issue: {}",
        app.application_id.clone()
    )))
}

#[post("application/check_for_changes")]
pub async fn check_for_changes(
    info: web::Json<ValidationPullRequestData>,
) -> actix_web::Result<impl Responder> {
    let ValidationPullRequestData {
        pr_number,
        user_handle,
        owner,
        repo,
    } = info.into_inner();

    if let Ok(pr_number) = pr_number.trim_matches('"').parse::<u64>() {
        let result = LDNApplication::check_for_changes(pr_number, &user_handle, owner, repo)
            .await
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(result))
    } else {
        Err(ErrorBadRequest("Invalid PR Number"))
    }
}

#[post("application/submit_kyc")]
pub async fn submit_kyc(info: web::Json<SubmitKYCInfo>) -> actix_web::Result<impl Responder> {
    let ldn_application = LDNApplication::load(
        info.message.client_id.clone(),
        info.message.allocator_repo_owner.clone(),
        info.message.allocator_repo_name.clone(),
    )
    .await
    .map_err(ErrorNotFound)?;

    ldn_application
        .submit_kyc(&info.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Address verified with score")
            .expect("Serialization of static string should succeed"),
    ))
}

#[get("/health")]
pub async fn health() -> actix_web::Result<impl Responder> {
    Ok(HttpResponse::Ok().body("OK"))
}

#[post("application/request_kyc")]
pub async fn request_kyc(
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    ldn_application
        .request_kyc(&query.id, &query.owner, &query.repo)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("application/trigger_ssa")]
pub async fn trigger_ssa(
    query: web::Query<VerifierActionsQueryParams>,
    info: web::Json<TriggerSSAInfo>,
) -> actix_web::Result<impl Responder> {
    LDNApplication::trigger_ssa(
        &query.id,
        &query.owner,
        &query.repo,
        &query.github_username,
        info.into_inner(),
    )
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("application/remove_pending_allocation")]
pub async fn remove_pending_allocation(
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    ldn_application
        .remove_pending_allocation(&query.id, &query.owner, &query.repo)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}

#[post("application/allocation_failed")]
pub async fn allocation_failed(
    query: web::Query<VerifierActionsQueryParams>,
) -> actix_web::Result<impl Responder> {
    let ldn_application =
        LDNApplication::load(query.id.clone(), query.owner.clone(), query.repo.clone())
            .await
            .map_err(ErrorNotFound)?;
    ldn_application
        .revert_to_ready_to_sign(&query.id, &query.owner, &query.repo)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("Success")
            .expect("Serialization of static string should succeed"),
    ))
}
