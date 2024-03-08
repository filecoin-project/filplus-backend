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