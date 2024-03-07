use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use sea_orm::entity::*;

use crate::models::rkh::ActiveModel;
use crate::{get_database_connection, models::rkh::{Column, Entity as RKH, Model as RKHModel}};

/**
 * Update an rkh entry in the database
 * 
 * # Arguments
 * @param wallet_address: &str - The wallet address of the Root Key Holder
 * @param nonce: &str - The nonce to update
 * 
 * # Returns
 * @return Result<RKHModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn create_or_update_rkh(
  wallet_address: &str,
  nonce: &str,
) -> Result<RKHModel, sea_orm::DbErr> {
  let conn = get_database_connection().await?;

  let existing_rkh = get_rkh(wallet_address).await?;
  if let Some(rkh_model) = existing_rkh {
      let mut rkh_active_model = rkh_model.into_active_model();

      rkh_active_model.nonce = Set(nonce.to_string());
      rkh_active_model.updated_at = Set(chrono::Utc::now());

      let updated_model = rkh_active_model.update(&conn).await?;

      Ok(updated_model)
    } else {
      let new_rkh = ActiveModel {
        wallet_address: Set(wallet_address.to_string()),
        nonce: Set(nonce.to_string()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
      };

      let result = new_rkh.insert(&conn).await?;

      Ok(result)
    }
}

/**
 * Get an allocator from the database
 * 
 * # Arguments
 * @param wallet_Address: &str - the wallet address of the Root Key Holder
 * 
 * # Returns
 * @return Result<Option<RKHModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_rkh(
  wallet_address: &str,
) -> Result<Option<RKHModel>, sea_orm::DbErr> {
  let conn = get_database_connection().await?;
  RKH::find()
      .filter(Column::WalletAddress.eq(wallet_address))
      .one(&conn)
      .await
}