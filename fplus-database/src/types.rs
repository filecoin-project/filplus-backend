use alloy::primitives::{Address, AddressError};
use sea_orm_newtype::DeriveNewType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(super) struct DbConnectParams {
    password: String,
    dbname: String,
    engine: String,
    port: u16,
    host: String,
    username: String,
}

impl DbConnectParams {
    pub fn to_url(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}?{}",
            self.engine,
            self.username,
            urlencoding::encode(&self.password),
            self.host,
            self.port,
            self.dbname,
            std::env::var("DB_OPTIONS").unwrap_or_default(),
        )
    }
}

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