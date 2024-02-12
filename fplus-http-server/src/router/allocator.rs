use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use fplus_database::database;
use fplus_lib::core::{Allocator, AllocatorUpdateInfo};

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
 * Create a new allocator
 * 
 * # Arguments
 * @param info: web::Json<Allocator> - The allocator information
 * 
 * # Returns
 * @return HttpResponse - The result of the operation
 */
#[post("/allocator")]
pub async fn create(info: web::Json<Allocator>) -> impl Responder {
    match database::create_allocator(
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