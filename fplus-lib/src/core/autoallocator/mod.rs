use self::file::Autoallocation;
use alloy::primitives::Address;
use chrono::Utc;

pub mod file;

impl Autoallocation {
    pub fn new(evm_wallet_address: Address) -> Self {
        Self {
            evm_wallet_address,
            last_allocation: Utc::now().to_string(),
        }
    }
}
