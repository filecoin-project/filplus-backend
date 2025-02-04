use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "comparable_applications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub client_address: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub application: ApplicationComparableData,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ApplicationComparableData {
    pub project_desc: String,
    pub stored_data_desc: String,
    pub data_owner_name: String,
    pub data_set_sample: String,
}
