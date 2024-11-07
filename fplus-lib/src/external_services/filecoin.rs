use serde_json::json;

use crate::{
    config::get_env_var_or_default,
    models::filecoin::{
        StateReadStateResponse, StateVerifiedClientStatusResponse, StateVerifierStatusResponse,
        VerifiedClientResponse,
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

pub async fn get_allowance_for_address(address: &str) -> Result<String, reqwest::Error> {
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

pub async fn get_client_allocation(
    address: &str,
) -> Result<VerifiedClientResponse, reqwest::Error> {
    let api_url = get_env_var_or_default("DATACAPSTATS_API_URL");
    let url = format!("{}/getVerifiedClients?filter={}", api_url, address);

    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<VerifiedClientResponse>()
        .await?;
    Ok(response)
}
