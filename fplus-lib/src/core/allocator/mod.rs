use octocrab::models::repos::ContentItems;

use crate::{base64::decode_allocator_model, error::LDNError, external_services::github::GithubWrapper};

use self::file::AllocatorModel;

pub mod file;

pub async fn process_allocator_file(owner: &str, repo: &str, branch: &str, path: &str) -> Result<AllocatorModel, LDNError> {
    let gh = GithubWrapper::new(owner.to_string(), repo.to_string());

    let content_items = gh.get_file(path, branch).await.map_err(|e| LDNError::Load(e.to_string()))?;
    let model = content_items_to_allocator_model(content_items).map_err(|e| LDNError::Load(e.to_string()))?;

    Ok(model)
}


fn content_items_to_allocator_model(file: ContentItems) -> Result<AllocatorModel, LDNError> {
    let encoded_content = file.items
        .get(0)
        .and_then(|f| f.content.clone())
        .ok_or(LDNError::Load("Allocator file is corrupted".to_string()))?;

    let allocator_model = decode_allocator_model(&encoded_content.replace("\n", ""))
        .ok_or(LDNError::Load("Failed to parse allocator model".to_string()))?;

    Ok(allocator_model)
}

pub fn extract_owner_repo(file_name: &str) -> Result<(&str, &str), &'static str> {
    let parts: Vec<&str> = file_name.splitn(2, '_').collect();
    if parts.len() != 2 {
        return Err("Invalid file name format");
    }
    Ok((parts[0], parts[1]))
}