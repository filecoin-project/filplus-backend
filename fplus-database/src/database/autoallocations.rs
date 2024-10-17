use crate::get_database_connection;
use crate::models::autoallocations::{
    Column, Entity as Autoallocations, Model as AutoallocationModel,
};
use crate::types::AddressWrapper;
use alloy::primitives::Address;
use chrono::{DateTime, FixedOffset};
use sea_orm::{entity::*, query::*, DbBackend, DbErr};

pub async fn get_last_client_autoallocation(
    client_evm_address: impl Into<AddressWrapper>,
) -> Result<Option<DateTime<FixedOffset>>, DbErr> {
    let response = get_autoallocation(client_evm_address.into()).await?;
    Ok(response.map(|allocation| allocation.last_allocation))
}

pub async fn create_or_update_autoallocation(
    client_evm_address: &Address,
    days_to_next_autoallocation: &i64,
) -> Result<u64, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let client_address = client_evm_address.to_checksum(None);

    let exec_res = conn
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO autoallocations (evm_wallet_address, last_allocation)
                VALUES ($1, NOW())
                ON CONFLICT (evm_wallet_address)
                DO UPDATE SET last_allocation = NOW()
                WHERE autoallocations.last_allocation <= NOW() - (INTERVAL '1 day' * $2::int);",
            [client_address.into(), (*days_to_next_autoallocation).into()],
        ))
        .await?;
    Ok(exec_res.rows_affected())
}

pub async fn get_autoallocation(
    client_evm_address: impl Into<AddressWrapper>,
) -> Result<Option<AutoallocationModel>, DbErr> {
    let conn = get_database_connection().await?;
    let response = Autoallocations::find()
        .filter(Column::EvmWalletAddress.contains(client_evm_address.into()))
        .one(&conn)
        .await?;
    Ok(response)
}

pub async fn delete_autoallocation(
    client_evm_address: impl Into<AddressWrapper>,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    Autoallocations::delete_by_id(client_evm_address.into())
        .exec(&conn)
        .await?;
    Ok(())
}
