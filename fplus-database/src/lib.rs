pub mod models;
pub mod config;

use sea_orm::DatabaseConnection;
use crate::config::get_env_var_or_default;

pub fn init() {
    dotenv::dotenv().ok();
}

pub async fn setup() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let database_url = get_env_var_or_default("DB_URL", "");
    sea_orm::Database::connect(&database_url).await
}


#[cfg(test)]
mod tests {
    use crate::models::users;
    use crate::models::blockchain;

    use super::*;
    use sea_orm::entity::*;
    use tokio;

    #[tokio::test]
    async fn test_establish_connection_with_env_url() {
        init();
        let connection_result = setup().await;
        assert!(connection_result.is_ok());
    }

    #[tokio::test]
    async fn test_user_create_and_fetch() {
        init();
        let db = setup().await.expect("Failed to setup database");

        let user = users::ActiveModel {
            onchain_address: ActiveValue::set("0x124".to_owned()),
            github_handle: ActiveValue::set(Some("github_user".to_owned())),
            user_type: ActiveValue::set(Some(0)),
            created: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let insert_result = user.insert(&db).await;
        assert!(insert_result.is_ok(), "Failed to insert user");

        let select_result = users::Entity::find_by_id("0x124").one(&db).await;
        assert!(select_result.is_ok(), "Failed to select user");
        let selected_user = select_result.unwrap().unwrap();
        assert_eq!(selected_user.onchain_address, "0x124");
        assert_eq!(selected_user.github_handle, Some("github_user".to_owned()));

        let delete_result = selected_user.delete(&db).await;
        assert!(delete_result.is_ok(), "Failed to delete user");
        
    }

    #[tokio::test]
    async fn test_user_update() {
        init();
        let db = setup().await.expect("Failed to setup database");

        let user = users::ActiveModel {
            onchain_address: ActiveValue::set("0x123".to_owned()),
            github_handle: ActiveValue::set(Some("github_user".to_owned())),
            user_type: ActiveValue::set(Some(0)),
            created: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let insert_result = user.insert(&db).await;
        assert!(insert_result.is_ok(), "Failed to insert user (for update)");

        let select_result = users::Entity::find_by_id("0x123").one(&db).await;
        assert!(select_result.is_ok(), "Failed to select user (for update)");

        let mut selected_user: users::ActiveModel = select_result.unwrap().unwrap().into();
        selected_user.github_handle = ActiveValue::set(Some("new_github_user".to_owned()));
        let update_result = selected_user.save(&db).await;
        assert!(update_result.is_ok(), "Failed to update user (for update)");

        let select_result = users::Entity::find().one(&db).await;
        assert!(select_result.is_ok(), "Failed to select user (for update)");
        let selected_user = select_result.unwrap().unwrap();
        assert_eq!(selected_user.github_handle, Some("new_github_user".to_owned()));

        let delete_result = selected_user.delete(&db).await;
        assert!(delete_result.is_ok(), "Failed to delete user (for update)");

    }

    #[tokio::test]
    async fn test_relation_user_blockchain() {
        init();
        let db = setup().await.expect("Failed to setup database");

        let user = users::ActiveModel {
            onchain_address: ActiveValue::set("0x123".to_owned()),
            github_handle: ActiveValue::set(Some("github_user".to_owned())),
            user_type: ActiveValue::set(Some(0)),
            created: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let user_onchain_address = user.onchain_address.clone(); 
        let insert_result = user.insert(&db).await;
        assert!(insert_result.is_ok(), "Failed to insert user (for relation)");

        let blockchain_item = blockchain::ActiveModel {
            application_onchain_address: ActiveValue::set(Some("0x123".to_owned())),
            datacap_amount: ActiveValue::set(Some("100".to_owned())),
            user_onchain_address: ActiveValue::set(Some(user_onchain_address.unwrap())),
            r#type: ActiveValue::set(Some(0)),
            successful: ActiveValue::set(Some(true)),
            refill: ActiveValue::set(Some(false)),
            created: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let insert_result = blockchain_item.insert(&db).await;
        assert!(insert_result.is_ok(), "Failed to insert blockchain (for relation)");
        
        let blockchain_item = blockchain::Entity::find_by_id(insert_result.unwrap().id).one(&db).await.unwrap().unwrap();
        // Get users related to this blockchain item
        let select_result = blockchain_item.try_into_model().unwrap().find_related(users::Entity).all(&db).await;
        assert!(select_result.is_ok(), "Failed to select related users (for relation)");
        let related_users = select_result.unwrap();
        assert_eq!(related_users.len(), 1);

    }



}