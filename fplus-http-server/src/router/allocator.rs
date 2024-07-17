use actix_web::{delete, get, post, web, HttpResponse, Responder};
use fplus_database::database::allocators as allocators_db;
use fplus_lib::core::{
    allocator::{
        create_allocator_from_file, fetch_installation_ids, force_update_allocators,
        generate_github_app_jwt,
    },
    AllocatorUpdateForceInfo, ChangedAllocators,
};
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
pub async fn create_allocator_from_json(files: web::Json<ChangedAllocators>) -> impl Responder {
    let ChangedAllocators { files_changed } = files.into_inner();
    match create_allocator_from_file(files_changed).await {
        Ok(()) => HttpResponse::Ok()
            .body(serde_json::to_string_pretty("All files processed successfully").unwrap()),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
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
            HttpResponse::InternalServerError().body(e.to_string())
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
