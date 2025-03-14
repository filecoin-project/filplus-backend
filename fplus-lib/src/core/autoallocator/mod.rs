use crate::core::autoallocator::metaallocator_interaction::add_verified_client;
use crate::core::get_env_var_or_default;
use crate::core::verify_on_gitcoin;
use crate::core::{LDNApplication, TriggerAutoallocationInfo};
use crate::error::LDNError;
use crate::external_services::blockchain::get_allowance_for_address_contract;
use crate::external_services::filecoin::evm_address_to_filecoin_address;
use crate::external_services::filecoin::get_allowance_for_address_direct;
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use fplus_database::config::get_env_or_throw;
use fplus_database::database::applications::get_applications_by_client_id;
use fplus_database::database::autoallocations as autoallocations_db;
use std::cmp::min;

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
    if let Err(e) = add_verified_client(fil_client_address, &amount).await {
        autoallocations_db::delete_autoallocation(evm_address_from_signature)
            .await
            .map_err(|err| LDNError::New(format!("Delete autoallocation failed: {}", err)))?;
        return Err(LDNError::New(format!("Add verified client failed: {}", e)));
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

pub async fn check_if_allowance_is_sufficient() -> Result<bool, LDNError> {
    let contract_address = get_env_var_or_default("ALLOCATOR_CONTRACT_ADDRESS");
    let parsed_constract_address_to_fil = evm_address_to_filecoin_address(&contract_address)
        .await
        .map_err(|e| {
            LDNError::Load(format!("Failed to parse EVM address to FIL address: {}", e))
        })?;
    let contract_allowance = get_allowance_for_address_direct(&parsed_constract_address_to_fil)
        .await
        .map_err(|e| LDNError::Load(format!("Failed to retrieve allowance: {}", e)))?
        .parse::<u64>()
        .map_err(|e| LDNError::New(format!("Parse contract allowance to u64 failed: {}", e)))?;

    let parsed_contract_address_to_evm: Address = contract_address.parse().map_err(|e| {
        LDNError::New(format!(
            "Failed to get evm address from filecoin address: {e:?}"
        ))
    })?;
    let allocator_private_key = get_env_or_throw("AUTOALLOCATOR_PRIVATE_KEY");
    let allocator_address = allocator_private_key
        .parse::<PrivateKeySigner>()
        .expect("Should parse private key")
        .address();

    let allocator_allowance_on_contract =
        get_allowance_for_address_contract(&allocator_address, &parsed_contract_address_to_evm)
            .await?;

    let allowance = min(allocator_allowance_on_contract, contract_allowance);
    let autoallocation_amount = get_env_var_or_default("AUTOALLOCATION_AMOUNT")
        .parse::<u64>()
        .map_err(|e| {
            LDNError::New(format!(
                "Parse days to next allocation to i64 failed: {}",
                e
            ))
        })?;
    if allowance < autoallocation_amount {
        return Ok(false);
    }
    Ok(true)
}
