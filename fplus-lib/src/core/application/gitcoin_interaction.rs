use std::str::FromStr;
use serde::Deserialize;

use alloy::{
    network::TransactionBuilder,
    primitives::{address, Address, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    signers::Signature,
    sol,
    sol_types::{SolCall, eip712_domain, SolStruct},
};

use anyhow::{anyhow, Result, ensure};
use fplus_database::config::get_env_or_throw;
use crate::config::get_env_var_or_default;


sol! {
    #[allow(missing_docs)]
    function getScore(address user) view returns (uint256);
    
    #[derive(Deserialize)]
    struct KycApproval {
        string message;
        string client_id;
        string issued_at;
        string expires_at;
        string allocator_repo_name;
        string allocator_repo_owner;
    }
}

pub async fn verify_on_gitcoin(message: &KycApproval, signature: &str) -> Result<()> {
    let address_from_signature = get_address_from_signature(&message, &signature)?;

    let rpc_url = format!("{}", get_env_or_throw("RPC_URL"));
    let score = get_gitcoin_score_for_address(&rpc_url, address_from_signature).await?;

    let minimum_score = get_env_var_or_default("GITCOIN_MINIMUM_SCORE");
    let minimum_score = minimum_score.parse::<f64>().unwrap();

    ensure!(score > minimum_score, 
        anyhow!(format!(
            "For address: {}, Gitcoin passport score is too low ({}). Minimum value is: {}",
            address_from_signature, score, minimum_score
    )));
    Ok(())
}

async fn get_gitcoin_score_for_address(rpc_url: &str, address: Address) -> Result<f64> {
    let provider = ProviderBuilder::new().on_builtin(rpc_url).await?;
    let gitcoin_passport_decoder = Address::from_str(
        &get_env_var_or_default("GITCOIN_PASSPORT_DECODER"))?;
    let call = getScoreCall { user: address }.abi_encode();
    let input = Bytes::from(call);
    let tx = TransactionRequest::default()
        .with_to(gitcoin_passport_decoder)
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
    message: &KycApproval, signature: &str
) -> Result<Address, alloy::signers::Error> {
    let domain = eip712_domain! {
        name: "Fil+ KYC",
        version: "1",
        chain_id: get_env_var_or_default("PASSPORT_VERIFIER_CHAIN_ID").parse().map_err(|_| alloy::signers::Error::Other("Parse chain Id to u64 failed".into()))?, // Filecoin Chain Id 
        verifying_contract: address!("0000000000000000000000000000000000000000"),
    };
    let hash = message.eip712_signing_hash(&domain);
    let signature = Signature::from_str(&signature)?;
    Ok(signature.recover_address_from_prehash(&hash).unwrap())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::node_bindings::{Anvil, AnvilInstance};

    use super::*;

    const SIGNATURE_HASH: &str = "0xa3fbeb584a3c4a7f2eface5f0255fe7de7793a07d64e9289f28bfd0536e196b4613f1ad628b25ca8f73bd821844d4918a20572e17b6779b521848afa76a7fa101b";

    #[actix_rt::test]
    async fn getting_score_from_gitcoin_passport_decoder_works() {
        let anvil = init_anvil();

        let test_address = address!("907F988126Fd7e3BB5F46412b6Db6775B3dC3F9b");
        let result = get_gitcoin_score_for_address(&anvil.endpoint(), test_address).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        // getScore returns 10410 for input address on block 12507578
        assert_eq!(result, 104.09999999999999);
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

    // #[actix_rt::test]
    // async fn verifier_returns_valid_address_for_valid_message() {
    //     let signature_message: KycApproval = KycApproval {
    //         message: "Connect your Fil+ application with your wallet and give access to your Gitcoin passport".into(),
    //         client_id: "test".into(),
    //         issued_at: "Fri May 24 2024 13:43:37 GMT+0200 (Central European Summer Time)".into(),
    //         expires_at: "Sat May 25 2024 13:43:37 GMT+0200 (Central European Summer Time)".into(),
    //         allocator_repo_name: "test".into(),
    //         allocator_repo_owner: "test".into()
    //     };
    //     let address_from_signature =
    //         get_address_from_signature(&signature_message, &SIGNATURE_HASH).unwrap();

    //     let expected_address= address!("907F988126Fd7e3BB5F46412b6Db6775B3dC3F9b");

    //     assert_eq!(expected_address, address_from_signature);
    // }

    // #[actix_rt::test]
    // async fn verifier_returns_invalid_address_for_invalid_message() {
    //     let message = b"Invalid message";
    //     let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

    //     let address_from_signature = get_address_from_signature(message, &signature).unwrap();

    //     let expected_address =
    //         Address::from_str("0x79e214f3aa3101997ffe810a57eca4586e3bdeb2").unwrap();

    //     assert_ne!(expected_address, address_from_signature);
    // }

    fn init_anvil() -> AnvilInstance {
        let rpc_url = "https://sepolia.optimism.io/";
        let block_number = 12507578;

        let anvil = Anvil::new()
            .fork(rpc_url)
            .fork_block_number(block_number)
            .try_spawn()
            .unwrap();

        anvil
    }
}
