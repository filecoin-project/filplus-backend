use crate::core::autoallocator::metaallocator_interaction::add_verified_client;
use crate::core::get_env_var_or_default;
use crate::core::verify_on_gitcoin;
use crate::core::{LDNApplication, TriggerAutoallocationInfo};
use crate::error::LDNError;
use alloy::primitives::Address;
use chrono::{Duration, Utc};
use fplus_database::database::applications::get_applications_by_client_id;
use fplus_database::database::autoallocations as autoallocations_db;

pub mod metaallocator_interaction;

pub async fn trigger_autoallocation(info: &TriggerAutoallocationInfo) -> Result<(), LDNError> {
    let evm_address_from_signature =
        LDNApplication::verify_kyc_data_and_get_eth_address(&info.message, &info.signature)?;
    autoallocation_timeout_exceeded(&evm_address_from_signature).await?;
    verify_on_gitcoin(&evm_address_from_signature).await?;
    let fil_client_address = &info.message.client_fil_address;
    let client_applications = get_applications_by_client_id(fil_client_address)
        .await
        .map_err(|e| LDNError::Load(format!("Get applications for client failed: {}", e)))?;
    if client_applications.len() != 0 {
        return Err(LDNError::Load(
            "Cient already has an application".to_string(),
        ));
    }

    let amount = get_env_var_or_default("AMOUNT_TO_AUTOALLOCATION")
        .parse::<u64>()
        .map_err(|e| {
            LDNError::New(format!(
                "Parse days to next allocation to i64 failed: {}",
                e
            ))
        })?;
    add_verified_client(&fil_client_address, &amount).await?;
    autoallocations_db::create_or_update_autoallocation(evm_address_from_signature.clone())
        .await
        .map_err(|e| LDNError::New(format!("Create or update autoallocation failed: {}", e)))?;
    Ok(())
}

async fn autoallocation_timeout_exceeded(
    evm_address_from_signature: &Address,
) -> Result<(), LDNError> {
    let last_client_allocation =
        autoallocations_db::get_last_client_autoallocation(evm_address_from_signature.clone())
            .await
            .map_err(|e| {
                LDNError::Load(format!("Failed to get last client allocation /// {}", e))
            })?;

    if let Some(last_client_allocation) = last_client_allocation {
        let days_to_next_autoallocation = get_env_var_or_default("DAYS_TO_NEXT_AUTOALLOCATION")
            .parse::<i64>()
            .map_err(|e| {
                LDNError::New(format!(
                    "Parse days to next allocation to i64 failed: {}",
                    e
                ))
            })?;
        if (last_client_allocation + Duration::days(days_to_next_autoallocation)) > Utc::now() {
            return Err(LDNError::Load(format!(
                "Last allocation was within {} days.",
                days_to_next_autoallocation
            )));
        }
    }
    Ok(())
}
