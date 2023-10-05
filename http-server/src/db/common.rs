use actix_web::web;
use anyhow::Result;
use mongodb::{Client, Collection};
use std::sync::Mutex;

pub const DATABASE: &str = "fplus-db";

pub async fn get_collection<T>(
    state: web::Data<Mutex<Client>>,
    collection_name: &str,
) -> Result<Collection<T>> {
    let col: Collection<T> = state
        .lock()
        .unwrap()
        .database(DATABASE)
        .collection(collection_name);
    Ok(col)
}
