use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AllocatorModel {
    pub slug: String,
    pub organization: String,
    pub multisig_address: String,
    pub verifiers: Vec<String>,
    pub installation_id: u64,
}