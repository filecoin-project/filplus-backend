use log::warn;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

pub fn default_env_vars() -> &'static HashMap<&'static str, &'static str> {
    static DEFAULTS: OnceCell<HashMap<&'static str, &'static str>> = OnceCell::new();
    DEFAULTS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("GITHUB_OWNER", "filecoin-project");
        m.insert("GITHUB_REPO", "filecoin-plus-falcon");
        m.insert("GITHUB_APP_ID", "826129");
        m.insert("GITHUB_INSTALLATION_ID", "48299904");
        m.insert(
            "RUST_LOG",
            "info,actix_web::middleware::logger=debug,sqlx::query=warn",
        );
        m.insert("RUST_BACKTRACE", "1");
        m.insert("DB_URL", "");
        m.insert("ALLOCATOR_GOVERNANCE_OWNER", "fidlabs");
        m.insert("ALLOCATOR_GOVERNANCE_REPO", "Allocator-Governance-Staging");
        m.insert("ALLOCATOR_TEMPLATE_OWNER", "fidlabs");
        m.insert("ALLOCATOR_TEMPLATE_REPO", "allocator-template");
        m.insert("BOT_USER", "filplus-allocators-staging-bot[bot]");
        m.insert(
            "BACKEND_URL",
            "https://fp-core.dp04sa0tdc6pk.us-east-1.cs.amazonlightsail.com",
        );
        m.insert("FILPLUS_ENV", "staging");
        m.insert("GLIF_NODE_URL", "https://api.node.glif.io/rpc/v1");
        m.insert("ISSUE_TEMPLATE_VERSION", "1.3");
        m.insert(
            "GITCOIN_PASSPORT_DECODER",
            "5558D441779Eca04A329BcD6b47830D2C6607769",
        );
        m.insert("PASSPORT_VERIFIER_CHAIN_ID", "10");
        m.insert("GITCOIN_MINIMUM_SCORE", "30");
        m.insert("KYC_URL", "https://kyc.allocator.tech");
        m.insert("RPC_URL", "https://mainnet.optimism.io");
        m.insert("DMOB_API_URL", "https://api.datacapstats.io");
        m.insert("DMOB_API_KEY", "5c993a17-7b18-4ead-a8a8-89dad981d87e");
        m.insert("DAYS_TO_NEXT_AUTOALLOCATION", "14");
        m.insert(
            "ALLOCATOR_CONTRACT_ADDRESS",
            "0x640bD4be149f40714D95aBcD414338bc7CfF39a3",
        );
        m.insert("AUTOALLOCATION_AMOUNT", "1099511627776"); // 1099511627776 B == 1 TiB
        m.insert("TFIDF_THRESHOLD", "0.4");
        m.insert("LEVENSHTEIN_THRESHOLD", "8");
        m
    })
}

pub fn get_env_var_or_default(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            let defaults = default_env_vars();
            let default = defaults.get(key).unwrap_or(&"");
            warn!("{} not set, using default value: {}", key, default);
            default.to_string()
        }
    }
}
