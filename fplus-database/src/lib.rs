pub mod models;
pub mod database;
pub mod config;

use sea_orm::{Database, DatabaseConnection, DbErr};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::config::get_env_var_or_default;

/**
 * The global database connection
 */
static DB_CONN: Lazy<Mutex<Option<DatabaseConnection>>> = Lazy::new(|| Mutex::new(None));

/**
 * Initialize the database (Just for testing purposes, not used in the actual application, as dotenv is called in the main function of the application)
 * 
 * # Returns
 * @return () - The result of the operation
 */
pub fn init() {
    dotenv::dotenv().ok();
}

/**
 * Establish a connection to the database
 *  
 * # Returns
 * @return Result<DatabaseConnection, sea_orm::DbErr> - The result of the operation
 */
pub async fn setup() -> Result<(), DbErr> {
    let database_url = get_env_var_or_default("DB_URL", "");
    let db_conn = Database::connect(&database_url).await?;
    let mut db_conn_global = DB_CONN.lock().unwrap();
    *db_conn_global = Some(db_conn);
    Ok(())
}

/**
 * Get a reference to the established database connection
 * 
 * # Returns
 * @return Result<DatabaseConnection, &'static str> - The database connection or an error message
 */
pub async fn get_database_connection() -> Result<DatabaseConnection, DbErr> {
    let db_conn = DB_CONN.lock().unwrap();
    if let Some(ref conn) = *db_conn {
        Ok(conn.clone())
    } else {
        Err(DbErr::Custom("Database connection is not established".into()))
    }
}
#[cfg(test)]
mod tests {
    
    use super::*;
    use sea_orm::entity::*;
    use tokio;

    /**
     * Sets up the initial test environment (database connection and env variables)
     */
    async fn setup_test_environment() {
        init();
        setup().await.expect("Failed to setup database connection.");
    }

    /**
     * Test the establish_connection function
     * 
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    async fn test_establish_connection_with_env_url() {
        init();
        let connection_result = setup().await;
        assert!(connection_result.is_ok());
    }

    /**
     * Test the create_allocator function
     * 
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    async fn test_create_allocator() {
        setup_test_environment().await;

        let new_allocator = models::allocators::ActiveModel {
            owner: Set("test_owner".to_string()),
            repo: Set("test_repo".to_string()),
            installation_id: Set(Some(123)),
            multisig_address: Set(Some("0x1234567890".to_string())),
            verifiers_gh_handles: Set(Some("test_verifier_1, test_verifier_2".to_string())),
            ..Default::default()
        };

        let result = database::create_allocator(new_allocator).await;
        assert!(result.is_ok());
    }

    /**
     * Test the get_allocators function
     * 
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    async fn test_get_allocators() {
        setup_test_environment().await;
        
        let result = database::get_allocators().await;
        assert!(result.is_ok());
    }

    /**
     * Test the update_allocator function
     * 
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    async fn test_update_allocator() {
        setup_test_environment().await;

        let allocator = database::get_allocator("test_owner", "test_repo").await.expect("Failed to get allocator").expect("No allocator found");

        let updated_allocator = models::allocators::ActiveModel {
            id: Set(allocator.id),
            multisig_address: Set(Some("0x123456789".to_string())),
            verifiers_gh_handles: Set(Some("test_verifier_1, test_verifier_2, test_verifier_3".to_string())),
            ..Default::default()
        };

        let result = database::update_allocator(&allocator.owner, &allocator.repo, updated_allocator).await;
        assert!(result.is_ok());
    }

    /**
     * Test the get_allocator function
     * 
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    async fn test_get_allocator() {
        setup_test_environment().await;

        let allocator = database::get_allocators().await.expect("Failed to get allocators").pop().expect("No allocators found");

        let result = database::get_allocator(&allocator.owner, &allocator.repo).await;
        assert!(result.is_ok());
    }

    /**
     * Test the delete_allocator function
     * 
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    async fn test_delete_allocator() {
        setup_test_environment().await;

        let allocator = database::get_allocators().await.expect("Failed to get allocators").pop().expect("No allocators found");

        let result = database::delete_allocator(&allocator.owner, &allocator.repo).await;
        assert!(result.is_ok());
    }

}