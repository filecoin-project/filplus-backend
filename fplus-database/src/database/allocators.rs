use crate::get_database_connection;
use crate::models::allocators::{
    ActiveModel, Column, Entity as Allocator, Model as AllocatorModel,
};
use sea_orm::{entity::*, query::*, DbErr};

/**
 * Get all allocators from the database
 *
 * # Returns
 * @return Result<Vec<AllocatorModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocators() -> Result<Vec<AllocatorModel>, sea_orm::DbErr> {
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
 * @param multisig_threshold: Option<i32> - The multisig threshold
 * @param address: Option<String> - Address of the Allocator
 * @param tooling: Option<String> - Supported tooling
 *
 * # Returns
 * @return Result<AllocatorModel, sea_orm::DbErr> - The result of the operation
 */
#[allow(clippy::too_many_arguments)]
pub async fn update_allocator(
    owner: &str,
    repo: &str,
    installation_id: Option<i64>,
    multisig_address: Option<String>,
    verifiers_gh_handles: Option<String>,
    multisig_threshold: Option<i32>,
    address: Option<String>,
    tooling: Option<String>,
) -> Result<AllocatorModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;

    let existing_allocator = get_allocator(owner, repo).await?;
    if let Some(allocator_model) = existing_allocator {
        let mut allocator_active_model = allocator_model.into_active_model();

        //if fields are not None, update them
        if Some(installation_id).is_some() {
            allocator_active_model.installation_id = Set(installation_id);
        }

        if Some(multisig_address.clone()).is_some() {
            allocator_active_model.multisig_address = Set(multisig_address);
        }

        if Some(verifiers_gh_handles.clone()).is_some() {
            allocator_active_model.verifiers_gh_handles = Set(verifiers_gh_handles);
        }

        if Some(multisig_threshold).is_some() {
            allocator_active_model.multisig_threshold = Set(multisig_threshold);
        }

        if address.is_some() {
            allocator_active_model.address = Set(address);
        }

        if tooling.is_some() {
            allocator_active_model.tooling = Set(tooling);
        }

        let updated_model = allocator_active_model.update(&conn).await?;

        Ok(updated_model)
    } else {
        Err(DbErr::Custom("Allocator not found".to_string()))
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
 * @param address: Option<String> - Address of the Allocator
 * @param tooling: Option<String> - Supported tooling
 *
 * # Returns
 * @return Result<AllocatorModel, sea_orm::DbErr> - The result of the operation
 */
#[allow(clippy::too_many_arguments)]
pub async fn create_or_update_allocator(
    owner: String,
    repo: String,
    installation_id: Option<i64>,
    multisig_address: Option<String>,
    verifiers_gh_handles: Option<String>,
    multisig_threshold: Option<i32>,
    allocation_amount_type: Option<String>,
    address: Option<String>,
    tooling: Option<String>,
) -> Result<AllocatorModel, sea_orm::DbErr> {
    let existing_allocator = get_allocator(&owner, &repo).await?;
    if let Some(allocator_model) = existing_allocator {
        let conn = get_database_connection().await?;
        let mut allocator_active_model = allocator_model.into_active_model();

        if installation_id.is_some() {
            allocator_active_model.installation_id = Set(installation_id);
        }

        if multisig_address.is_some() {
            allocator_active_model.multisig_address = Set(multisig_address);
        }

        if verifiers_gh_handles.is_some() {
            allocator_active_model.verifiers_gh_handles = Set(verifiers_gh_handles);
        }

        if multisig_threshold.is_some() {
            allocator_active_model.multisig_threshold = Set(multisig_threshold);
        }

        if let Some(allocation_amount_type) = allocation_amount_type {
            allocator_active_model.allocation_amount_type =
                Set(Some(allocation_amount_type.to_lowercase()));
        } else {
            allocator_active_model.allocation_amount_type = Set(None);
        }

        if address.is_some() {
            allocator_active_model.address = Set(address);
        }

        if tooling.is_some() {
            allocator_active_model.tooling = Set(tooling);
        }

        let updated_model = allocator_active_model.update(&conn).await?;

        Ok(updated_model)
    } else {
        let mut new_allocator = ActiveModel {
            owner: Set(owner),
            repo: Set(repo),
            ..Default::default()
        };

        if installation_id.is_some() {
            new_allocator.installation_id = Set(installation_id);
        }

        if multisig_address.is_some() {
            new_allocator.multisig_address = Set(multisig_address);
        }

        if verifiers_gh_handles.is_some() {
            new_allocator.verifiers_gh_handles = Set(verifiers_gh_handles);
        }

        if multisig_threshold.is_some() {
            new_allocator.multisig_threshold = Set(multisig_threshold);
        }

        if let Some(allocation_amount_type) = allocation_amount_type {
            new_allocator.allocation_amount_type = Set(Some(allocation_amount_type.to_lowercase()));
        } else {
            new_allocator.allocation_amount_type = Set(None);
        }

        if address.is_some() {
            new_allocator.address = Set(address);
        }

        if tooling.is_some() {
            new_allocator.tooling = Set(tooling);
        }

        let conn = get_database_connection()
            .await
            .expect("Failed to get DB connection");
        let insert_result = new_allocator.insert(&conn).await;
        println!("Allocator inserted: {:?}", insert_result);
        Ok(insert_result.unwrap())
    }
}

/**
 * Update the multisig threshold of an allocator in the database
 *
 * This function specifically targets and updates only the multisig threshold of an existing allocator.
 *
 * # Arguments
 * @param owner: &str - The owner of the repository.
 * @param repo: &str - The name of the repository.
 * @param multisig_threshold: i32 - The new multisig threshold to be updated in the allocator.
 *
 * # Returns
 * @return Result<AllocatorModel, sea_orm::DbErr> - The result of the operation.
 * On successful update, it returns the updated AllocatorModel.
 * On failure, it returns an error of type DbErr.
 */
pub async fn update_allocator_threshold(
    owner: &str,
    repo: &str,
    multisig_threshold: i32,
) -> Result<AllocatorModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let mut existing_allocator = get_allocator(owner, repo)
        .await?
        .ok_or_else(|| DbErr::Custom("Allocator not found".into()))?
        .into_active_model();

    existing_allocator.multisig_threshold = Set(Some(multisig_threshold));

    existing_allocator.update(&conn).await
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
pub async fn delete_allocator(owner: &str, repo: &str) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let allocator = get_allocator(owner, repo).await?;
    let allocator = match allocator {
        Some(allocator) => allocator,
        None => return Err(DbErr::Custom("Allocator not found".to_string())),
    };
    allocator.delete(&conn).await?;
    Ok(())
}
