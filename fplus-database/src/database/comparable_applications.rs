use crate::get_database_connection;
use crate::models::comparable_applications::{
    ActiveModel, ApplicationComparableData, Entity as ComparableApplication,
    Model as ComparableApplicationModel,
};
use sea_orm::prelude::Expr;
use sea_orm::{entity::*, Condition, DbErr, QueryFilter};

pub async fn create_comparable_application(
    client_address: &str,
    comparable_data: &ApplicationComparableData,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let new_comparable_data = ActiveModel {
        client_address: Set(client_address.to_string()),
        application: Set(comparable_data.clone()),
    };
    new_comparable_data.insert(&conn).await?;
    Ok(())
}

pub async fn get_comparable_applications() -> Result<Vec<ComparableApplicationModel>, DbErr> {
    let conn = get_database_connection().await?;
    let condition = Condition::any()
        .add(Expr::cust("char_length(application->>'project_desc') > 40"))
        .add(Expr::cust(
            "char_length(application->>'stored_data_desc') > 40",
        ));
    let response = ComparableApplication::find()
        .filter(condition)
        .all(&conn)
        .await?;
    Ok(response)
}
