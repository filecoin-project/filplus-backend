use crate::get_database_connection;
use crate::models::autoallocations::AddressWrapper;
use crate::models::autoallocations::{Column, Entity as Autoallocation};
use chrono::{DateTime, FixedOffset};
use sea_orm::{entity::*, query::*, DbErr};

pub async fn get_last_client_autoallocation(
    client_evm_address: impl Into<AddressWrapper>,
) -> Result<Option<DateTime<FixedOffset>>, DbErr> {
    let conn = get_database_connection().await?;
    let response = Autoallocation::find()
        .filter(Column::EvmWalletAddress.contains(client_evm_address.into()))
        .one(&conn)
        .await?;

    Ok(response.map(|allocation| allocation.last_allocation))
}
