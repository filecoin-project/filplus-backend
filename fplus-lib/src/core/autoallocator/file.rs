use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Autoallocation {
    pub evm_wallet_address: Address,
    pub last_allocation: String,
}
