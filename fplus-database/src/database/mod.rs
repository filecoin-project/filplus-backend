use sea_orm::{entity::*, query::*};
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
 * 
 * # Returns
 * @return Result<(), sea_orm::DbErr> - The result of the operation
 */
pub async fn update_allocator(
    owner: &str,
    repo: &str,
    allocator_fields: ActiveModel,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    Allocator::update(allocator_fields)
        .filter(Column::Owner.eq(owner))
        .filter(Column::Repo.eq(repo))
        .exec(&conn)
        .await?;
    Ok(())
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
 * Create an allocator in the database
 * 
 * # Arguments
 * @param allocator_fields: ActiveModel - The fields of the allocator to create
 * 
 * # Returns
 * @return Result<AllocatorModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn create_allocator(
    allocator_fields: ActiveModel,
) -> Result<AllocatorModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    allocator_fields.insert(&conn).await
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
        None => return Ok(()),
    };
    allocator.delete(&conn).await?;
    Ok(())
}