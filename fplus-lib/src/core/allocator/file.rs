use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AllocatorModel {
    #[serde(rename = "slug")]
    pub repo: String,  
    #[serde(rename = "organization")]
    pub owner: String,
    #[serde(rename = "address")]
    pub multisig_address: String,
    pub application: Application,
    #[serde(rename = "common_ui_install_id")]
    pub installation_id: u64, 
    #[serde(rename = "multisig_threshold")]
    pub multisig_threshold: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    #[serde(rename = "github_handles")]
    pub verifiers_gh_handles: Vec<String>
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
