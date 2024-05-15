use std::str::FromStr;

use alloy::{
    network::TransactionBuilder,
    node_bindings::Anvil,
    primitives::{address, Address, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    signers::Signature,
    sol,
    sol_types::SolCall,
};

use anyhow::{anyhow, Result};

use crate::core::SignatureRequest;

const GITCOIN_PASSPORT_DECODER: Address = address!("5558D441779Eca04A329BcD6b47830D2C6607769");

sol!(
    #[allow(missing_docs)]
    function getScore(address user) view returns (uint256);
);

pub async fn verify_on_gitcoin(signature: &str, message: &[u8]) -> Result<()> {
    let signature = Signature::from_str(signature)?;
    let address_from_signature = get_address_from_signature(&signature, message)?;

    let anvil = Anvil::new()
        .fork("https://mainnet.optimism.io")
        .try_spawn()
        .unwrap();

    let rpc_url = anvil.endpoint().parse().unwrap();
    let provider = ProviderBuilder::new().on_http(rpc_url);

    let call = getScoreCall {
        user: address_from_signature,
    }
    .abi_encode();
    let input = Bytes::from(call);
    let tx = TransactionRequest::default()
        .with_to(GITCOIN_PASSPORT_DECODER)
        .with_input(input);

    let response = provider.call(&tx).block(BlockId::latest()).await?;

    Ok(())
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
        let result = verify_on_gitcoin(SIGNATURE_HASH, SIGNATURE_MESSAGE).await;
        assert!(result.is_ok());
    }

    #[actix_rt::test]
    async fn verifier_returns_valid_address_for_valid_message() {
        let message = b"KYC";
        let signature = Signature::from_str(SIGNATURE_HASH).unwrap();

        let address_from_signature = get_address_from_signature(&signature, message).unwrap();

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
