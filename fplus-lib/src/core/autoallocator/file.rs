use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Autoallocation {
    #[serde(rename = "EVM Wallet Address")]
    pub evm_wallet_address: Address,
    #[serde(rename = "Last Allocation")]
    pub last_allocation: String,
}
