use fplus_database::database::allocators::{create_or_update_allocator, get_allocators,};
use fplus_database::models::allocators::Model;

use octocrab::auth::create_jwt;
use octocrab::models::repos::{ContentItems, Content};

use crate::config::get_env_var_or_default;
use crate::external_services::filecoin::get_multisig_threshold_for_actor;
use crate::external_services::github::{github_async_new, GithubWrapper};
use crate::{
    base64::decode_allocator_model, error::LDNError,
};

use self::file::{AccessTokenResponse, AllocatorModel, Installation, InstallationRepositories, RepositoriesResponse, RepositoryInfo};

use jsonwebtoken::EncodingKey;
use reqwest::{Client, header};
use anyhow::Result;

use super::GithubQueryParams;

pub mod file;

pub async fn process_allocator_file(file_name: &str) -> Result<AllocatorModel, LDNError> {
    let owner = get_env_var_or_default("ALLOCATOR_GOVERNANCE_OWNER");
    let repo = get_env_var_or_default("ALLOCATOR_GOVERNANCE_REPO");
    let installation_id = get_env_var_or_default("GITHUB_INSTALLATION_ID");
    let branch = "main";
    let path = file_name.to_string();

    let gh = GithubWrapper::new(owner.clone(), repo.clone(), installation_id.clone());
    let content_items: ContentItems = gh.get_files_from_public_repo(&owner, &repo, branch, Some(&path)).await.map_err(|e| LDNError::Load(e.to_string()))?;
    let mut model: AllocatorModel = content_items_to_allocator_model(content_items).map_err(|e| LDNError::Load(e.to_string()))?;

    // Get multisig threshold from the blockchain if multisig address is available
    if let Ok(blockchain_threshold) = get_multisig_threshold_for_actor(&model.pathway_addresses.msig).await {
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
        Some(mut model) => {
            let mut owner_repo_parts: Vec<&str> = model.application.allocation_bookkeeping.split('/').collect();
            // If last part is empty, remove it
            if owner_repo_parts[owner_repo_parts.len() - 1].is_empty() {
                owner_repo_parts.pop();
            }
            if owner_repo_parts.len() < 2 {
                log::error!("Failed to parse allocator model");
                return Err(LDNError::Load("Failed to parse allocator model".to_string()));
            }

            //If repo ends with .git, remove it
            let mut repo = owner_repo_parts[owner_repo_parts.len() - 1].to_string();
            if repo.ends_with(".git") {
                repo = repo[..repo.len() - 4].to_string();
            }
            
            model.owner = Some(owner_repo_parts[owner_repo_parts.len() - 2].to_string());
            model.repo = Some(repo);
            
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
    let repo_flag_file = "invalid.md";
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

pub async fn create_file_in_repo(gh: &GithubWrapper, file: &Content, force: bool) -> Result<(), LDNError> {
    let file_path = file.path.clone();
    let file_sha = file.sha.clone();
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
                    LDNError::Load(format!("Failed to create file in GitHub repo {}/{}. Reason: {} in file {}", gh.owner.clone(), gh.repo.clone(), e, file_path))
                })?;
            } else if !force {
                log::info!("File already exists in target repo {}/{}: {}", gh.owner.clone(), gh.repo.clone(), file_path);
            } else if target_file.items[0].sha.clone() != file_sha {
                log::info!("Force creating file in target repo {}/{}: {}", gh.owner.clone(), gh.repo.clone(), file_path);
                let file_sha = target_file.items[0].sha.clone();
                gh.update_file(&file_path,"Update", &file, "main", &file_sha).await.map_err(|e| {
                    LDNError::Load(format!("Failed to update file in GitHub repo {}/{}. Reason: {} in file {}", gh.owner.clone(), gh.repo.clone(), e, file_path))
                })?;
            }
        },
        Err(_) => {
            log::info!("Creating file in target repo: {}", file_path);
            gh.add_file(&file_path, &file, "first commit", "main").await.map_err(|e| {
                LDNError::Load(format!("Failed to create file in GitHub repo {}/{}. Reason: {} in file {}", gh.owner.clone(), gh.repo.clone(), e, file_path))
            })?;
        },
    }

    Ok(())
}

