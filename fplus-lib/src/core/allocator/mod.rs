use octocrab::models::repos::ContentItems;

use crate::config::get_env_var_or_default;
use crate::external_services::filecoin::get_multisig_threshold_for_actor;
use crate::external_services::github::{github_async_new, GithubWrapper};
use crate::{
    base64::decode_allocator_model, error::LDNError,
};

use self::file::AllocatorModel;

pub mod file;

pub async fn process_allocator_file(file_name: &str) -> Result<AllocatorModel, LDNError> {
    let owner = get_env_var_or_default("ALLOCATOR_GOVERNANCE_OWNER");
    let repo = get_env_var_or_default("ALLOCATOR_GOVERNANCE_REPO");
    let installation_id = get_env_var_or_default("GITHUB_INSTALLATION_ID");
    let branch = "main";
    let path = file_name.to_string();

    let gh = GithubWrapper::new(owner.clone(), repo.clone(), installation_id);
    let content_items = gh.get_file(&path, branch).await.map_err(|e| LDNError::Load(e.to_string()))?;
    let mut model = content_items_to_allocator_model(content_items).map_err(|e| LDNError::Load(e.to_string()))?;

    // Get multisig threshold from the blockchain if multisig address is available
    if let Ok(blockchain_threshold) = get_multisig_threshold_for_actor(&model.multisig_address).await {
        model.multisig_threshold = Some(blockchain_threshold as i32);
    } else {
        log::warn!("Blockchain multisig threshold not found, using default or provided value");
        model.multisig_threshold = model.multisig_threshold.or(Some(2));
    }

    Ok(model)
}

fn content_items_to_allocator_model(file: ContentItems) -> Result<AllocatorModel, LDNError> {
    let encoded_content = match file.items.get(0).and_then(|f| f.content.clone()) {
        Some(content) => {
            log::info!("Fetched content: {:?}", content);
            content
        }
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
        }
        None => {
            log::error!("Failed to parse allocator model");
            Err(LDNError::Load(
                "Failed to parse allocator model".to_string(),
            ))
        }
    }
}

pub async fn is_allocator_repo_created(owner: &str, repo: &str) -> Result<bool, LDNError> {
    let repo_flag_file = "invalisd.md";
    let applications_directory = "applications";
    let gh = github_async_new(owner.to_string(), repo.to_string()).await;
    let all_files_result = gh.get_files(applications_directory).await.map_err(|e| {
        LDNError::Load(format!("Failed to retrieve all files from GitHub. Reason: {}", e))
    });

    match all_files_result {
        Ok(content_items) => {
            let mut is_repo_created = false;
            for file in content_items.items.iter() {
                if file.name == repo_flag_file {
                    is_repo_created = true;
                    break;
                }
            }
            Ok(is_repo_created)
        },
        Err(e) => {
            if e.to_string().contains("GitHub: This repository is empty") || e.to_string().contains("GitHub: Not Found"){
                Ok(false)
            } else {
                Err(e)
            }
        },
    }
}

pub async fn create_allocator_repo(owner: &str, repo: &str) -> Result<(), LDNError> {
    let gh = github_async_new(owner.to_string(), repo.to_string()).await;
    let mut dirs = Vec::new();
    dirs.push("".to_string());
    
    while dirs.len() > 0 {
        let dir = dirs.pop().unwrap();
        let files_list = gh.get_files_from_public_repo("clriesco", "filplus-allocator-template", Some(&dir)).await.map_err(|e| {
            LDNError::Load(format!("Failed to retrieve all files from GitHub. Reason: {}", e))
        })?;

        for file in files_list.items.iter() {
            let file_path = file.path.clone();
            if file.r#type == "dir" {
                dirs.push(file_path);
                continue;
            }
            let file = reqwest::Client::new()
            .get(&file.download_url.clone().unwrap())
            .send()
            .await
            .map_err(|e| LDNError::Load(format!("here {}", e)))?;
            let file = file
                .text()
                .await
                .map_err(|e| LDNError::Load(format!("here1 {}", e)))?;

            //Get file from target repo. If file does not exist or fails to retrieve, create it
            let target_file = gh.get_file(&file_path, "main").await.map_err(|e| {
                LDNError::Load(format!("Failed to retrieve file from GitHub. Reason: {} in file {}", e, file_path))
            });

            match target_file {
                Ok(target_file) => {
                    if target_file.items.is_empty() {
                        log::info!("Creating file in target repo: {}", file_path);
                        gh.add_file(&file_path, &file, "first commit", "main").await.map_err(|e| {
                            LDNError::Load(format!("Failed to create file in GitHub. Reason: {} in file {}", e, file_path))
                        })?;
                    } else {
                        log::info!("File already exists in target repo: {}", file_path);
                    }
                },
                Err(_) => {
                    log::info!("Creating file in target repo: {}", file_path);
                    gh.add_file(&file_path, &file, "first commit", "main").await.map_err(|e| {
                        LDNError::Load(format!("Failed to create file in GitHub. Reason: {} in file {}", e, file_path))
                    })?;
                },
            }
        }
    }

    Ok(())
}
