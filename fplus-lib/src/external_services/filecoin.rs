use futures::future::join_all;
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

    match get_public_addresses_from_ids(signer_ids).await {
        Ok(signers) => Ok(signers),
        Err(e) => Err(e),
    }
}

pub async fn get_public_addresses_from_ids(signer_ids: Vec<String>) -> Result<Vec<String>, String> {
    let node_url = get_env_var_or_default("GLIF_NODE_URL");
    let client = reqwest::Client::new();

    let futures = signer_ids.into_iter().map(|id| {
        let client = client.clone();
        let node_url = node_url.clone();
        async move {
            let body = json!({
                "jsonrpc": "2.0",
                "method": "Filecoin.StateAccountKey",
                "params": [id, null],
                "id": 1
            });

            match client.post(&node_url)
                .json(&body)
                .send().await {
                    Ok(resp) => match resp.json::<serde_json::Value>().await {
                        Ok(body) => {
                            println!("Got public address from ID {}: {:?}", id, body);
                            body["result"].as_str().map(|s| s.to_string())
                        },

                        Err(err) => {
                            print!( "Error getting public address from ID {}: {:?}", id, err);
                            None
                        },
                    },
                    Err(_) => None,
                }
        }
    });

    let results = join_all(futures).await;
    results.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| "Failed to get public addresses".to_string())
}