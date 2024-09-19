use alloy::primitives::{Address, AddressError};
use chrono::{DateTime, FixedOffset};
use sea_orm::entity::prelude::*;
use sea_orm_newtype::DeriveNewType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "autoallocations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub evm_wallet_address: AddressWrapper,
    pub last_allocation: DateTime<FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, DeriveNewType, Eq, Serialize, Deserialize)]
#[sea_orm_newtype(try_from_into = "String", primary_key)]
pub struct AddressWrapper(pub Address);

impl TryFrom<String> for AddressWrapper {
    type Error = AddressError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(AddressWrapper(Address::parse_checksummed(value, None)?))
    }
}

impl From<AddressWrapper> for String {
    fn from(value: AddressWrapper) -> Self {
        value.0.to_checksum(None)
    }
}

impl From<Address> for AddressWrapper {
    fn from(value: Address) -> Self {
        AddressWrapper(value)
    }
}
