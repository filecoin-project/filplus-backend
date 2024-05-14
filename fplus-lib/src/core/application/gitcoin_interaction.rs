use std::{ops::Add, str::FromStr};

use alloy::{
    network::TransactionBuilder,
    node_bindings::Anvil,
    primitives::{address, Address, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{BlockId, TransactionRequest},
    sol,
    sol_types::SolCall,
};

sol!(
    #[allow(missing_docs)]
    function getScore(address user) view returns (uint256);
);

#[cfg(test)]
mod tests {

    use super::*;

    const GITCOIN_PASSPORT_DECODER: Address = address!("5558D441779Eca04A329BcD6b47830D2C6607769");
    const TEST_ADDRESS: Address = address!("c17611c85ea2216ee343763035eeb9c104b4066f");

    #[actix_rt::test]
    async fn getting_score_from_gitcoin_passport_decoder_works() {
        let anvil = Anvil::new()
            .fork("https://mainnet.optimism.io")
            .try_spawn()
            .unwrap();

        let rpc_url = anvil.endpoint().parse().unwrap();
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let call = getScoreCall { user: TEST_ADDRESS }.abi_encode();
        let input = Bytes::from(call);

        let tx = TransactionRequest::default()
            .with_to(GITCOIN_PASSPORT_DECODER)
            .with_input(input);

        let response = provider.call(&tx).block(BlockId::latest()).await;

        assert!(response.is_ok());

        assert_eq!(4, 2 + 2);
    }
}
