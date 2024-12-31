use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorNotFound},
    get, post, web, HttpResponse, Responder,
};
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
pub async fn allocators() -> actix_web::Result<impl Responder> {
    let allocators = allocators_db::get_allocators()
        .await
        .map_err(ErrorNotFound)?;
    Ok(HttpResponse::Ok().json(allocators))
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
pub async fn create_allocator_from_json(
    files: web::Json<ChangedAllocators>,
) -> actix_web::Result<impl Responder> {
    let ChangedAllocators { files_changed } = files.into_inner();
    create_allocator_from_file(files_changed)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(
        serde_json::to_string_pretty("All files processed successfully")
            .expect("Serialization of static string should succeed"),
    ))
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
pub async fn allocator(path: web::Path<(String, String)>) -> actix_web::Result<impl Responder> {
    let (owner, repo) = path.into_inner();
    let allocator = allocators_db::get_allocator(&owner, &repo)
        .await
        .map_err(ErrorInternalServerError)?;
    if let Some(allocator) = allocator {
        Ok(HttpResponse::Ok().json(allocator))
    } else {
        Err(ErrorNotFound("Allocator not found"))
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
pub async fn delete(path: web::Path<(String, String)>) -> actix_web::Result<impl Responder> {
    let (owner, repo) = path.into_inner();
    allocators_db::delete_allocator(&owner, &repo)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
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
pub async fn update_allocator_force(
    body: web::Json<AllocatorUpdateForceInfo>,
) -> actix_web::Result<impl Responder> {
    // First we need to deconstruct the body
    let AllocatorUpdateForceInfo {
        files,
        allocators: affected_allocators,
    } = body.into_inner();

    // Logic will be implemented in allocator::update_allocator_force
    force_update_allocators(files, affected_allocators)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(()))
}

#[get("/get_installation_ids")]
pub async fn get_installation_ids() -> actix_web::Result<impl Responder> {
    let client = Client::new();
    let jwt = generate_github_app_jwt()
        .await
        .map_err(ErrorInternalServerError)?;

    let ids = fetch_installation_ids(&client, &jwt).await.map_err(|e| {
        log::error!("Failed to generate GitHub App JWT: {}", e);
        ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().json(ids))
}
