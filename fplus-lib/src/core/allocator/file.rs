use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AllocatorModel {
    pub slug: String,  
    pub organization: String,
    pub address: String,
    pub application: Application,
    pub common_ui_install_id: u64, 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    pub github_handles: Vec<String>
}