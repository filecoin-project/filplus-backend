use serde::{Deserialize, Serialize};

pub type StateReadStateResponse = JSONRPCResponse<StateReadStateResult>;
pub type StateVerifierStatusResponse = JSONRPCResponse<StateVerifierStatusResult>;
pub type StateVerifiedClientStatusResponse = JSONRPCResponse<StateVerifiedClientStatusResult>;

#[derive(Debug, Serialize, Deserialize)]
pub struct JSONRPCResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: u64,
}

pub type StateVerifierStatusResult = String;
pub type StateVerifiedClientStatusResult = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct StateReadStateResult {
    #[serde(rename = "Balance")]
    pub balance: String,
    #[serde(rename = "Code")]
    pub code: Code,
    #[serde(rename = "State")]
    pub state: MultisigState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    #[serde(rename = "/")]
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultisigState {
    #[serde(rename = "Signers")]
    pub signers: Vec<String>,
    #[serde(rename = "NumApprovalsThreshold")]
    pub num_approvals_threshold: u64,
    #[serde(rename = "NextTxnID")]
    pub next_txn_id: u64,
    #[serde(rename = "InitialBalance")]
    pub initial_balance: String,
    #[serde(rename = "StartEpoch")]
    pub start_epoch: u64,
    #[serde(rename = "UnlockDuration")]
    pub unlock_duration: u64,
    #[serde(rename = "PendingTxns")]
    pub pending_txns: Code,
}
