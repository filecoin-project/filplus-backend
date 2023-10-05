use actix_web::web;
use mongodb::{Client, Collection};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use anyhow::Result;

use crate::db::common::get_collection;

const COLLECTION_NAME: &str = "rkh";

#[derive(Debug, Serialize, Deserialize)]
pub struct RootKeyHolder {
    pub github_handle: String,
}

pub async fn find(state: web::Data<Mutex<Client>>) -> Result<Vec<RootKeyHolder>> {
    let rkh_collection: Collection<RootKeyHolder> = get_collection(state, COLLECTION_NAME).await?;
    let mut cursor = rkh_collection.find(None, None).await?;
    let mut ret = vec![];
    while let Ok(result) = cursor.advance().await {
        if result {
            let d = match cursor.deserialize_current() {
                Ok(d) => d,
                Err(_) => { continue; }
            };
            ret.push(d);
        } else {
            break;
        }
    }
    Ok(ret)
}

pub async fn insert(state: web::Data<Mutex<Client>>, rkh: RootKeyHolder) -> Result<()> {
    let rkh_collection: Collection<RootKeyHolder> = get_collection(state, COLLECTION_NAME).await?;
    rkh_collection.insert_one(rkh, None).await?;
    Ok(())
}
