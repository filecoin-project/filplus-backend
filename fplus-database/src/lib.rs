pub mod models;
pub mod database;
pub mod config;

use sea_orm::DatabaseConnection;
use crate::config::get_env_var_or_default;

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
pub async fn setup() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let database_url = get_env_var_or_default("DB_URL", "");
    sea_orm::Database::connect(&database_url).await
}

#[cfg(test)]
mod tests {
    
    use super::*;
    use sea_orm::entity::*;
    use tokio;

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
        init();
        let conn = setup().await.expect("Failed to connect to the database");

        let new_allocator = models::allocators::ActiveModel {
            owner: Set("test_owner".to_string()),
            repo: Set("test_repo".to_string()),
            installation_id: Set(Some(123)),
            multisig_address: Set(Some("0x1234567890".to_string())),
            verifiers_gh_handles: Set(Some("test_verifier_1, test_verifier_2".to_string())),
            ..Default::default()
        };

        let result = database::create_allocator(&conn, new_allocator).await;
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
        init();
        let conn = setup().await.expect("Failed to connect to the database");

        let result = database::get_allocators(&conn).await;
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
        init();
        let conn = setup().await.expect("Failed to connect to the database");

        let allocator = database::get_allocator(&conn, "test_owner", "test_repo").await.expect("Failed to get allocator").expect("No allocator found");

        let updated_allocator = models::allocators::ActiveModel {
            id: Set(allocator.id),
            multisig_address: Set(Some("0x123456789".to_string())),
            verifiers_gh_handles: Set(Some("test_verifier_1, test_verifier_2, test_verifier_3".to_string())),
            ..Default::default()
        };

        let result = database::update_allocator(&conn, &allocator.owner, &allocator.repo, updated_allocator).await;
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
        init();
        let conn = setup().await.expect("Failed to connect to the database");

        let allocator = database::get_allocators(&conn).await.expect("Failed to get allocators").pop().expect("No allocators found");

        let result = database::get_allocator(&conn, &allocator.owner, &allocator.repo).await;
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
        init();
        let conn = setup().await.expect("Failed to connect to the database");

        let allocator = database::get_allocators(&conn).await.expect("Failed to get allocators").pop().expect("No allocators found");

        let result = database::delete_allocator(&conn, &allocator.owner, &allocator.repo).await;
        assert!(result.is_ok());
    }

}