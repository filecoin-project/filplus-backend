use crate::get_database_connection;
use crate::models::autoallocations::AddressWrapper;
use crate::models::autoallocations::{
    Column, Entity as Autoallocations, Model as AutoallocationModel,
};
use alloy::primitives::Address;
use chrono::{DateTime, FixedOffset};
use sea_orm::{entity::*, query::*, DbBackend, DbErr, ExecResult};

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
    client_evm_address: &Address,
    days_to_next_autoallocation: &u64,
) -> Result<u64, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let client_address = client_evm_address.to_checksum(None);
    let exec_res: ExecResult = conn
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "INSERT INTO autoallocations (evm_wallet_address, last_allocation)
                VALUES ('{}', NOW())
                ON CONFLICT (evm_wallet_address)
                DO UPDATE SET last_allocation = NOW()
                WHERE autoallocations.last_allocation <= NOW() - INTERVAL '{} days';",
                client_address, days_to_next_autoallocation
            ),
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
    let client_autoallocation = get_autoallocation(client_evm_address).await?;
    if let Some(client_autoallocation) = client_autoallocation {
        client_autoallocation.delete(&conn).await?;
    }
    Ok(())
}
