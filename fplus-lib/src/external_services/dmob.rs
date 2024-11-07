use crate::config::get_env_var_or_default;
use crate::models::dmob::VerifiedClientResponse;

pub async fn get_client_allocation(
    address: &str,
) -> Result<VerifiedClientResponse, reqwest::Error> {
    let api_url = get_env_var_or_default("DMOB_API_URL");
    let url = format!("{}/api/getVerifiedClients?filter={}", api_url, address);

    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<VerifiedClientResponse>()
        .await?;
    Ok(response)
}
