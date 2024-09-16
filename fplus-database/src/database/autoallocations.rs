use crate::get_database_connection;
use crate::models::autoallocations::AddressWrapper;
use crate::models::autoallocations::{ActiveModel, Column, Entity as Autoallocations};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{entity::*, query::*, DbErr};

pub async fn get_last_client_autoallocation(
    client_evm_address: impl Into<AddressWrapper>,
) -> Result<Option<DateTime<FixedOffset>>, DbErr> {
    let conn = get_database_connection().await?;
    let response = Autoallocations::find()
        .filter(Column::EvmWalletAddress.contains(client_evm_address.into()))
        .one(&conn)
        .await?;

    Ok(response.map(|allocation| allocation.last_allocation))
}

pub async fn create_or_update_autoallocation(
    client_evm_address: impl Into<AddressWrapper> + Clone,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let last_autoallocation = get_last_client_autoallocation(client_evm_address.clone()).await?;

    let autoallocation_active_model = ActiveModel {
        evm_wallet_address: Set(client_evm_address.into()),
        last_allocation: Set(Utc::now().into()),
    };
    if last_autoallocation.is_some() {
        autoallocation_active_model.update(&conn).await?;
    } else {
        autoallocation_active_model.insert(&conn).await?;
    }
    Ok(())
}
