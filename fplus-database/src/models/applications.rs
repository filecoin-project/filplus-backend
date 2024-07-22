//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "applications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub owner: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub repo: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub pr_number: i64,
    #[sea_orm(nullable)]
    pub issue_number: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub application: Option<String>,
    #[sea_orm(nullable)]
    pub updated_at: DateTime<Utc>,
    #[sea_orm(nullable)]
    pub sha: Option<String>,
    #[sea_orm(nullable)]
    pub path: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
