use std::{env, str::FromStr};

use alloy::{
    network::TransactionBuilder,
    node_bindings::Anvil,
    primitives::{address, Address, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    signers::Signature,
    sol,
    sol_types::SolCall,
};

use anyhow::{anyhow, Result};
use fplus_database::config::get_env_or_throw;

use crate::core::SignatureRequest;

const GITCOIN_PASSPORT_DECODER: Address = address!("5558D441779Eca04A329BcD6b47830D2C6607769");

sol!(
    #[allow(missing_docs)]
    function getScore(address user) view returns (uint256);
);

pub async fn verify_on_gitcoin(signature: &str, message: &[u8]) -> Result<()> {
    let signature = Signature::from_str(signature)?;
    let address_from_signature = get_address_from_signature(&signature, message)?;
    let score = get_gitcoin_score_for_address(address_from_signature).await?;

    let minimum_score = env::var("GITCOIN_MINIMUM_SCORE")?;
    let minimum_score = minimum_score.parse::<f64>()?;

    if score > minimum_score {
        Ok(())
    } else {
        Err(anyhow!(format!("For address: {}, Gitcoin passport score is too low ({}). Minimum value is: {}", address_from_signature, score, minimum_score)))
    }
}

async fn get_gitcoin_score_for_address(address: Address) -> Result<f64> {
    // todo somehow separate anvil from logic
    
    // let ledger_url = get_env_or_throw("NETWORK_URL");
    let anvil = Anvil::new()
        // .fork(ledger_url)
        .fork("https://mainnet.optimism.io")
        .fork_block_number(120144603)
        .try_spawn()
        .unwrap();

    let rpc_url = anvil.endpoint().parse().unwrap();
    let provider = ProviderBuilder::new().on_http(rpc_url);

    let call = getScoreCall {
        user: address,
    }
    .abi_encode();

    let input = Bytes::from(call);
    let tx = TransactionRequest::default()
        .with_to(GITCOIN_PASSPORT_DECODER)
        .with_input(input);

    let response = provider.call(&tx).block(BlockId::latest()).await?;
    let score = U256::from_str(&response.to_string())?;
    let score = score.to::<u128>();
    let score = score as f64 / 100.0;
    Ok(score)
}

fn get_address_from_signature(
    signature: &Signature,
    message: &[u8],
) -> Result<Address, alloy::signers::Error> {
    let address_from_signature = signature.recover_address_from_msg(&message[..])?;
    Ok(address_from_signature)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const SIGNATURE_HASH: &str = "0xeeec9a87a01977a48fa5bac97d2f1c67d83905ac378573d6749ae078b76b3ef078f2187ef9cbf4eaf2069066fdc32b07823508db3871ab07f32d30137c0140a81c";
    const SIGNATURE_MESSAGE: &[u8] = b"KYC";

    #[actix_rt::test]
    async fn getting_score_from_gitcoin_passport_decoder_works() {
        let result = get_gitcoin_score_for_address(address!("0922B44805CB5D90F35F3b9781aFf83b47D722d3")).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // getScore returns 301058 for input address on block 120144603
        assert_eq!(result, 3010.58);
    }

    #[actix_rt::test]
    async fn verifier_returns_valid_address_for_valid_message() {
        let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

        let address_from_signature = get_address_from_signature(&signature, SIGNATURE_MESSAGE).unwrap();

        let expected_address =
            Address::from_str("0x79e214f3aa3101997ffe810a57eca4586e3bdeb2").unwrap();

        assert_eq!(expected_address, address_from_signature);
    }

    #[actix_rt::test]
    async fn verifier_returns_invalid_address_for_invalid_message() {
        let message = b"Invalid message";
        let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

        let address_from_signature = get_address_from_signature(&signature, message).unwrap();

        let expected_address =
            Address::from_str("0x79e214f3aa3101997ffe810a57eca4586e3bdeb2").unwrap();

        assert_ne!(expected_address, address_from_signature);
    }
}
