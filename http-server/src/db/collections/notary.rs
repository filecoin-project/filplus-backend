use actix_web::web;
use mongodb::{Client, Collection};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use anyhow::Result;

use crate::db::common::get_collection;

const COLLECTION_NAME: &str = "notary";

#[derive(Debug, Serialize, Deserialize)]
pub struct Notary {
    pub github_handle: String,
    pub on_chain_address: String,
}


pub async fn find(state: web::Data<Mutex<Client>>) -> Result<Vec<Notary>> {
    let notary_collection: Collection<Notary> = get_collection(state, COLLECTION_NAME).await?;
    let mut cursor = notary_collection.find(None, None).await?;
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