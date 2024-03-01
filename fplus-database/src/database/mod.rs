use sea_orm::{entity::*, query::*, DbErr};
use crate::models::allocators::{Column, ActiveModel, Entity as Allocator, Model as AllocatorModel};
use crate::get_database_connection;

/**
 * Get all allocators from the database
 * 
 * # Returns
 * @return Result<Vec<AllocatorModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocators() ->Result<Vec<AllocatorModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    Allocator::find().all(&conn).await
}

/**
 * Update an allocator in the database
 * 
 * # Arguments
 * @param owner: &str - The owner of the repository
 * @param repo: &str - The repository name
 * @param installation_id: Option<i64> - The installation ID
 * @param multisig_address: Option<String> - The multisig address
 * @param verifiers_gh_handles: Option<String> - The GitHub handles of the verifiers
 * 
 * # Returns
 * @return Result<AllocatorModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn update_allocator(
    owner: &str,
    repo: &str,
    installation_id: Option<i64>,
    multisig_address: Option<String>,
    verifiers_gh_handles: Option<String>,
) -> Result<AllocatorModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;

    let existing_allocator = get_allocator(owner, repo).await?;
    if let Some(allocator_model) = existing_allocator {
        let mut allocator_active_model = allocator_model.into_active_model();

        allocator_active_model.installation_id = Set(installation_id);
        allocator_active_model.multisig_address = Set(multisig_address);
        allocator_active_model.verifiers_gh_handles = Set(verifiers_gh_handles);

        let updated_model = allocator_active_model.update(&conn).await?;

        Ok(updated_model)
    } else {
        Err(DbErr::Custom(format!("Allocator not found").into()))
    }
}

/**
 * Get an allocator from the database
 * 
 * # Arguments
 * @param owner: &str - The owner of the repository
 * @param repo: &str - The repository name
 * 
 * # Returns
 * @return Result<Option<AllocatorModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocator(
    owner: &str,
    repo: &str,
) -> Result<Option<AllocatorModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    Allocator::find()
        .filter(Column::Owner.eq(owner))
        .filter(Column::Repo.eq(repo))
        .one(&conn)
        .await
}

/**
 * Creates or updates an allocator in the database
 * 
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param installation_id: Option<i64> - The installation ID
 * @param multisig_address: Option<String> - The multisig address
 * @param verifiers_gh_handles: Option<String> - The GitHub handles of the verifiers
 * 
 * # Returns
 * @return Result<AllocatorModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn create_or_update_allocator(
    owner: String,
    repo: String,
    installation_id: Option<i64>,
    multisig_address: Option<String>,
    verifiers_gh_handles: Option<String>,
    multisig_threshold: Option<i32>
) -> Result<AllocatorModel, sea_orm::DbErr> {

    let existing_allocator = get_allocator(&owner, &repo).await?;
    if let Some(allocator_model) = existing_allocator {
        let conn = get_database_connection().await?;
        let mut allocator_active_model = allocator_model.into_active_model();

        allocator_active_model.installation_id = Set(installation_id);
        allocator_active_model.multisig_address = Set(multisig_address);
        allocator_active_model.verifiers_gh_handles = Set(verifiers_gh_handles);

        let updated_model = allocator_active_model.update(&conn).await?;

        Ok(updated_model)
    } else {
        let new_allocator = ActiveModel {
            owner: Set(owner),
            repo: Set(repo),
            installation_id: Set(installation_id),
            multisig_address: Set(multisig_address),
            verifiers_gh_handles: Set(verifiers_gh_handles),
            multisig_threshold: Set(multisig_threshold),
            ..Default::default()
        };
    
        let conn = get_database_connection().await.expect("Failed to get DB connection");
        new_allocator.insert(&conn).await
    }
}

/**
 * Delete an allocator from the database
 * 
 * # Arguments
 * @param owner: &str - The owner of the repository
 * @param repo: &str - The repository name
 * 
 * # Returns
 * @return Result<(), sea_orm::DbErr> - The result of the operation
 */
pub async fn delete_allocator(
    owner: &str,
    repo: &str,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let allocator = get_allocator(owner, repo).await?;
    let allocator = match allocator {
        Some(allocator) => allocator,
        None => return Err(DbErr::Custom(format!("Allocator not found").into())),
    };
    allocator.delete(&conn).await?;
    Ok(())
}