use serde::Deserialize;
use std::str::FromStr;

use alloy::{
    network::TransactionBuilder,
    primitives::{address, Address, Bytes, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    signers::Signature,
    sol,
    sol_types::{eip712_domain, SolCall, SolStruct},
};

use crate::config::get_env_var_or_default;
use crate::error::LDNError;
use anyhow::Result;

pub trait ExpirableSolStruct: SolStruct {
    fn get_expires_at(&self) -> &str;
    fn get_issued_at(&self) -> &str;
}

sol! {
    #[allow(missing_docs)]
    function getScore(address user) view returns (uint256);

    #[derive(Deserialize)]
    struct KycApproval {
        string message;
        string client_id;
        string allocator_repo_name;
        string allocator_repo_owner;
        string issued_at;
        string expires_at;
    }

    #[derive(Deserialize)]
    struct KycAutoallocationApproval {
        string message;
        string client_fil_address;
        string issued_at;
        string expires_at;
    }
}

impl ExpirableSolStruct for KycApproval {
    fn get_expires_at(&self) -> &str {
        &self.expires_at
    }

    fn get_issued_at(&self) -> &str {
        &self.issued_at
    }
}

impl ExpirableSolStruct for KycAutoallocationApproval {
    fn get_expires_at(&self) -> &str {
        &self.expires_at
    }

    fn get_issued_at(&self) -> &str {
        &self.issued_at
    }
}

pub async fn verify_on_gitcoin(address_from_signature: &Address) -> Result<f64, LDNError> {
    let rpc_url = get_env_var_or_default("RPC_URL");
    let score = get_gitcoin_score_for_address(&rpc_url, *address_from_signature).await?;

    let minimum_score = get_env_var_or_default("GITCOIN_MINIMUM_SCORE");
    let minimum_score = minimum_score
        .parse::<f64>()
        .map_err(|e| LDNError::New(format!("Parse minimum score to f64 failed: {e:?}")))?;

    if score <= minimum_score {
        return Err(LDNError::New(format!(
            "For address: {}, Gitcoin passport score is too low ({}). Minimum value is: {}",
            address_from_signature, score, minimum_score
        )));
    }
    Ok(score)
}

async fn get_gitcoin_score_for_address(rpc_url: &str, address: Address) -> Result<f64, LDNError> {
    let provider = ProviderBuilder::new()
        .on_builtin(rpc_url)
        .await
        .map_err(|e| LDNError::New(format!("Invalid RPC URL: {e:?}")))?;
    let gitcoin_passport_decoder =
        Address::from_str(&get_env_var_or_default("GITCOIN_PASSPORT_DECODER"))
            .map_err(|e| LDNError::New(format!("Parse GITCOIN PASSPORT DECODER failed: {e:?}")))?;
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
    score as f64 / 10000.0
}

pub fn get_address_from_signature<T: SolStruct>(
    message: &T,
    signature: &str,
) -> Result<Address, LDNError> {
    let domain = eip712_domain! {
        name: "Fil+ KYC",
        version: "1",
        chain_id: get_env_var_or_default("PASSPORT_VERIFIER_CHAIN_ID").parse().map_err(|_| LDNError::New("Parse chain Id to u64 failed".to_string()))?, // Filecoin Chain Id
        verifying_contract: address!("0000000000000000000000000000000000000000"),
    };
    let hash = message.eip712_signing_hash(&domain);
    let signature = Signature::from_str(signature)
        .map_err(|e| LDNError::New(format!("Signature parsing failed: {e:?}")))?;
    signature
        .recover_address_from_prehash(&hash)
        .map_err(|e| LDNError::New(format!("Recover address from prehash failed: {e:?}")))
}

#[cfg(test)]
#[cfg(feature = "online-tests")]
mod tests {

    use alloy::node_bindings::{Anvil, AnvilInstance};

    use super::*;

    const SIGNATURE: &str = "0x0d65d92f0f6774ca40a232422329421183dca5479a17b552a9f2d98ad0bb22ac65618c83061d988cd657c239754253bf66ce6e169252710894041b345797aaa21b";

    #[actix_rt::test]
    async fn getting_score_from_gitcoin_passport_decoder_works() {
        env::set_var(
            "GITCOIN_PASSPORT_DECODER",
            "e53C60F8069C2f0c3a84F9B3DB5cf56f3100ba56",
        );
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
        env::set_var(
            "GITCOIN_PASSPORT_DECODER",
            "e53C60F8069C2f0c3a84F9B3DB5cf56f3100ba56",
        );
        let anvil = init_anvil();

        let test_address = address!("79E214f3Aa3101997ffE810a57eCA4586e3bdeb2");
        let result = get_gitcoin_score_for_address(&anvil.endpoint(), test_address).await;

        assert!(result.is_ok());
        let result = result.unwrap();

        assert_eq!(result, 0.0);
    }

    #[actix_rt::test]
    async fn verifier_returns_valid_address_for_valid_message() {
        env::set_var("PASSPORT_VERIFIER_CHAIN_ID", "11155420");
        let signature_message: KycApproval = KycApproval {
            message: "Connect your Fil+ application with your wallet and give access to your Gitcoin passport".into(),
            client_id: "test".into(),
            issued_at: "2024-05-28T09:02:51.126Z".into(),
            expires_at: "2024-05-29T09:02:51.126Z".into(),
            allocator_repo_name: "test".into(),
            allocator_repo_owner: "test".into()
        };
        let address_from_signature =
            get_address_from_signature(&signature_message, &SIGNATURE).unwrap();

        let expected_address = address!("7638462f3a5f2cdb49609bf4947ae396f9088949");

        assert_eq!(expected_address, address_from_signature);
    }

    #[actix_rt::test]
    async fn verifier_returns_invalid_address_for_invalid_message() {
        env::set_var("PASSPORT_VERIFIER_CHAIN_ID", "11155420");
        let message: KycApproval = KycApproval {
            message: "Connect your Fil+ application with your wallet and give access to your Gitcoin passport".into(),
            client_id: "test".into(),
            issued_at: "2024-05-28T09:02:51.126Z".into(),
            expires_at: "2024-05-29T09:02:51.126Z".into(),
            allocator_repo_name: "test".into(),
            allocator_repo_owner: "test".into()
        };

        let address_from_signature = get_address_from_signature(&message, &SIGNATURE).unwrap();

        let expected_address =
            Address::from_str("0x79e214f3aa3101997ffe810a57eca4586e3bdeb2").unwrap();

        assert_ne!(expected_address, address_from_signature);
    }

    fn init_anvil() -> AnvilInstance {
        let rpc_url = "https://sepolia.optimism.io/";
        let block_number = 12507578;

        Anvil::new()
            .fork(rpc_url)
            .fork_block_number(block_number)
            .try_spawn()
            .unwrap()
    }
}
