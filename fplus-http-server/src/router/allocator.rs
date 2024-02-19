use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use fplus_database::database;
use fplus_lib::core::{allocator::process_allocator_file, Allocator, AllocatorUpdateInfo, ChangedAllocator};

/**
 * Get all allocators
 * 
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[get("/allocators")]
pub async fn allocators() -> impl Responder {
    let allocators = database::get_allocators().await;
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

    match process_allocator_file(file_name).await {
        Ok(model) => {
            if model.multisig_address.is_empty() {
                return Ok(HttpResponse::BadRequest().body("Missing or invalid multisig_address"));
            }
            let verifiers_gh_handles = if model.verifiers.is_empty() {
                None
            } else {
                Some(model.verifiers.join(", ")) // Join verifiers in a string if exists
            };
            
            match database::create_or_update_allocator(
                model.organization,
                model.slug,
                Some(model.installation_id as i64), 
                Some(model.multisig_address),      
                verifiers_gh_handles,
            ).await {
                Ok(allocator_model) => Ok(HttpResponse::Ok().json(allocator_model)),
                Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
            }
        },
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}







/**
 * Create a new allocator
 * 
 * # Arguments
 * @param info: web::Json<Allocator> - The allocator information
 * 
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[post("/allocator")]
pub async fn create_or_update(info: web::Json<Allocator>) -> impl Responder {
    match database::create_or_update_allocator(
        info.owner.clone(),
        info.repo.clone(),
        info.installation_id,
        info.multisig_address.clone(),
        info.verifiers_gh_handles.clone(),
    ).await {
        Ok(allocator_model) => HttpResponse::Ok().json(allocator_model),
        Err(e) => {
            if e.to_string().contains("Allocator already exists") {
                return HttpResponse::BadRequest().body(e.to_string());
            }
            return HttpResponse::InternalServerError().body(e.to_string());
        }
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
    match database::update_allocator(
        &owner,
        &repo,
        info.installation_id,
        info.multisig_address.clone(),
        info.verifiers_gh_handles.clone(),
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
    match database::get_allocator(&owner, &repo).await {
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
    match database::delete_allocator(&owner, &repo).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            if e.to_string().contains("Allocator not found") {
                return HttpResponse::NotFound().body(e.to_string());
            }
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    }
}