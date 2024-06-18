use actix_web::{delete, get, post, web, HttpResponse, Responder};
use fplus_database::database::allocation_amounts as allocation_amounts_db;
use fplus_database::database::allocators as allocators_db;
use fplus_lib::core::allocator::fetch_installation_ids;
use fplus_lib::core::allocator::generate_github_app_jwt;
use fplus_lib::core::{
    allocator::{
        create_allocator_repo, force_update_allocators, is_allocator_repo_created,
        process_allocator_file, update_single_installation_id_logic,
        validate_amount_type_and_options,
    },
    AllocatorUpdateForceInfo, ChangedAllocators, InstallationIdUpdateInfo,
};
use fplus_lib::helpers::process_amount;
use reqwest::Client;

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
    let mut error_response: Option<HttpResponse> = None;

    for file_name in &files.files_changed {
        log::info!("Starting allocator creation on: {}", file_name);

        match process_allocator_file(file_name).await {
            Ok(mut model) => {
                let mut quantity_options: Vec<String>;
                if let Some(allocation_amount) = model.application.allocation_amount.clone() {
                    if allocation_amount.amount_type.clone().is_none()
                        || allocation_amount.quantity_options.clone().is_none()
                    {
                        error_response = Some(
                            HttpResponse::BadRequest()
                                .body("Amount type and quantity options are required"),
                        );
                        break;
                    }

                    let amount_type = allocation_amount
                        .amount_type
                        .clone()
                        .unwrap()
                        .to_lowercase(); // Assuming you still want to unwrap here
                    quantity_options = allocation_amount.quantity_options.unwrap(); // Assuming unwrap is desired

                    for option in quantity_options.iter_mut() {
                        *option = process_amount(option.clone());
                    }

                    match validate_amount_type_and_options(&amount_type, &quantity_options) {
                        Ok(()) => println!("Options are valid"),
                        Err(e) => {
                            error_response = Some(HttpResponse::BadRequest().body(e.to_string()));
                            break;
                        }
                    }

                    model
                        .application
                        .allocation_amount
                        .as_mut()
                        .unwrap()
                        .quantity_options = Some(quantity_options);
                }

                let verifiers_gh_handles = if model.application.verifiers_gh_handles.is_empty() {
                    None
                } else {
                    Some(model.application.verifiers_gh_handles.join(", ")) // Join verifiers in a string if exists
                };
                let tooling = if model.application.tooling.is_empty() {
                    None
                } else {
                    Some(model.application.tooling.join(", "))
                };

                let disable_ssa_bot = model.application.disable_ssa_bot.or(Some(false));

                let owner = model.owner.clone().unwrap_or_default().to_string();
                let repo = model.repo.clone().unwrap_or_default().to_string();

                let allocator_creation_result = allocators_db::create_or_update_allocator(
                    owner.clone(),
                    repo.clone(),
                    None,
                    Some(model.pathway_addresses.msig),
                    verifiers_gh_handles,
                    model.multisig_threshold,
                    model
                        .application
                        .allocation_amount
                        .clone()
                        .and_then(|a| a.amount_type.clone()),
                    model.address,
                    tooling,
                    disable_ssa_bot,
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

                let allocator_id = allocator_creation_result.unwrap().id;

                // Delete all old allocation amounts by allocator id
                match allocation_amounts_db::delete_allocation_amounts_by_allocator_id(allocator_id)
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        error_response = Some(HttpResponse::BadRequest().body(err.to_string()));
                        break;
                    }
                }

                if let Some(allocation_amount) = model.application.allocation_amount.clone() {
                    let allocation_amounts = allocation_amount.quantity_options.unwrap();

                    for allocation_amount in allocation_amounts {
                        let parsed_allocation_amount = allocation_amount.replace('%', "");
                        match allocation_amounts_db::create_allocation_amount(
                            allocator_id,
                            parsed_allocation_amount,
                        )
                        .await
                        {
                            Ok(_) => (),
                            Err(err) => {
                                error_response =
                                    Some(HttpResponse::BadRequest().body(err.to_string()));
                                break;
                            }
                        }
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
            HttpResponse::InternalServerError().body(e.to_string())
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

/**
 * Force updating allocator files from template.
 * It receives a list of changed files and allocators to update.
 * If allocators is not provided, it will update all allocators as long as they have an installation id.
 *
 * # Arguments
 * @param AllocatorUpdateForceInfo - The list of changed JSON file names and allocators to update
 */
#[post("/allocator/update/force")]
pub async fn update_allocator_force(body: web::Json<AllocatorUpdateForceInfo>) -> impl Responder {
    // First we need to deconstruct the body
    let AllocatorUpdateForceInfo {
        files,
        allocators: affected_allocators,
    } = body.into_inner();

    // Logic will be implemented in allocator::update_allocator_force
    match force_update_allocators(files, affected_allocators).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            log::error!("Failed to update allocators: {}", e);
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/get_installation_ids")]
pub async fn get_installation_ids() -> impl Responder {
    let client = Client::new();
    let jwt = match generate_github_app_jwt().await {
        Ok(jwt) => jwt,
        Err(e) => {
            log::error!("Failed to generate GitHub App JWT: {}", e);
            return HttpResponse::InternalServerError().finish(); // Ensure to call .finish()
        }
    };

    match fetch_installation_ids(&client, &jwt).await {
        Ok(ids) => {
            // Assuming `ids` can be serialized directly; adjust based on your actual data structure
            HttpResponse::Ok().json(ids)
        }
        Err(e) => {
            log::error!("Failed to fetch installation IDs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
