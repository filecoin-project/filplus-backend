use crate::get_database_connection;
use crate::models::autoallocations::AddressWrapper;
use crate::models::autoallocations::{Column, Entity as Autoallocation};
use chrono::{DateTime, FixedOffset};
use sea_orm::{entity::*, query::*, DbErr};

pub async fn get_last_calient_autoallocation(
    client_evm_address: impl Into<AddressWrapper>,
) -> Result<DateTime<FixedOffset>, DbErr> {
    let conn = get_database_connection().await?;
    let result = Autoallocation::find()
        .filter(Column::EvmWalletAddress.contains(client_evm_address.into()))
        .one(&conn)
        .await?
        .ok_or_else(|| DbErr::Custom("Autoallocation not found.".to_string()));
    Ok(result?.last_allocation)
}
