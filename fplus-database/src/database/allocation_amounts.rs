use crate::get_database_connection;
use crate::models::allocation_amounts::{
    ActiveModel, Column, Entity as AllocationAmount, Model as AllocationAmountModel,
    QuantityOptionModel,
};
use sea_orm::{entity::*, query::*};

/**
 * Get all allocation amount rows from the database
 *
 * # Returns
 * @return Result<Vec<AllocationAmountModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocation_amounts() -> Result<Vec<AllocationAmountModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    AllocationAmount::find().all(&conn).await
}

/**
 * Get all allocation quantity options from the database for a given allocator
 *
 * # Parameters
 * - `allocator_id`: The ID of the allocator to fetch quantity options for
 *
 * # Returns
 * @return Result<Vec<String>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_allocation_quantity_options(
    allocator_id: i32,
) -> Result<Vec<String>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let quantity_options = AllocationAmount::find()
        .select_only()
        .column(Column::QuantityOption)
        .filter(Column::AllocatorId.eq(allocator_id))
        .into_model::<QuantityOptionModel>() // You will need to define this struct
        .all(&conn)
        .await?
        .into_iter()
        .map(|model| model.quantity_option)
        .collect();

    Ok(quantity_options)
}

/**
 * Create allocation amount rows in the database
 *
 * # Arguments
 * - `allocator_id`: i32 - The allocator_id for the new rows
 * - `allocation_amounts`: Vec<String> - The allocation amounts for the new rows
 *
 * # Returns
 * @return Result<(), sea_orm::DbErr> - The result of the operation
 */
pub async fn create_allocation_amount(
    allocator_id: i32,
    allocation_amount: String,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;

    let new_allocation_amount = ActiveModel {
        allocator_id: Set(allocator_id),
        quantity_option: Set(allocation_amount.clone()),
        ..Default::default()
    };

    let insert_result = new_allocation_amount.insert(&conn).await;
    println!("Allocation amount inserted: {:?}", insert_result);

    Ok(())
}

/**
* Delete all allocation amount rows from the database based on allocator_id
*
* # Arguments
* - `allocator_id`: i32 - The allocator_id to filter the rows
*
* # Returns
* @return Result<(), sea_orm::DbErr> - The result of the operation
*/
pub async fn delete_allocation_amounts_by_allocator_id(
    allocator_id: i32,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    AllocationAmount::delete_many()
        .filter(Column::AllocatorId.eq(allocator_id))
        .exec(&conn)
        .await?;

    Ok(())
}
