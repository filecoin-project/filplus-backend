use log::warn;

/**
 * Get an environment variable or a default value
 * 
 * # Arguments
 * @param key: &str - The environment variable key
 * @param default: &str - The default value
 * 
 * # Returns
 * @return String - The value of the environment variable or the default value
 */
pub fn get_env_var_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            warn!("{} not set, using default value: {}", key, default);
            default.to_string()
        }
    }
}
