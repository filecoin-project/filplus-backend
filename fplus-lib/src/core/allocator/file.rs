use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllocatorModel {
    pub application: Application,
    pub multisig_threshold: Option<i32>,
    pub pathway_addresses: AllocatorModelPathwayAddresses,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllocatorModelPathwayAddresses {
    pub msig: String,
    pub signer: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Application {
    #[serde(rename = "github_handles")]
    pub verifiers_gh_handles: Vec<String>,
    pub allocation_bookkeeping: String,
    pub allocation_amount: Option<AllocationAmount>,
    pub tooling: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllocationAmount {
    pub amount_type: Option<String>,
    pub quantity_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}

#[derive(Deserialize, Debug)]
pub struct Installation {
    pub id: u64,
}

#[derive(Deserialize)]
pub struct AccessTokenResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct Repository {
    pub name: String,
    pub owner: Owner,
}

#[derive(Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Deserialize)]
pub struct RepositoriesResponse {
    pub repositories: Vec<Repository>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstallationRepositories {
    pub installation_id: u64,
    pub repositories: Vec<RepositoryInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepositoryInfo {
    pub slug: String,
    pub owner: String,
}
