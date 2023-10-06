use actix_web::web;
use mongodb::{Client, Collection};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use anyhow::Result;

use crate::core::common::get_collection;

const COLLECTION_NAME: &str = "logs";

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub timestamp: String,
    pub message: String,
}

pub async fn find(state: web::Data<Mutex<Client>>) -> Result<Vec<Log>> {
    let logs_collection: Collection<Log> = get_collection(state, COLLECTION_NAME).await?;
    let mut cursor = logs_collection.find(None, None).await?;
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

pub async fn insert(state: web::Data<Mutex<Client>>, log: Log) -> Result<()> {
    let log_collection: Collection<Log> = get_collection(state, COLLECTION_NAME).await?;
    log_collection.insert_one(log, None).await?;
    Ok(())
}
