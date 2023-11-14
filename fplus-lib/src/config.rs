use log::warn;

pub fn get_env_var_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            warn!("{} not set, using default value: {}", key, default);
            default.to_string()
        }
    }
}
