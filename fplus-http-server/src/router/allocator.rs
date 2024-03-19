use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use fplus_database::database::allocators as allocators_db;
use fplus_lib::{core::{allocator::{
    create_allocator_repo, is_allocator_repo_created, process_allocator_file, update_single_installation_id_logic
}, AllocatorUpdateInfo, ChangedAllocator, InstallationIdUpdateInfo}, external_services::filecoin::get_multisig_threshold_for_actor};

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
        },
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
pub async fn create_from_json(file: web::Json<ChangedAllocator>) -> actix_web::Result<impl Responder> {
    let file_name = &file.file_changed;
    log::info!("Starting allocator creation on:  {}", file_name);

    match process_allocator_file(file_name).await {
        Ok(model) => {
            if model.pathway_addresses.msig.is_empty() {
                return Ok(HttpResponse::BadRequest().body("Missing or invalid multisig_address"));
            }
            let verifiers_gh_handles = if model.application.verifiers_gh_handles.is_empty() {
                None
            } else {
                Some(model.application.verifiers_gh_handles.join(", ")) // Join verifiers in a string if exists
            };
            let owner = model.owner.clone().unwrap_or_default().to_string();
            let repo = model.repo.clone().unwrap_or_default().to_string();

            let allocator_model = match allocators_db::create_or_update_allocator(
                owner.clone(),
                repo.clone(),
                None,
                Some(model.pathway_addresses.msig),      
                verifiers_gh_handles,
                model.multisig_threshold
            ).await {
                Ok(allocator_model) => allocator_model,
                Err(e) => return Ok(HttpResponse::BadRequest().body(e.to_string())),
            };

            match is_allocator_repo_created(&owner, &repo).await {
                Ok(true) => Ok(HttpResponse::Ok().json(allocator_model)),
                Ok(false) => {
                    //Create allocator repo. If it fails, return http error
                    match create_allocator_repo(&owner, &repo).await {
                        Ok(files_list) => {
                            log::info!("Allocator repo created successfully: {:?}", files_list);
                            Ok(HttpResponse::Ok().json(allocator_model))
                        }
                        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
                    }
                },
                Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
            }
        },
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
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
    info: web::Json<AllocatorUpdateInfo>
) -> impl Responder {
    let (owner, repo) = path.into_inner();
    match allocators_db::update_allocator(
        &owner,
        &repo,
        None,
        info.multisig_address.clone(),
        info.verifiers_gh_handles.clone(),
        info.multisig_threshold
    ).await {
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
        Ok(allocator) => {
            match allocator {
                Some(allocator) => HttpResponse::Ok().json(allocator),
                None => HttpResponse::NotFound().finish(),
            }
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string())
        }
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
pub async fn update_single_installation_id(query: web::Query<InstallationIdUpdateInfo>,) -> impl Responder {
    match update_single_installation_id_logic(query.installation_id.to_string()).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => {
            log::error!("Failed to fetch installation ids: {}", e);
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}