pub async fn create_allocator_repo(owner: &str, repo: &str) -> Result<(), LDNError> {
    let gh = github_async_new(owner.to_string(), repo.to_string()).await;
    let mut dirs = Vec::new();
    let branch = match get_env_var_or_default("FILPLUS_ENV").as_str() {
        "staging" => "staging",
        "production" => "main",
        _ => "main",
    };
    let allocator_template_owner = get_env_var_or_default("ALLOCATOR_TEMPLATE_OWNER");
    let allocator_template_repo = get_env_var_or_default("ALLOCATOR_TEMPLATE_REPO");

    dirs.push("".to_string());
    
    while dirs.len() > 0 {
        let dir = dirs.pop().unwrap();
        let files_list = gh.get_files_from_public_repo(&allocator_template_owner, &allocator_template_repo, branch, Some(&dir)).await.map_err(|e| {
            LDNError::Load(format!("Failed to retrieve all files from GitHub. Reason: {}", e))
        })?;

        for file in files_list.items.iter() {
            let file_path = file.path.clone();
            if file.r#type == "dir" {
                dirs.push(file_path);
                continue;
            }
            self::create_file_in_repo(&gh, &file, false).await?;
        }
    }

    Ok(())
}

pub async fn generate_github_app_jwt() -> Result<String, jsonwebtoken::errors::Error> {
    let app_id = get_env_var_or_default("GITHUB_APP_ID").parse().unwrap();
    let pem = get_env_var_or_default("GH_PRIVATE_KEY");

    return match EncodingKey::from_rsa_pem(pem.to_string().as_bytes()) {
        Ok(key) => {
            let token = create_jwt(octocrab::models::AppId(app_id), &key).unwrap();
            Ok(token)
        },
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }

}

pub async fn fetch_installation_ids(client: &Client, jwt: &str) -> Result<Vec<u64>> {
    let req_url = "https://api.github.com/app/installations";
    let response = client.get(req_url)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .header(header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header(header::USER_AGENT, "YourApp")
        .send()
        .await?;

    if !response.status().is_success() {
        log::error!("Request failed with status: {}", response.status());
    }

    let text = response.text().await?;

    log::debug!("Response body: {}", text);

    let installations: Vec<Installation> = match serde_json::from_str(&text) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse response as JSON: {}", e);
            return Err(e.into());
        }
    };
    Ok(installations.into_iter().map(|i| i.id).collect())
}

pub async fn fetch_access_token(client: &Client, jwt: &str, installation_id: u64) -> Result<String> {
    let req_url = format!("https://api.github.com/app/installations/{}/access_tokens", installation_id);
    let res: AccessTokenResponse = client.post(req_url)
        .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
        .header(header::USER_AGENT, "YourApp")
        .send()
        .await?
        .json()
        .await?;
    Ok(res.token)
}

pub async fn fetch_repositories(client: &Client, token: &str) -> Result<Vec<RepositoryInfo>> {
    let req_url = "https://api.github.com/installation/repositories";
    let res: RepositoriesResponse = client.get(req_url)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::USER_AGENT, "YourApp")
        .send()
        .await?
        .json()
        .await?;
    Ok(res.repositories.into_iter().map(|r| RepositoryInfo { slug: r.name, owner: r.owner.login }).collect())
}

pub async fn fetch_repositories_for_installation_id(client: &Client, jwt: &str, id: u64) -> Result<Vec<RepositoryInfo>> {
    let token = fetch_access_token(&client, &jwt, id).await.unwrap();
    let repositories = fetch_repositories(&client, &token).await.unwrap();
    Ok(repositories)
}

pub async fn update_installation_ids_in_db(installation: InstallationRepositories) {
    let installation_id = installation.installation_id;
    for repo in installation.repositories.iter() {
        let owner = repo.owner.clone();
        let repo = repo.slug.clone();
        let _ = create_or_update_allocator(owner, repo, Some(installation_id.try_into().unwrap()), None, None, None, None, None, None).await;
    }
}

pub async fn update_installation_ids_logic() {
    let client = Client::new();
    let jwt = match generate_github_app_jwt().await {
        Ok(jwt) => jwt,
        Err(e) => {
            log::error!("Failed to generate GitHub App JWT: {}", e);
            return;
        }
    };

    let installation_ids_result = fetch_installation_ids(&client, &jwt).await;
    let mut results: Vec<InstallationRepositories> = Vec::new();

    for id in installation_ids_result.unwrap_or_default() {
        let repositories: Vec<RepositoryInfo> = fetch_repositories_for_installation_id(&client, &jwt, id).await.unwrap();
        results.push(InstallationRepositories { installation_id: id, repositories });
    }

    for installation in results.iter() {
        update_installation_ids_in_db(installation.clone()).await;
    }
}

