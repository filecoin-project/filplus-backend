use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use fplus_database::database::allocators::{self as allocators_db};
use fplus_lib::{
    core::{
        allocator::{
            create_allocator_repo, create_issue_for_multisig_change, is_allocator_repo_created, process_allocator_file, update_single_installation_id_logic
        },
        AllocatorUpdateInfo, ChangedAllocators, InstallationIdUpdateInfo,
    },
    external_services::filecoin::get_multisig_signers_for_msig,
};

/**
 * Get all allocators
 *
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[get("/allocators")]
pub async fn allocators() -> impl Responder {
    let allocators = allocators_db::get_allocators().await;
    match allocators {
        Ok(allocators) => HttpResponse::Ok().json(allocators),
        Err(e) => {
            log::error!("Failed to fetch allocators: {}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}
/**
 * Creates new Allocator in the db from a JSON file in the repository
 *
 * # Arguments
 * @param files: web::Json<ChangedAllocators> - The list of changed JSON file names
 *
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[post("/allocator/create")]
pub async fn create_from_json(
    files: web::Json<ChangedAllocators>,
) -> actix_web::Result<impl Responder> {
    println!("Files: {:?}", files);
    let mut error_response: Option<HttpResponse> = None;

    for file_name in &files.files_changed {
        log::info!("Starting allocator creation on: {}", file_name);

        match process_allocator_file(file_name).await {
            Ok(model) => {
                if model.pathway_addresses.msig.is_empty() {
                    error_response = Some(
                        HttpResponse::BadRequest().body("Missing or invalid multisig_address"),
                    );
                    break;
                }
                let verifiers_gh_handles = if model.application.verifiers_gh_handles.is_empty() {
                    None
                } else {
                    Some(model.application.verifiers_gh_handles.join(", "))
                };

                let signers = if model.pathway_addresses.signer.is_empty() {
                    None
                } else {
                    Some(model.pathway_addresses.signer.join(", "))
                };

                let model_signers = model.pathway_addresses.signer.clone();
                let owner = model.owner.clone().unwrap_or_default().to_string();
                let repo = model.repo.clone().unwrap_or_default().to_string();

                match get_multisig_signers_for_msig(&model.pathway_addresses.msig).await {
                    Ok(blockchain_signers) => {
                        if !blockchain_signers.iter().all(|s| model_signers.contains(s)) {
                            // Creating an issue instead of commenting
                            if let Err(err) = create_issue_for_multisig_change(
                                owner.clone(),
                                repo.clone(),
                                &model.pathway_addresses.msig,
                                model_signers,
                                blockchain_signers,
                            )
                            .await {
                                log::error!("Failed to create issue: {}", err);
                            }
                            error_response = Some(HttpResponse::BadRequest().body("Signer mismatch"));
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Error fetching signers from blockchain: {}", e);
                        error_response =
                            Some(HttpResponse::InternalServerError().body("Blockchain error"));
                        break;
                    }
                }

                let allocator_creation_result = allocators_db::create_or_update_allocator(
                    owner.clone(),
                    repo.clone(),
                    None,
                    Some(model.pathway_addresses.msig),
                    verifiers_gh_handles,
                    model.multisig_threshold,
                    signers,
                )
                .await;

                match allocator_creation_result {
                    Ok(_) => match is_allocator_repo_created(&owner, &repo).await {
                        Ok(true) => (),
                        Ok(false) => match create_allocator_repo(&owner, &repo).await {
                            Ok(_) => (),
                            Err(e) => {
                                error_response =
                                    Some(HttpResponse::BadRequest().body(e.to_string()));
                                break;
                            }
                        },
                        Err(e) => {
                            error_response = Some(HttpResponse::BadRequest().body(e.to_string()));
                            break;
                        }
                    },
                    Err(e) => {
                        error_response = Some(HttpResponse::BadRequest().body(e.to_string()));
                        break;
                    }
                }
            }
            Err(e) => {
                error_response = Some(HttpResponse::BadRequest().body(e.to_string()));
                break;
            }
        }
    }

    if let Some(response) = error_response {
        return Ok(response);
    }

    Ok(HttpResponse::Ok().body("All files processed successfully"))
}

/**
 * Update an allocator
 *
 * # Arguments
 * @param path: web::Path<(String, String)> - The owner and repo of the allocator
 * @param info: web::Json<AllocatorUpdateInfo> - The updated allocator information
 *
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[put("/allocator/{owner}/{repo}")]
pub async fn update(
    path: web::Path<(String, String)>,
    info: web::Json<AllocatorUpdateInfo>,
) -> impl Responder {
    let (owner, repo) = path.into_inner();
    match allocators_db::update_allocator(
        &owner,
        &repo,
        None,
        info.multisig_address.clone(),
        info.verifiers_gh_handles.clone(),
        info.multisig_threshold,
    )
    .await
    {
        Ok(allocator_model) => HttpResponse::Ok().json(allocator_model),
        Err(e) => {
            if e.to_string().contains("Allocator not found") {
                return HttpResponse::NotFound().body(e.to_string());
            }
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    }
}

/**
 * Get an allocator
 *
 * # Arguments
 * @param path: web::Path<(String, String)> - The owner and repo of the allocator
 *
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[get("/allocator/{owner}/{repo}")]
pub async fn allocator(path: web::Path<(String, String)>) -> impl Responder {
    let (owner, repo) = path.into_inner();
    match allocators_db::get_allocator(&owner, &repo).await {
        Ok(allocator) => match allocator {
            Some(allocator) => HttpResponse::Ok().json(allocator),
            None => HttpResponse::NotFound().finish(),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/**
 * Delete an allocator
 *
 * # Arguments
 * @param path: web::Path<(String, String)> - The owner and repo of the allocator
 *
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[delete("/allocator/{owner}/{repo}")]
pub async fn delete(path: web::Path<(String, String)>) -> impl Responder {
    let (owner, repo) = path.into_inner();
    match allocators_db::delete_allocator(&owner, &repo).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            if e.to_string().contains("Allocator not found") {
                return HttpResponse::NotFound().body(e.to_string());
            }
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    }
}

// #[post("/allocator/update_installation_ids")]
// pub async fn update_installation_ids() -> impl Responder {
//     match update_installation_ids_logic().await {
//         Ok(results) => HttpResponse::Ok().json(results),
//         Err(e) => {
//             log::error!("Failed to fetch installation ids: {}", e);
//             HttpResponse::InternalServerError().body(format!("{}", e))
//         }
//     }
// }

#[get("/allocator/update_installation_id")]
pub async fn update_single_installation_id(
    query: web::Query<InstallationIdUpdateInfo>,
) -> impl Responder {
    match update_single_installation_id_logic(query.installation_id.to_string()).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            log::error!("Failed to fetch installation ids: {}", e);
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}
