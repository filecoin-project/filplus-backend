use serde_json::json;

use crate::{
    config::get_env_var_or_default,
    models::filecoin::{
        StateReadStateResponse, StateVerifiedClientStatusResponse, StateVerifierStatusResponse,
    },
};

pub async fn state_get_state(
    actor_address: &str,
) -> Result<StateReadStateResponse, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.StateReadState",
        "params": [actor_address, null],
        "id": 1
    });

    let request = client.post(&node_url).json(&body);

    let response = request
        .send()
        .await?
        .json::<StateReadStateResponse>()
        .await?;
    Ok(response)
}

pub async fn get_multisig_threshold_for_actor(actor_address: &str) -> Result<u64, reqwest::Error> {
    let actor_state_info = state_get_state(actor_address).await?;
    Ok(actor_state_info.result.state.num_approvals_threshold)
}

pub async fn get_allowance_for_address_direct(address: &str) -> Result<String, reqwest::Error> {
    let allowance = get_allowance_for_client(address).await;
    if let Ok(allowance) = allowance {
        if allowance != "0" {
            return Ok(allowance);
        }
    }
    get_allowance_for_verifier(address).await
}

pub async fn get_allowance_for_verifier(address: &str) -> Result<String, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.StateVerifierStatus",
        "params": [address, null],
        "id": 1
    });

    let request = client.post(&node_url).json(&body);

    let response = request
        .send()
        .await?
        .json::<StateVerifierStatusResponse>()
        .await?;
    Ok(response.result)
}

pub async fn get_allowance_for_client(address: &str) -> Result<String, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.StateVerifiedClientStatus",
        "params": [address, null],
        "id": 1
    });

    let request = client.post(&node_url).json(&body);

    let response = request
        .send()
        .await?
        .json::<StateVerifiedClientStatusResponse>()
        .await?;
    Ok(response.result)
}

pub async fn filecoin_address_to_evm_address(address: &str) -> Result<String, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.FilecoinAddressToEthAddress",
        "params": [address, null],
        "id": 0
    });

    let request = client.post(&node_url).json(&body);

    let response = request
        .send()
        .await?
        .json::<StateVerifiedClientStatusResponse>()
        .await?;
    Ok(response.result)
}

pub async fn evm_address_to_filecoin_address(address: &str) -> Result<String, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.EthAddressToFilecoinAddress",
        "params": [address],
        "id": 0
    });

    let request = client.post(&node_url).json(&body);

    let response = request
        .send()
        .await?
        .json::<StateVerifiedClientStatusResponse>()
        .await?;
    Ok(response.result)
}
