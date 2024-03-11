use std::env;
use log::error;

/**
 * Get an environment variable or exit the program if not set
 * 
 * # Arguments
 * * `key` - The environment variable key
 * 
 * # Returns
 * * The value of the environment variable
 * 
 * # Panics
 * * Exits the program if the environment variable is not set
 */
pub fn get_env_or_throw(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            error!("Environment variable '{}' not set. Exiting program.", key);
            std::process::exit(1);
        }
    }
}