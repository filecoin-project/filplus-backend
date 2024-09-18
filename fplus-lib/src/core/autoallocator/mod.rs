use crate::core::autoallocator::metaallocator_interaction::add_verified_client;
use crate::core::get_env_var_or_default;
use crate::core::verify_on_gitcoin;
use crate::core::{LDNApplication, TriggerAutoallocationInfo};
use crate::error::LDNError;
use alloy::primitives::Address;
use fplus_database::database::applications::get_applications_by_client_id;
use fplus_database::database::autoallocations as autoallocations_db;

pub mod metaallocator_interaction;

pub async fn trigger_autoallocation(info: &TriggerAutoallocationInfo) -> Result<(), LDNError> {
    let evm_address_from_signature =
        LDNApplication::verify_kyc_data_and_get_eth_address(&info.message, &info.signature)?;
    verify_on_gitcoin(&evm_address_from_signature).await?;
    let fil_client_address = &info.message.client_fil_address;
    let client_applications = get_applications_by_client_id(fil_client_address)
        .await
        .map_err(|e| LDNError::Load(format!("Get applications for client failed: {}", e)))?;
    if !client_applications.is_empty() {
        return Err(LDNError::Load(
            "Client already has an application".to_string(),
        ));
    }
    let amount = get_env_var_or_default("AUTOALLOCATION_AMOUNT")
        .parse::<u64>()
        .map_err(|e| {
            LDNError::New(format!(
                "Parse days to next allocation to i64 failed: {}",
                e
            ))
        })?;
    upsert_autoallocation_if_eligible(&evm_address_from_signature).await?;
    match add_verified_client(fil_client_address, &amount).await {
        Ok(_) => {}
        Err(e) => {
            autoallocations_db::delete_autoallocation(evm_address_from_signature)
                .await
                .map_err(|e| LDNError::New(format!("Delete autoallocation failed: {}", e)))?;
            return Err(LDNError::New(format!("Add verified client failed: {}", e)));
        }
    }
    Ok(())
}

async fn upsert_autoallocation_if_eligible(evm_client_address: &Address) -> Result<(), LDNError> {
    let days_to_next_autoallocation = get_env_var_or_default("DAYS_TO_NEXT_AUTOALLOCATION")
        .parse::<i64>()
        .map_err(|e| {
            LDNError::New(format!(
                "Parse days to next allocation to i64 failed: {}",
                e
            ))
        })?;
    let rows_affected = autoallocations_db::create_or_update_autoallocation(
        evm_client_address,
        &days_to_next_autoallocation,
    )
    .await
    .map_err(|e| LDNError::New(format!("Create or update autoallocation failed: {}", e)))?;
    if rows_affected == 0 {
        return Err(LDNError::Load(format!(
            "Last allocation was within {} days.",
            days_to_next_autoallocation
        )));
    }
    Ok(())
}
