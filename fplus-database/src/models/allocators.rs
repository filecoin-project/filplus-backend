//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "allocators")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub owner: String,
    pub repo: String,
    pub installation_id: Option<i64>,
    pub multisig_address: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub verifiers_gh_handles: Option<String>,
    pub multisig_threshold: Option<i32>,
    pub allocation_amount_type: Option<String>,
    pub address: Option<String>,
    pub tooling: Option<String>,
    pub ma_address: Option<String>,
    pub required_sps: Option<String>,
    pub required_replicas: Option<String>,
    pub registry_file_path: Option<String>,
    pub client_contract_address: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AllocationAmounts,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AllocationAmounts => Entity::has_many(super::allocation_amounts::Entity).into(),
        }
    }
}

impl Related<super::allocation_amounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AllocationAmounts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
