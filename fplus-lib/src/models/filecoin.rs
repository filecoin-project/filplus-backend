use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedClientResponse {
    #[serde(deserialize_with = "number_to_string")]
    pub count: Option<String>,
}

fn number_to_string<'de, D>(de: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let helper: Value = Deserialize::deserialize(de)?;

    match helper {
        Value::Number(n) => Ok(n
            .as_u64()
            .filter(|&number| number != 0)
            .map(|_| n.to_string())),
        Value::String(s) => Ok(Some(s)),
        _ => Ok(None),
    }
}
