use actix_web::web;
use anyhow::Result;
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::core::common::get_collection;

const COLLECTION_NAME: &str = "govteam";

#[derive(Debug, Serialize, Deserialize)]
pub struct GovTeamMember {
    pub github_handle: String,
}

pub async fn find(state: web::Data<Mutex<Client>>) -> Result<Vec<GovTeamMember>> {
    let govteam_collection: Collection<GovTeamMember> = get_collection(state, COLLECTION_NAME).await?;
    let mut cursor = govteam_collection.find(None, None).await?;
    let mut ret = vec![];
    while let Ok(result) = cursor.advance().await {
        if result {
            let d = match cursor.deserialize_current() {
                Ok(d) => d,
                Err(_) => {
                    continue;
                }
            };
            ret.push(d);
        } else {
            break;
        }
    }
    Ok(ret)
}

pub async fn insert(state: web::Data<Mutex<Client>>, govteam: GovTeamMember) -> Result<()> {
    let govteam_collection: Collection<GovTeamMember> = get_collection(state, COLLECTION_NAME).await?;
    govteam_collection.insert_one(govteam, None).await?;
    Ok(())
}
