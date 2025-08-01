pub mod config;
pub mod database;
pub mod models;
mod types;

use crate::config::get_env_or_throw;
use once_cell::sync::Lazy;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::sync::Mutex;
use types::DbConnectParams;

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
    let database_url = std::env::var("DB_URL").unwrap_or_else(|_| {
        let params: DbConnectParams =
            serde_json::from_str(&get_env_or_throw("DB_CONNECT_PARAMS_JSON"))
                .expect("Invalid JSON in DB_CONNECT_PARAMS_JSON");
        params.to_url()
    });
    let db_conn = Database::connect(&database_url).await?;
    let mut db_conn_global = DB_CONN
        .lock()
        .map_err(|e| DbErr::Custom(format!("Failed to lock database connection: {e}")))?;
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
    let db_conn = DB_CONN
        .lock()
        .map_err(|e| DbErr::Custom(format!("Failed to lock database connection: {e}")))?;
    if let Some(ref conn) = *db_conn {
        Ok(conn.clone())
    } else {
        Err(DbErr::Custom(
            "Database connection is not established".into(),
        ))
    }
}

/**
* Sets up the initial test environment (database connection and env variables)
*/
pub async fn setup_test_environment() {
    init();
    setup().await.expect("Failed to setup database connection.");
}

#[cfg(test)]
mod tests {

    use super::*;
    use serial_test::serial;

    /**
     * Test the establish_connection function
     *
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    #[serial]
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
    #[serial]
    async fn test_create_allocator() {
        setup_test_environment().await;

        let owner = "test_owner".to_string();
        let repo = "test_repo".to_string();

        let existing_allocator = database::allocators::get_allocator(&owner, &repo)
            .await
            .unwrap();
        if existing_allocator.is_some() {
            let result = database::allocators::delete_allocator(&owner, &repo).await;
            return assert!(result.is_ok());
        }

        let installation_id = Some(1234);
        let multisig_address = Some("0x1234567890".to_string());
        let verifiers_gh_handles = Some("test_verifier_1, test_verifier_2".to_string());
        let multisig_threshold = Some(2);
        let amount_type = Some("Fixed".to_string());
        let address = Some("0x1234567890".to_string());
        let tooling = Some("common_ui, smart_contract_allocator".to_string());
        let required_sps = Some("5+".to_string());
        let required_replicas = Some("5+".to_string());
        let registry_file_path = Some("Allocators/123.json".to_string());
        let client_contract_address = Some("f1owcbryeqlq3vl7kydzax7r75sbtyvgpnny7fswy".to_string());
        let ma_address = Some("f11234567890".to_string());
        let result = database::allocators::create_or_update_allocator(
            owner,
            repo,
            installation_id,
            multisig_address,
            verifiers_gh_handles,
            multisig_threshold,
            amount_type,
            address,
            tooling,
            required_sps,
            required_replicas,
            registry_file_path,
            client_contract_address,
            ma_address,
        )
        .await;
        assert!(result.is_ok());
    }

    /**
     * Test the get_allocators function
     *
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    #[serial]
    async fn test_get_allocators() {
        setup_test_environment().await;

        let result = database::allocators::get_allocators().await;
        assert!(result.is_ok());
    }

    /**
     * Test the get_allocator function
     *
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    #[serial]
    async fn test_get_allocator() {
        setup_test_environment().await;

        let allocator = database::allocators::get_allocators()
            .await
            .expect("Failed to get allocators")
            .pop()
            .expect("No allocators found");

        let result = database::allocators::get_allocator(&allocator.owner, &allocator.repo).await;
        assert!(result.is_ok());
    }

    /**
     * Test the delete_allocator function
     *
     * # Returns
     * @return () - The result of the test
     */
    #[tokio::test]
    #[serial]
    async fn test_delete_allocator() {
        setup_test_environment().await;

        let owner = "test_owner".to_string();
        let repo = "test_repo".to_string();

        let existing_allocator = database::allocators::get_allocator(&owner, &repo)
            .await
            .unwrap();
        if existing_allocator.is_some() {
            let result = database::allocators::delete_allocator(&owner, &repo).await;
            return assert!(result.is_ok());
        }

        let installation_id = Some(1234);
        let multisig_address = Some("0x1234567890".to_string());
        let verifiers_gh_handles = Some("test_verifier_1, test_verifier_2".to_string());
        let multisig_threshold = Some(2);
        let amount_type = Some("Fixed".to_string());
        let address = Some("0x1234567890".to_string());
        let tooling = Some("common_ui, smart_contract_allocator".to_string());
        let required_sps = Some("5+".to_string());
        let required_replicas = Some("5+".to_string());
        let registry_file_path = Some("Allocators/123.json".to_string());
        let client_contract_address = Some("f1owcbryeqlq3vl7kydzax7r75sbtyvgpnny7fswy".to_string());
        let ma_address = Some("f11234567890".to_string());

        let result = database::allocators::create_or_update_allocator(
            owner.clone(),
            repo.clone(),
            installation_id,
            multisig_address,
            verifiers_gh_handles,
            multisig_threshold,
            amount_type,
            address,
            tooling,
            required_sps,
            required_replicas,
            registry_file_path,
            client_contract_address,
            ma_address,
        )
        .await;

        assert!(result.is_ok());

        let result = database::allocators::delete_allocator(&owner, &repo).await;
        assert!(result.is_ok());
    }
}
