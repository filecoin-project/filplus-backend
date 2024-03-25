use serde_json::json;

use crate::{config::get_env_var_or_default, models::filecoin::StateReadStateResponse};

pub async fn state_get_state(actor_address: &str) -> Result<StateReadStateResponse, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.StateReadState",
        "params": [actor_address, null],
        "id": 1
    });

    let request = client.post(&node_url).json(&body);

    let response = request.send().await?.json::<StateReadStateResponse>().await?;
    Ok(response)
}

pub async fn get_multisig_threshold_for_actor(actor_address: &str) -> Result<u64, reqwest::Error> {
    let actor_state_info = state_get_state(actor_address).await?;
    Ok(actor_state_info.result.state.num_approvals_threshold)
}

pub async fn get_multisig_signers_for_msig(actor_address: &str) -> Result<Vec<String>, String> {
    let actor_state_info = state_get_state(actor_address).await.map_err(|e| e.to_string())?;
    let signer_ids = actor_state_info.result.state.signers;

    get_public_addresses_from_ids(signer_ids).await
}

pub async fn get_public_addresses_from_ids(signer_ids: Vec<String>) -> Result<Vec<String>, String> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");
    let client = reqwest::Client::new();
    let mut public_addresses = Vec::new();

    for id in signer_ids {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "Filecoin.StateAccountKey",
            "params": [id, null],
            "id": 1
        });

        let response = match client.post(&node_url)
            .json(&body)
            .send().await {
                Ok(resp) => resp,
                Err(_) => return Err("Failed to send request".to_string()),
            };

        let response_body = match response.json::<serde_json::Value>().await {
            Ok(body) => body,
            Err(_) => return Err("Failed to parse response".to_string()),
        };

        if let Some(address) = response_body["result"].as_str() {
            public_addresses.push(address.to_string());
        } else {
            return Err("Failed to get public address".to_string());
        }
    }

    Ok(public_addresses)
}