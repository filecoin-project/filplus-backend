use octocrab::models::repos::ContentItems;

use crate::config::get_env_var_or_default;
use crate::external_services::github::github_async_new;
use crate::{base64::decode_allocator_model, error::LDNError};

use self::file::AllocatorModel;

pub mod file;

pub async fn process_allocator_file(file_name: &str) -> Result<AllocatorModel, LDNError> {

    let owner = get_env_var_or_default("ALLOCATOR_GOVERNANCE_OWNER");
    let repo = get_env_var_or_default("ALLOCATOR_GOVERNANCE_REPO");
    let branch = "main";
    let path = file_name.to_string();

    let gh = github_async_new(owner.to_string(), repo.to_string()).await;
    let content_items = gh.get_file(&path, branch).await.map_err(|e| LDNError::Load(e.to_string()))?;
    let model = content_items_to_allocator_model(content_items).map_err(|e| LDNError::Load(e.to_string()))?;

    Ok(model)
}


fn content_items_to_allocator_model(file: ContentItems) -> Result<AllocatorModel, LDNError> {
    let encoded_content = match file.items.get(0).and_then(|f| f.content.clone()) {
        Some(content) => {
            log::info!("Fetched content: {:?}", content);
            content
        },
        None => {
            log::error!("Allocator file is corrupted or empty");
            return Err(LDNError::Load("Allocator file is corrupted".to_string()));
        }
    };

    let cleaned_content = encoded_content.replace("\n", "");
    log::info!("Cleaned content: {:?}", cleaned_content);

    match decode_allocator_model(&cleaned_content) {
        Some(model) => {
            log::info!("Parsed allocator model successfully");
            Ok(model)
        },
        None => {
            log::error!("Failed to parse allocator model");
            Err(LDNError::Load("Failed to parse allocator model".to_string()))
        }
    }
}