pub async fn update_single_installation_id_logic(installation_id: String) -> Result<InstallationRepositories, LDNError> {
    let client = Client::new();
    let jwt = match generate_github_app_jwt().await {
        Ok(jwt) => jwt,
        Err(e) => {
            log::error!("Failed to generate GitHub App JWT: {}", e);
            return Err(LDNError::Load(e.to_string()));
        }
    };
    
    let repositories: Vec<RepositoryInfo> = fetch_repositories_for_installation_id(&client, &jwt, installation_id.parse().unwrap()).await.unwrap();
    let installation = InstallationRepositories { installation_id: installation_id.parse().unwrap(), repositories };
    update_installation_ids_in_db(installation.clone()).await;
    return Ok(installation);
}

pub async fn force_update_allocators(files: Vec<String>, affected_allocators: Option<Vec<GithubQueryParams>>) -> Result<(), LDNError> {
    // first get all allocators from db and filter by affected_allocators
    let allocators = get_allocators()
        .await
        .map_err(|e| LDNError::Load(e.to_string()))?;

    //filter allocators that have installation_id and msig_address
    let allocators: Vec<Model> = allocators
        .iter()
        .filter(|a| a.installation_id.is_some() && a.multisig_address.is_some())
        .cloned() // Clone the elements before collecting them
        .collect();

    // if affected_allocators is provided, filter allocators by owner and repo
    let allocators: Vec<Model> = match affected_allocators {
        Some(affected_allocators) => {
            allocators
                .iter()
                .filter(|a| {
                    affected_allocators
                        .iter()
                        .any(|aa| aa.owner == a.owner && aa.repo == a.repo)
                })
                .cloned()
                .collect()
        }
        None => allocators,
    };

    //If no allocators return
    if allocators.is_empty() {
        log::info!("No allocators to update");
        return Ok(());
    }
    
    let branch = match get_env_var_or_default("FILPLUS_ENV").as_str() {
        "staging" => "staging",
        "production" => "main",
        _ => "main"
    };
    let allocator_template_owner = get_env_var_or_default("ALLOCATOR_TEMPLATE_OWNER");
    let allocator_template_repo = get_env_var_or_default("ALLOCATOR_TEMPLATE_REPO");

    //now iterate over allocators and files
    for allocator in allocators {
        let gh = GithubWrapper::new(
            allocator.owner, 
            allocator.repo, 
            allocator.installation_id.unwrap().to_string()
        );

        for file in files.iter() {

            match gh.get_files_from_public_repo(&allocator_template_owner, &allocator_template_repo, branch, Some(&file)).await {
                Ok(content) => {
                    match create_file_in_repo(&gh, &content.items[0], true).await {
                        Ok(_) => {
                            log::info!("File {} updated successfully", file);
                        },
                        Err(e) => {
                            log::error!("{}", e);
                        }
                    }
                },
                Err(e) => {
                    log::error!("{}", e);
                }
            }
        }
    }    
    
    Ok(())
}


pub fn validate_amount_type_and_options(amount_type: &str, amount_options: &[String]) -> Result<(), String> {
    match amount_type {
        "fixed" => validate_fixed_amount_options(amount_options),
        "percentage" => validate_percentage_amount_options(amount_options),
        _ => Err("Invalid amount type".into()),
    }
}

pub fn validate_fixed_amount_options(amount_options: &[String]) -> Result<(), String> {
    for option in amount_options {
        if !is_valid_fixed_option(option) {
            return Err(format!("Invalid fixed amount option: {}", option));
        }
    }
    Ok(())
}

pub fn validate_percentage_amount_options(amount_options: &[String]) -> Result<(), String> {
    for option in amount_options {
        let no_percentage_option = option.replace("%", "");
        if !no_percentage_option.parse::<i32>().is_ok() {
            return Err(format!("Invalid percentage amount option: {}", option));
        }
    }
    Ok(())
}

pub fn is_valid_fixed_option(option: &str) -> bool {
    let allowed_units = ["GiB", "TiB", "PiB", "GB", "TB", "PB"];
    let number_part = option.trim_end_matches(|c: char| !c.is_digit(10));
    let unit_part = option.trim_start_matches(|c: char| c.is_digit(10));

    number_part.parse::<i32>().is_ok() && allowed_units.contains(&unit_part)
}
