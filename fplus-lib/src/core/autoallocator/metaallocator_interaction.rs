use std::str::FromStr;

use crate::config::get_env_var_or_default;
use crate::error::LDNError;
use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolCall,
};
use anyhow::Result;
use fvm_shared::address::{set_current_network, Address as FilecoinAddress, Network};
sol! {
  #[allow(missing_docs)]
  function addVerifiedClient(bytes calldata clientAddress, uint256 amount);
}

pub async fn add_verified_client(address: &str, amount: &u64) -> Result<(), LDNError> {
    let private_key = get_env_var_or_default("AUTOALLOCATOR_PRIVATE_KEY");
    let signer: PrivateKeySigner = private_key.parse().expect("Should parse private key");
    let wallet = EthereumWallet::from(signer);
    let rpc_url = get_env_var_or_default("GLIF_NODE_URL")
        .parse()
        .map_err(|e| LDNError::New(format!("Failed to pase string to URL /// {}", e)))?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);
    let fil_address = decode_filecoin_address(address)?;
    let amount = U256::try_from(*amount)
        .map_err(|e| LDNError::New(format!("Failed to prase amount to U256 /// {}", e)))?;
    let call = addVerifiedClientCall {
        clientAddress: fil_address.into(),
        amount,
    }
    .abi_encode();
    let allocator_contract =
        Address::parse_checksummed(get_env_var_or_default("ALLOCATOR_CONTRACT_ADDRESS"), None)
            .map_err(|e| {
                LDNError::New(format!(
                    "Parse ALLOCATOR_CONTRACT_ADDRESS to Address failed: {}",
                    e
                ))
            })?;
    let input = Bytes::from(call);

    let tx = TransactionRequest::default()
        .with_to(allocator_contract)
        .with_input(input)
        .with_gas_limit(45_000_000);

    provider
        .send_transaction(tx)
        .await
        .map_err(|e| LDNError::New(format!("RPC error: {}", e)))?
        .watch()
        .await
        .map_err(|e| LDNError::New(format!("Transaction failed: {}", e)))?;
    Ok(())
}

fn decode_filecoin_address(address: &str) -> Result<Vec<u8>, LDNError> {
    let address_prefix = address.get(0..1);
    if let Some(address_prefix) = address_prefix {
        if address_prefix.eq("f") {
            set_current_network(Network::Mainnet);
        } else if address_prefix.eq("t") {
            set_current_network(Network::Testnet);
        }
    }
    let fil_address = FilecoinAddress::from_str(address)
        .map_err(|e| LDNError::New(format!("Failed to prase address from string /// {}", e)))?;
    Ok(fil_address.to_bytes())
}
