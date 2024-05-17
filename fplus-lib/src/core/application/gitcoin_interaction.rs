use std::{env, str::FromStr};

use alloy::{
    network::TransactionBuilder,
    primitives::{address, Address, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    signers::Signature,
    sol,
    sol_types::SolCall,
};

use anyhow::{anyhow, Result};
use fplus_database::config::get_env_or_throw;

const GITCOIN_PASSPORT_DECODER: Address = address!("5558D441779Eca04A329BcD6b47830D2C6607769");

sol!(
    #[allow(missing_docs)]
    function getScore(address user) view returns (uint256);
);

pub async fn verify_on_gitcoin(signature: &str, message: &[u8]) -> Result<()> {
    let signature = Signature::from_str(signature)?;
    let address_from_signature = get_address_from_signature(&signature, message)?;

    let rpc_url = format!(
        "{}/{}",
        get_env_or_throw("ALCHEMY_RPC_URL"),
        get_env_or_throw("ALCHEMY_API_KEY")
    );
    let score = get_gitcoin_score_for_address(&rpc_url, address_from_signature).await?;

    let minimum_score = env::var("GITCOIN_MINIMUM_SCORE")?;
    let minimum_score = minimum_score.parse::<f64>()?;

    if score > minimum_score {
        Ok(())
    } else {
        Err(anyhow!(format!(
            "For address: {}, Gitcoin passport score is too low ({}). Minimum value is: {}",
            address_from_signature, score, minimum_score
        )))
    }
}

async fn get_gitcoin_score_for_address(rpc_url: &str, address: Address) -> Result<f64> {
    let provider = ProviderBuilder::new().on_builtin(rpc_url).await?;

    let call = getScoreCall { user: address }.abi_encode();
    let input = Bytes::from(call);
    let tx = TransactionRequest::default()
        .with_to(GITCOIN_PASSPORT_DECODER)
        .with_input(input);

    match provider.call(&tx).block(BlockId::latest()).await {
        Ok(response) => Ok(calculate_score(response)),
        Err(_) => Ok(0.0),
    }
}

fn calculate_score(response: Bytes) -> f64 {
    let score = U256::from_str(&response.to_string()).unwrap().to::<u128>();
    score as f64 / 100.0
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

    use alloy::node_bindings::{Anvil, AnvilInstance};

    use super::*;

    const SIGNATURE_HASH: &str = "0xeeec9a87a01977a48fa5bac97d2f1c67d83905ac378573d6749ae078b76b3ef078f2187ef9cbf4eaf2069066fdc32b07823508db3871ab07f32d30137c0140a81c";
    const SIGNATURE_MESSAGE: &[u8] = b"KYC";

    #[actix_rt::test]
    async fn getting_score_from_gitcoin_passport_decoder_works() {
        let anvil = init_anvil();

        let test_address = address!("0922B44805CB5D90F35F3b9781aFf83b47D722d3");
        let result = get_gitcoin_score_for_address(&anvil.endpoint(), test_address).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        // getScore returns 301058 for input address on block 120144603
        assert_eq!(result, 3010.58);
    }

    #[actix_rt::test]
    async fn getting_score_with_not_verified_score_should_return_zero() {
        let anvil = init_anvil();

        let test_address = address!("79E214f3Aa3101997ffE810a57eCA4586e3bdeb2");
        let result = get_gitcoin_score_for_address(&anvil.endpoint(), test_address).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result, 0.0);
    }

    #[actix_rt::test]
    async fn verifier_returns_valid_address_for_valid_message() {
        let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

        let address_from_signature =
            get_address_from_signature(&signature, SIGNATURE_MESSAGE).unwrap();

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

    fn init_anvil() -> AnvilInstance {
        let rpc_url = "https://mainnet.optimism.io";
        let block_number = 120144603;

        let anvil = Anvil::new()
            .fork(rpc_url)
            .fork_block_number(block_number)
            .try_spawn()
            .unwrap();

        anvil
    }
}
