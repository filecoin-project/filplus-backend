use serde_json::json;

use crate::{config::get_env_var_or_default, models::filecoin::StateReadStateResponse};

pub async fn state_get_state(actor_address: &str) -> Result<StateReadStateResponse, reqwest::Error> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");
    let node_token = get_env_var_or_default("GLIF_NODE_TOKEN");

    let client = reqwest::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "Filecoin.StateReadState",
        "params": [actor_address, null],
        "id": 1
    });

    let mut request = client.post(&node_url).json(&body);
    if !node_token.is_empty() {
        request = request.bearer_auth(&node_token);
    }

    let response = request.send().await?.json::<StateReadStateResponse>().await?;
    Ok(response)
}

pub async fn get_multisig_threshold_for_actor(actor_address: &str) -> Result<u64, reqwest::Error> {
    let actor_state_info = state_get_state(actor_address).await?;
    Ok(actor_state_info.result.state.num_approvals_threshold)
}