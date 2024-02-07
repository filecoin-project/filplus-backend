use sea_orm::{entity::*, query::*, DatabaseConnection};
use crate::models::allocators::{Column, ActiveModel, Entity as Allocator, Model as AllocatorModel};

/**
 * Get all allocators from the database
 * 
 * # Arguments
 * @param conn: &DatabaseConnection - The database connection
 * 
 * # Returns
 * @return Result<Vec<AllocatorModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocators(conn: &DatabaseConnection) ->Result<Vec<AllocatorModel>, sea_orm::DbErr> {
    Allocator::find().all(conn).await
}

/**
 * Update an allocator in the database
 * 
 * # Arguments
 * @param conn: &DatabaseConnection - The database connection
 * @param owner: &str - The owner of the repository
 * @param repo: &str - The repository name
 * 
 * # Returns
 * @return Result<(), sea_orm::DbErr> - The result of the operation
 */
pub async fn update_allocator(
    conn: &DatabaseConnection,
    owner: &str,
    repo: &str,
    allocator_fields: ActiveModel,
) -> Result<(), sea_orm::DbErr> {
    Allocator::update(allocator_fields)
        .filter(Column::Owner.eq(owner))
        .filter(Column::Repo.eq(repo))
        .exec(conn)
        .await?;
    Ok(())
}

/**
 * Get an allocator from the database
 * 
 * # Arguments
 * @param conn: &DatabaseConnection - The database connection
 * @param owner: &str - The owner of the repository
 * @param repo: &str - The repository name
 * 
 * # Returns
 * @return Result<Option<AllocatorModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocator(
    conn: &DatabaseConnection,
    owner: &str,
    repo: &str,
) -> Result<Option<AllocatorModel>, sea_orm::DbErr> {
    Allocator::find()
        .filter(Column::Owner.eq(owner))
        .filter(Column::Repo.eq(repo))
        .one(conn)
        .await
}

pub async fn create_allocator(
    conn: &DatabaseConnection,
    allocator_fields: ActiveModel,
) -> Result<AllocatorModel, sea_orm::DbErr> {
    allocator_fields.insert(conn).await
}

/**
 * Delete an allocator from the database
 * 
 * # Arguments
 * @param conn: &DatabaseConnection - The database connection
 * @param owner: &str - The owner of the repository
 * @param repo: &str - The repository name
 * 
 * # Returns
 * @return Result<(), sea_orm::DbErr> - The result of the operation
 */
pub async fn delete_allocator(
    conn: &DatabaseConnection,
    owner: &str,
    repo: &str,
) -> Result<(), sea_orm::DbErr> {
    let allocator = get_allocator(conn, owner, repo).await?;
    let allocator = match allocator {
        Some(allocator) => allocator,
        None => return Ok(()),
    };
    allocator.delete(conn).await?;
    Ok(())
}