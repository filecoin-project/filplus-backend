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
 * @param data_types: Option<Vec<String>> - Supported data_types
 * @param required_sps: Option<String> - Required number of SPs
 * @param required_replicas: Option<String> - Required number of replicas
 * @param registry_file_path: Option<String> - Path to JSON file specifying the allocator in registry repo
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
    data_types: Option<Vec<String>>,
    required_sps: Option<String>,
    required_replicas: Option<String>,
    registry_file_path: Option<String>,
    client_contract_address: Option<String>,
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

        if data_types.is_some() {
            allocator_active_model.data_types = Set(data_types);
        }

        if required_sps.is_some() {
            allocator_active_model.required_sps = Set(required_sps);
        }

        if required_replicas.is_some() {
            allocator_active_model.required_replicas = Set(required_replicas);
        }

        if registry_file_path.is_some() {
            allocator_active_model.registry_file_path = Set(registry_file_path);
        }

        if let Some(client_contract_address) = client_contract_address {
            if !client_contract_address.is_empty() {
                allocator_active_model.client_contract_address = Set(Some(client_contract_address));
            } else {
                allocator_active_model.client_contract_address = Set(None);
            }
        } else {
            allocator_active_model.client_contract_address = Set(None);
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

        if data_types.is_some() {
            new_allocator.data_types = Set(data_types);
        }

        if required_sps.is_some() {
            new_allocator.required_sps = Set(required_sps);
        }

        if required_replicas.is_some() {
            new_allocator.required_replicas = Set(required_replicas);
        }

        if registry_file_path.is_some() {
            new_allocator.registry_file_path = Set(registry_file_path);
        }

        if let Some(client_contract_address) = client_contract_address {
            if !client_contract_address.is_empty() {
                new_allocator.client_contract_address = Set(Some(client_contract_address));
            } else {
                new_allocator.client_contract_address = Set(None);
            }
        }
        let conn = get_database_connection()
            .await
            .expect("Failed to get DB connection");
        let insert_result = new_allocator.insert(&conn).await?;
        println!("Allocator inserted: {:?}", insert_result);
        Ok(insert_result)
    }
}

/**
 * Update installation ID for allocator in the database
 *
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param installation_id: Option<i64> - The installation ID
 */
pub async fn update_allocator_installation_ids(
    owner: String,
    repo: String,
    installation_id: Option<i64>,
) -> Result<(), sea_orm::DbErr> {
    let existing_allocator = get_allocator(&owner, &repo).await?;
    if let Some(allocator_model) = existing_allocator {
        let conn = get_database_connection().await?;
        let mut allocator_active_model = allocator_model.into_active_model();
        if installation_id.is_some() {
            allocator_active_model.installation_id = Set(installation_id);
        }
        allocator_active_model.update(&conn).await?;
    }
    Ok(())
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
    let allocator = get_allocator(owner, repo)
        .await?
        .ok_or(DbErr::Custom("Allocator not found".to_string()))?;
    allocator.delete(&conn).await?;
    Ok(())
}
