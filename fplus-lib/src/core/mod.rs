use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};

use alloy::primitives::Address;

use application::file::{SpsChangeRequest, StorageProviderChangeVerifier};
use chrono::{DateTime, Local, Utc};
use futures::future;
use octocrab::models::{
    pulls::PullRequest,
    repos::{Content, ContentItems},
};
use rayon::prelude::*;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::external_services::filecoin::get_client_allocation;
use crate::{
    base64,
    config::get_env_var_or_default,
    core::application::{
        file::Allocations,
        gitcoin_interaction::{
            get_address_from_signature, verify_on_gitcoin, ExpirableSolStruct, KycApproval,
            KycAutoallocationApproval,
        },
    },
    error::LDNError,
    external_services::{
        filecoin::{get_allowance_for_address, get_multisig_threshold_for_actor},
        github::{
            github_async_new, CreateMergeRequestData, CreateRefillMergeRequestData, GithubWrapper,
        },
    },
    helpers::{compare_allowance_and_allocation, parse_size_to_bytes, process_amount},
    parsers::ParsedIssue,
};
use fplus_database::database::allocation_amounts::get_allocation_quantity_options;
use fplus_database::database::{
    self,
    allocators::{get_allocator, update_allocator_threshold},
};

use fplus_database::models::applications::Model as ApplicationModel;

use self::application::file::{
    AllocationRequest, AllocationRequestType, AppState, ApplicationFile, DeepCompare,
    ValidVerifierList, VerifierInput,
};

use crate::core::application::file::Allocation;
use std::collections::HashSet;

pub mod allocator;
pub mod application;
pub mod autoallocator;

#[derive(Deserialize)]
pub struct CreateApplicationInfo {
    pub issue_number: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct TriggerSSAInfo {
    pub amount: String,
    pub amount_type: String,
}

#[derive(Deserialize)]
pub struct BranchDeleteInfo {
    pub owner: String,
    pub repo: String,
    pub branch_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VerifierList(pub Vec<String>);

#[derive(Deserialize, Serialize, Debug)]
pub struct ApplicationProposalApprovalSignerInfo {
    pub signing_address: String,
    pub created_at: String,
    pub message_cids: GrantDataCapCids,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrantDataCapCids {
    pub message_cid: String,
    pub increase_allowance_cid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompleteNewApplicationProposalInfo {
    pub signer: ApplicationProposalApprovalSignerInfo,
    pub request_id: String,
    pub new_allocation_amount: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StorageProvidersChangeSignerInfo {
    pub signing_address: String,
    pub max_deviation_cid: Option<String>,
    pub allowed_sps_cids: Option<HashMap<String, Vec<String>>>,
    pub removed_allowed_sps_cids: Option<HashMap<String, Vec<String>>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StorageProvidersChangeProposalInfo {
    pub signer: StorageProvidersChangeSignerInfo,
    pub allowed_sps: Option<Vec<u64>>,
    pub max_deviation: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StorageProvidersChangeApprovalInfo {
    pub signer: StorageProvidersChangeSignerInfo,
    pub request_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompleteNewApplicationApprovalInfo {
    pub signer: ApplicationProposalApprovalSignerInfo,
    pub request_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MoreInfoNeeded {
    pub verifier_message: String,
}

#[derive(Debug)]
pub struct LDNApplication {
    github: GithubWrapper,
    pub application_id: String,
    pub file_sha: String,
    pub file_name: String,
    pub branch_name: String,
}

#[derive(Deserialize, Debug)]
pub struct RefillInfo {
    pub id: String,
    pub amount: String,
    pub amount_type: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct NotifyRefillInfo {
    pub owner: String,
    pub repo: String,
    pub issue_number: String,
}

#[derive(Deserialize)]
pub struct DcReachedInfo {
    pub id: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct ValidationPullRequestData {
    pub pr_number: String,
    pub user_handle: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct ValidationIssueData {
    pub issue_number: String,
    pub user_handle: String,
}

#[derive(Deserialize)]
pub struct Allocator {
    pub owner: String,
    pub repo: String,
    pub installation_id: Option<i64>,
    pub multisig_address: Option<String>,
    pub verifiers_gh_handles: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ChangedAllocators {
    pub files_changed: Vec<String>,
}

#[derive(Deserialize)]
pub struct AllocatorUpdateForceInfo {
    pub files: Vec<String>,
    pub allocators: Option<Vec<GithubQueryParams>>,
}

#[derive(Deserialize, Debug)]
pub struct LastAutoallocationQueryParams {
    pub evm_wallet_address: Address,
}

#[derive(Deserialize)]
pub struct TriggerAutoallocationInfo {
    pub message: KycAutoallocationApproval,
    pub signature: String,
}
#[derive(Deserialize)]
pub struct GithubQueryParams {
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct ApplicationQueryParams {
    pub id: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct CompleteGovernanceReviewInfo {
    pub allocation_amount: String,
    pub client_contract_address: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifierActionsQueryParams {
    pub github_username: String,
    pub id: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize)]
pub struct SubmitKYCInfo {
    pub message: KycApproval,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct ApplicationFileWithDate {
    pub application_file: ApplicationFile,
    pub updated_at: DateTime<Utc>,
    pub pr_number: u64,
    pub sha: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationGithubInfo {
    pub sha: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationWithAllocation {
    application_file: ApplicationFile, // Assuming ApplicationFile is the type for app_file
    allocation: AllocationObject,
}

#[derive(Debug, Serialize)]
pub struct AllocationObject {
    allocation_amount_type: String,
    allocation_amount_quantity_options: Vec<String>,
}

impl LDNApplication {
    pub async fn single_active(
        pr_number: u64,
        owner: String,
        repo: String,
    ) -> Result<ApplicationFile, LDNError> {
        let gh = github_async_new(owner, repo).await;
        let (_, pull_request) = gh.get_pull_request_files(pr_number).await.unwrap();
        let pull_request = pull_request.first().unwrap();
        let pull_request: Response = reqwest::Client::new()
            .get(pull_request.raw_url.to_string())
            .send()
            .await
            .map_err(|e| LDNError::Load(format!("Failed to get pull request files /// {}", e)))?;
        let pull_request = pull_request
            .text()
            .await
            .map_err(|e| LDNError::Load(format!("Failed to get pull request files /// {}", e)))?;
        if let Ok(app) = serde_json::from_str::<ApplicationFile>(&pull_request) {
            Ok(app)
        } else {
            Err(LDNError::Load(format!(
                "Pull Request {} Application file is corrupted or invalid format: {}",
                pr_number,
                serde_json::from_str::<ApplicationFile>(&pull_request).unwrap_err()
            )))
        }
    }

    async fn get_pr_files_and_app(
        owner: String,
        repo: String,
        pr_number: u64,
    ) -> Result<
        Option<(
            (u64, Vec<octocrab::models::pulls::FileDiff>),
            ApplicationFile,
        )>,
        LDNError,
    > {
        let gh = github_async_new(owner, repo).await;
        let files = match gh.get_pull_request_files(pr_number).await {
            Ok(files) => files,
            Err(_) => return Ok(None),
        };
        let raw_url = match files.1.first() {
            Some(f) => f.raw_url.clone(),
            None => return Ok(None),
        };
        let response = reqwest::Client::new().get(raw_url).send().await;
        let response = match response {
            Ok(response) => response,
            Err(_) => return Ok(None),
        };
        let response = response.text().await;
        let response = match response {
            Ok(response) => response,
            Err(_) => return Ok(None),
        };
        let app = match ApplicationFile::from_str(&response) {
            Ok(app) => app,
            Err(e) => {
                dbg!(&e);
                return Ok(None);
            }
        };

        Ok(Some((files, app)))
    }

    async fn load_pr_files(
        pr: PullRequest,
        owner: String,
        repo: String,
    ) -> Result<Option<(String, String, ApplicationFile, PullRequest)>, LDNError> {
        let result = Self::get_pr_files_and_app(owner.clone(), repo.clone(), pr.number).await;
        if let Some((files, app)) = result? {
            Ok(Some((
                files.1.first().unwrap().sha.clone(),
                files.1.first().unwrap().filename.clone(),
                app,
                pr.clone(),
            )))
        } else {
            Ok(None)
        }
    }

    async fn get_application_model(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<ApplicationModel, LDNError> {
        let app_model_result =
            database::applications::get_application(application_id, owner, repo, None).await;
        match app_model_result {
            Ok(model) => Ok(model),
            Err(e) => Err(LDNError::Load(format!("Database error: {}", e))),
        }
    }

    pub async fn load_from_db(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<ApplicationFile, LDNError> {
        let app_model =
            Self::get_application_model(application_id.clone(), owner.clone(), repo.clone())
                .await?;

        let app_str = app_model.application.ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an application field",
                application_id
            ))
        })?;

        ApplicationFile::from_str(&app_str)
            .map_err(|e| LDNError::Load(format!("Failed to parse application file from DB: {}", e)))
    }

    pub async fn application_with_allocation_amount(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<ApplicationWithAllocation, LDNError> {
        let app_model_result = database::applications::get_application(
            application_id.clone(),
            owner.clone(),
            repo.clone(),
            None,
        )
        .await;

        let app_model = match app_model_result {
            Ok(model) => model,
            Err(e) => return Err(LDNError::Load(format!("Database error: {}", e))),
        };

        // Check if the application field is present and parse it
        let app_str = app_model.application.ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an application field",
                application_id
            ))
        })?;

        let app_file = ApplicationFile::from_str(&app_str).map_err(|e| {
            LDNError::Load(format!("Failed to parse application file from DB: {}", e))
        })?;

        let db_allocator = match get_allocator(&owner, &repo).await {
            Ok(allocator) => allocator.unwrap(),
            Err(err) => {
                return Err(LDNError::New(format!("Database: get_allocator: {}", err)));
            }
        };

        let allocation_amount_type = db_allocator
            .allocation_amount_type
            .unwrap_or("".to_string());

        let allocation_amount_quantity_options = get_allocation_quantity_options(db_allocator.id)
            .await
            .unwrap();

        Ok(ApplicationWithAllocation {
            allocation: {
                AllocationObject {
                    allocation_amount_type,
                    allocation_amount_quantity_options,
                }
            },
            application_file: app_file,
        })
    }

    pub async fn load(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<Self, LDNError> {
        let gh = github_async_new(owner.to_string(), repo.to_string()).await;
        let pull_requests = gh.list_pull_requests().await.unwrap();
        let pull_requests = future::try_join_all(
            pull_requests
                .into_iter()
                .map(|pr: PullRequest| {
                    LDNApplication::load_pr_files(pr, owner.clone(), repo.clone())
                })
                .collect::<Vec<_>>(),
        )
        .await?;
        let result = pull_requests
            .par_iter()
            .filter(|pr| {
                if let Some(r) = pr {
                    r.2.id.clone() == application_id.clone()
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();
        if let Some(Some(r)) = result.first() {
            return Ok(Self {
                github: gh,
                application_id: r.2.id.clone(),
                file_sha: r.0.clone(),
                file_name: r.1.clone(),
                branch_name: r.3.head.ref_field.clone(),
            });
        }

        let app = Self::single_merged(application_id, owner.clone(), repo.clone()).await?;
        Ok(Self {
            github: gh,
            application_id: app.1.id.clone(),
            file_sha: app.0.sha.clone(),
            file_name: app.0.path.clone(),
            branch_name: "main".to_string(),
        })
    }

    pub async fn all_applications() -> Result<Vec<(ApplicationFile, String, String)>, Vec<LDNError>>
    {
        let db_apps = database::applications::get_applications().await;
        let mut all_apps: Vec<(ApplicationFile, String, String)> = Vec::new();
        match db_apps {
            Ok(apps) => {
                for app in apps {
                    let app_file = match ApplicationFile::from_str(&app.application.unwrap()) {
                        Ok(app) => app,
                        Err(_) => {
                            continue;
                        }
                    };
                    all_apps.push((app_file, app.owner, app.repo));
                }
                Ok(all_apps)
            }
            Err(e) => Err(vec![LDNError::Load(format!(
                "Failed to retrieve applications from the database /// {}",
                e
            ))]),
        }
    }

    pub async fn active(
        owner: String,
        repo: String,
        filter: Option<String>,
    ) -> Result<Vec<ApplicationFile>, LDNError> {
        // Get all active applications from the database.
        let active_apps_result =
            database::applications::get_active_applications(Some(owner), Some(repo)).await;

        // Handle errors in getting active applications.
        let active_apps = match active_apps_result {
            Ok(apps) => apps,
            Err(e) => return Err(LDNError::Load(format!("Database error: {}", e))),
        };

        // Filter and convert active applications.
        let mut apps: Vec<ApplicationFile> = Vec::new();
        for app_model in active_apps {
            // If a filter was provided and it doesn't match the application's id, continue to the next iteration.
            if let Some(ref filter_id) = filter {
                if app_model.application.is_some() && app_model.id != filter_id.as_str() {
                    continue;
                }
            }

            // Try to deserialize the `application` field to `ApplicationFile`.
            if let Some(app_json) = app_model.application {
                match from_str::<ApplicationFile>(&app_json) {
                    Ok(app) => apps.push(app),
                    //if error, don't push into apps
                    Err(err) => {
                        log::error!("Failed to parse application file from DB: {}", err);
                    }
                }
            }
        }

        Ok(apps)
    }

    pub async fn active_apps_with_last_update(
        owner: String,
        repo: String,
        filter: Option<String>,
    ) -> Result<Vec<ApplicationFileWithDate>, LDNError> {
        let gh = github_async_new(owner.to_string(), repo.to_string()).await;
        let mut apps: Vec<ApplicationFileWithDate> = Vec::new();
        let pull_requests = gh.list_pull_requests().await.unwrap();
        let pull_requests = future::try_join_all(
            pull_requests
                .into_iter()
                .map(|pr: PullRequest| {
                    LDNApplication::load_pr_files(pr, owner.clone(), repo.clone())
                })
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
        for (sha, path, app_file, pr_info) in pull_requests.into_iter().flatten() {
            if let Some(updated_at) = pr_info.updated_at {
                let app_with_date = ApplicationFileWithDate {
                    application_file: app_file.clone(),
                    updated_at,
                    pr_number: pr_info.number,
                    sha,
                    path,
                };

                if filter.as_ref().map_or(true, |f| &app_file.id == f) {
                    apps.push(app_with_date);
                }
            }
        }
        Ok(apps)
    }

    pub async fn merged_apps_with_last_update(
        owner: String,
        repo: String,
        filter: Option<String>,
    ) -> Result<Vec<ApplicationFileWithDate>, LDNError> {
        let gh = Arc::new(github_async_new(owner.to_string(), repo.to_string()).await);

        let applications_path = "applications";
        let mut all_files_result = gh.get_files(applications_path).await.map_err(|e| {
            LDNError::Load(format!(
                "Failed to retrieve all files from GitHub. Reason: {}",
                e
            ))
        })?;

        all_files_result
            .items
            .retain(|item| item.download_url.is_some() && item.name.ends_with(".json"));

        let mut application_files_with_date: Vec<ApplicationFileWithDate> = vec![];
        for fd in all_files_result.items {
            let gh_clone = Arc::clone(&gh);
            let result = gh_clone.get_last_modification_date(&fd.path).await;

            if let Ok(updated_at) = result {
                let map_result = LDNApplication::map_merged(fd).await;

                if let Ok(Some((content, app_file))) = map_result {
                    application_files_with_date.push(ApplicationFileWithDate {
                        application_file: app_file,
                        updated_at,
                        pr_number: 0,
                        sha: content.sha,
                        path: content.path,
                    });
                }
            } else {
                log::warn!("Failed to get last modification date for file: {}", fd.path);
            }
        }

        let filtered_files: Vec<ApplicationFileWithDate> = if let Some(filter_val) = filter {
            application_files_with_date
                .into_iter()
                .filter(|f| f.application_file.id == filter_val)
                .collect()
        } else {
            application_files_with_date
        };

        Ok(filtered_files)
    }

    /// Create New Application
    pub async fn new_from_issue(info: CreateApplicationInfo) -> Result<Self, LDNError> {
        let issue_number = info.issue_number;
        let gh = github_async_new(info.owner.to_string(), info.repo.to_string()).await;
        let (mut parsed_ldn, _) = LDNApplication::parse_application_issue(
            issue_number.clone(),
            info.owner.clone(),
            info.repo.clone(),
        )
        .await?;

        parsed_ldn.datacap.total_requested_amount =
            process_amount(parsed_ldn.datacap.total_requested_amount.clone());
        parsed_ldn.datacap.weekly_allocation =
            process_amount(parsed_ldn.datacap.weekly_allocation.clone());

        let application_id = parsed_ldn.id.clone();
        let file_name = LDNPullRequest::application_path(&application_id);
        let branch_name = LDNPullRequest::application_branch_name(&application_id);

        let multisig_address = if parsed_ldn.datacap.custom_multisig == "[X] Use Custom Multisig" {
            "true".to_string()
        } else {
            "false".to_string()
        };

        match gh.get_file(&file_name, &branch_name).await {
            // If the file does not exist, create a new application file
            Err(_) => {
                log::info!("File not found, creating new application file");
                let application_file = ApplicationFile::new(
                    issue_number.clone(),
                    multisig_address,
                    parsed_ldn.version,
                    parsed_ldn.id.clone(),
                    parsed_ldn.client.clone(),
                    parsed_ldn.project,
                    parsed_ldn.datacap,
                )
                .await;

                let applications = database::applications::get_applications().await.unwrap();

                //check if id is in applications vector
                let app_model = applications.iter().find(|app| app.id == application_id);

                if let Some(app_model) = app_model {
                    // Add a comment to the GitHub issue
                    log::info!("Application already exists in the database");
                    Self::issue_pathway_mismatch_comment(
                        issue_number.clone(),
                        info.owner.clone(),
                        info.repo.clone(),
                        Some(app_model.clone()),
                    )
                    .await?;

                    // Return an error as the application already exists
                    return Err(LDNError::New(
                        "Pathway mismatch: Application already exists".to_string(),
                    ));
                } else {
                    log::info!("Application does not exist in the database");

                    // Check the allowance for the address
                    match get_allowance_for_address(&application_id).await {
                        Ok(allowance) if allowance != "0" => {
                            log::info!("Allowance found and is not zero. Value is {}", allowance);
                            // If allowance is found and is not zero, issue the pathway mismatch comment
                            Self::issue_pathway_mismatch_comment(
                                issue_number.clone(),
                                info.owner.clone(),
                                info.repo.clone(),
                                None,
                            )
                            .await?;

                            return Err(LDNError::New(
                                "Pathway mismatch: Application has already received datacap"
                                    .to_string(),
                            ));
                        }
                        Ok(_) => {
                            log::info!("Allowance not found or is zero");
                        }
                        Err(e) => {
                            //If error contains "DMOB api", add error label and comment to issue
                            if e.to_string().contains("DMOB api") {
                                log::error!("Error getting allowance for address. Unable to access blockchain data");
                                Self::add_error_label(
                                    issue_number.clone(),
                                    "".to_string(),
                                    info.owner.clone(),
                                    info.repo.clone(),
                                )
                                .await?;

                                Self::add_comment_to_issue(
                                    issue_number.clone(),
                                    info.owner.clone(),
                                    info.repo.clone(),
                                    "Unable to access blockchain data for your address. Please contact support.".to_string(),
                                ).await?;

                                return Err(LDNError::New(
                                    "Error getting allowance for address. Unable to access blockchain".to_string(),
                                ));
                            }
                        }
                    }

                    match get_client_allocation(&application_id).await {
                        Ok(response) => {
                            if response.count.is_some() {
                                log::info!("Allocation found for client {}", application_id);
                                Self::issue_pathway_mismatch_comment(
                                    issue_number,
                                    info.owner,
                                    info.repo,
                                    None,
                                )
                                .await?;

                                return Err(LDNError::New(
                                    "Pathway mismatch: Client has already allocation".to_string(),
                                ));
                            } else {
                                log::info!("Client allocation not found");
                            }
                        }
                        Err(e) => {
                            return Err(LDNError::New(format!(
                                "Getting client allocation failed /// {}",
                                e
                            )));
                        }
                    }
                }

                let file_content = match serde_json::to_string_pretty(&application_file) {
                    Ok(f) => f,
                    Err(e) => {
                        Self::add_error_label(
                            application_file.issue_number.clone(),
                            "".to_string(),
                            info.owner.clone(),
                            info.repo.clone(),
                        )
                        .await?;
                        return Err(LDNError::New(format!(
                            "Application issue file is corrupted /// {}",
                            e
                        )));
                    }
                };
                let app_id = parsed_ldn.id.clone();
                let file_sha = LDNPullRequest::create_pr_for_new_application(
                    issue_number.clone(),
                    parsed_ldn.client.name.clone(),
                    branch_name.clone(),
                    LDNPullRequest::application_path(&app_id),
                    file_content.clone(),
                    info.owner.clone(),
                    info.repo.clone(),
                )
                .await?;
                Self::issue_waiting_for_gov_review(
                    issue_number.clone(),
                    info.owner.clone(),
                    info.repo.clone(),
                )
                .await?;
                Self::update_issue_labels(
                    application_file.issue_number.clone(),
                    &[AppState::Submitted.as_str(), "waiting for allocator review"],
                    info.owner.clone(),
                    info.repo.clone(),
                )
                .await?;
                match gh.get_pull_request_by_head(&branch_name).await {
                    Ok(prs) => {
                        if let Some(pr) = prs.first() {
                            let number = pr.number;
                            let issue_number = issue_number.parse::<i64>().map_err(|e| {
                                LDNError::New(format!(
                                    "Parse issue number: {} to i64 failed. {}",
                                    issue_number, e
                                ))
                            })?;
                            database::applications::create_application(
                                application_id.clone(),
                                info.owner.clone(),
                                info.repo.clone(),
                                number,
                                issue_number,
                                file_content,
                                LDNPullRequest::application_path(&app_id),
                            )
                            .await
                            .map_err(|e| {
                                LDNError::New(format!(
                                    "Application issue {} cannot create application in DB /// {}",
                                    application_id, e
                                ))
                            })?;
                        }
                    }
                    Err(e) => log::warn!("Failed to get pull request by head: {}", e),
                }

                Ok(LDNApplication {
                    github: gh,
                    application_id,
                    file_sha,
                    file_name,
                    branch_name,
                })
            }

            // If the file already exists, return an error
            Ok(_) => {
                let app_model = match Self::get_application_model(
                    application_id.clone(),
                    info.owner.clone(),
                    info.repo.clone(),
                )
                .await
                {
                    Ok(model) => Some(model),
                    Err(_) => {
                        return Err(LDNError::New(
                            "Original application file not found in db, but GH file exists"
                                .to_string(),
                        ))
                    }
                };

                // Add a comment to the GitHub issue
                Self::issue_pathway_mismatch_comment(
                    issue_number.clone(),
                    info.owner.clone(),
                    info.repo.clone(),
                    Some(app_model.unwrap()),
                )
                .await?;

                // Return an error as the application already exists
                Err(LDNError::New(
                    "Pathway mismatch: Allocator already assigned".to_string(),
                ))
            }
        }
    }

    /// Move application from Governance Review to Proposal
    pub async fn complete_governance_review(
        &self,
        actor: String,
        owner: String,
        repo: String,
        allocation_amount: String,
        client_contract_address: Option<String>,
    ) -> Result<ApplicationFile, LDNError> {
        match self.app_state().await {
            Ok(s) => match s {
                AppState::KYCRequested
                | AppState::Submitted
                | AppState::AdditionalInfoRequired
                | AppState::AdditionalInfoSubmitted => {
                    let app_file: ApplicationFile = self.file().await?;
                    let allocation_amount_parsed = process_amount(allocation_amount.clone());

                    let db_allocator = match get_allocator(&owner, &repo).await {
                        Ok(allocator) => allocator.unwrap(),
                        Err(err) => {
                            return Err(LDNError::New(format!("Database: get_allocator: {}", err)));
                        }
                    };
                    let db_multisig_address = db_allocator.multisig_address.unwrap();
                    Self::check_and_handle_allowance(
                        &db_multisig_address.clone(),
                        Some(allocation_amount_parsed.clone()),
                    )
                    .await?;

                    let uuid = uuidv4::uuid::v4();
                    let request = AllocationRequest::new(
                        actor.clone(),
                        uuid,
                        AllocationRequestType::First,
                        allocation_amount_parsed,
                    );

                    let app_file = app_file.complete_governance_review(
                        actor.clone(),
                        request,
                        client_contract_address.clone(),
                    );
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    let app_path = &self.file_name.clone();
                    let app_branch = self.branch_name.clone();
                    Self::issue_datacap_request_trigger(
                        app_file.clone(),
                        owner.clone(),
                        repo.clone(),
                    )
                    .await?;
                    match LDNPullRequest::add_commit_to(
                        app_path.to_string(),
                        app_branch.clone(),
                        LDNPullRequest::application_move_to_proposal_commit(&actor),
                        file_content,
                        self.file_sha.clone(),
                        owner.clone(),
                        repo.clone(),
                    )
                    .await
                    {
                        Some(()) => {
                            match self.github.get_pull_request_by_head(&app_branch).await {
                                Ok(prs) => {
                                    if let Some(pr) = prs.first() {
                                        let number = pr.number;
                                        database::applications::update_application(
                                            app_file.id.clone(),
                                            owner.clone(),
                                            repo.clone(),
                                            number,
                                            serde_json::to_string_pretty(&app_file).unwrap(),
                                            Some(app_path.clone()),
                                            None,
                                            client_contract_address,
                                        )
                                        .await
                                        .map_err(|e| {
                                            LDNError::Load(format!(
                                                "Failed to update application: {} /// {}",
                                                app_file.id, e
                                            ))
                                        })?;

                                        Self::issue_datacap_allocation_requested(
                                            app_file.clone(),
                                            app_file.get_active_allocation(),
                                            owner.clone(),
                                            repo.clone(),
                                        )
                                        .await?;
                                        Self::update_issue_labels(
                                            app_file.issue_number.clone(),
                                            &[AppState::ReadyToSign.as_str()],
                                            owner.clone(),
                                            repo.clone(),
                                        )
                                        .await?;
                                        Self::issue_ready_to_sign(
                                            app_file.issue_number.clone(),
                                            owner.clone(),
                                            repo.clone(),
                                        )
                                        .await?;
                                    }
                                }
                                Err(e) => log::warn!("Failed to get pull request by head: {}", e),
                            };
                            Ok(app_file)
                        }
                        None => Err(LDNError::New(format!(
                            "Application issue {} cannot be triggered(1)",
                            self.application_id
                        ))),
                    }
                }
                _ => Err(LDNError::New(format!(
                    "Application issue {} cannot be triggered(2)",
                    self.application_id
                ))),
            },
            Err(e) => Err(LDNError::New(format!(
                "Application issue {} cannot be triggered {}(3)",
                self.application_id, e
            ))),
        }
    }

    /// Move application from Proposal to Approved
    pub async fn complete_new_application_proposal(
        &self,
        signer: VerifierInput,
        request_id: String,
        owner: String,
        repo: String,
        new_allocation_amount: Option<String>,
    ) -> Result<ApplicationFile, LDNError> {
        // TODO: Convert DB errors to LDN Error
        // Get multisig threshold from the database
        let db_allocator = match get_allocator(&owner, &repo).await {
            Ok(allocator) => allocator.unwrap(),
            Err(err) => {
                return Err(LDNError::New(format!("Database: get_allocator: {}", err)));
            }
        };
        let db_multisig_address = db_allocator.multisig_address.unwrap();

        // Get multisig threshold from blockchain
        let blockchain_threshold =
            match get_multisig_threshold_for_actor(&db_multisig_address).await {
                Ok(threshold) => Some(threshold),
                Err(_) => None,
            };

        let db_threshold: u64 = db_allocator.multisig_threshold.unwrap_or(2) as u64;

        // If blockchain threshold is available and different from DB, update DB (placeholder for update logic)
        if let Some(blockchain_threshold) = blockchain_threshold {
            if blockchain_threshold != db_threshold {
                match update_allocator_threshold(&owner, &repo, blockchain_threshold as i32).await {
                    Ok(_) => log::info!("Database updated with new multisig threshold"),
                    Err(e) => log::error!("Failed to update database: {}", e),
                };
            }
        }
        // Use the blockchain threshold if available; otherwise, fall back to the database value
        let threshold_to_use = blockchain_threshold.unwrap_or(db_threshold);

        // Rest of your function logic remains unchanged...
        if threshold_to_use < 2 {
            return self
                .complete_new_application_approval(
                    signer,
                    request_id,
                    owner,
                    repo,
                    new_allocation_amount,
                )
                .await;
        }

        match self.app_state().await {
            Ok(s) => match s {
                AppState::ReadyToSign => {
                    let app_file: ApplicationFile = self.file().await?;
                    if !app_file.allocation.is_active(request_id.clone()) {
                        return Err(LDNError::Load(format!(
                            "Request {} is not active",
                            request_id
                        )));
                    }
                    let app_lifecycle = app_file.lifecycle.finish_proposal();
                    let mut app_file = app_file.add_signer_to_allocation(
                        signer.clone().into(),
                        request_id,
                        app_lifecycle,
                    );
                    if new_allocation_amount.is_some() && app_file.allocation.0.len() > 1 {
                        Self::check_and_handle_allowance(
                            &db_multisig_address.clone(),
                            new_allocation_amount.clone(),
                        )
                        .await?;

                        let new_allocation_amount_parsed =
                            process_amount(new_allocation_amount.clone().unwrap());

                        app_file.adjust_active_allocation_amount(new_allocation_amount_parsed)?;
                    }

                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    match LDNPullRequest::add_commit_to(
                        self.file_name.to_string(),
                        self.branch_name.clone(),
                        LDNPullRequest::application_move_to_approval_commit(
                            &signer.signing_address,
                        ),
                        file_content,
                        self.file_sha.clone(),
                        owner.clone(),
                        repo.clone(),
                    )
                    .await
                    {
                        Some(()) => {
                            match self
                                .github
                                .get_pull_request_by_head(&self.branch_name)
                                .await
                            {
                                Ok(prs) => {
                                    if let Some(pr) = prs.first() {
                                        let number = pr.number;
                                        database::applications::update_application(
                                            app_file.id.clone(),
                                            owner.clone(),
                                            repo.clone(),
                                            number,
                                            serde_json::to_string_pretty(&app_file).unwrap(),
                                            Some(self.file_name.clone()),
                                            None,
                                            app_file.client_contract_address.clone(),
                                        )
                                        .await
                                        .map_err(|e| {
                                            LDNError::Load(format!(
                                                "Failed to update application: {} /// {}",
                                                app_file.id, e
                                            ))
                                        })?;
                                        Self::issue_start_sign_dc(
                                            app_file.issue_number.clone(),
                                            owner.clone(),
                                            repo.clone(),
                                        )
                                        .await?;
                                        Self::issue_datacap_request_signature(
                                            app_file.clone(),
                                            "proposed".to_string(),
                                            owner.clone(),
                                            repo.clone(),
                                        )
                                        .await?;
                                        Self::update_issue_labels(
                                            app_file.issue_number.clone(),
                                            &[AppState::StartSignDatacap.as_str()],
                                            owner.clone(),
                                            repo.clone(),
                                        )
                                        .await?;
                                    }
                                }
                                Err(e) => log::warn!("Failed to get pull request by head: {}", e),
                            };
                            Ok(app_file)
                        }
                        None => Err(LDNError::New(format!(
                            "Application issue {} cannot be proposed(1)",
                            self.application_id
                        ))),
                    }
                }
                _ => Err(LDNError::New(format!(
                    "Application issue {} cannot be proposed(2)",
                    self.application_id
                ))),
            },
            Err(e) => Err(LDNError::New(format!(
                "Application issue {} cannot be proposed {}(3)",
                self.application_id, e
            ))),
        }
    }

    pub async fn complete_sps_change_proposal(
        &self,
        signer: StorageProviderChangeVerifier,
        owner: String,
        repo: String,
        allowed_sps: Option<Vec<u64>>,
        max_deviation: Option<String>,
    ) -> Result<(), LDNError> {
        let db_allocator = get_allocator(&owner, &repo)
            .await
            .map_err(|e| LDNError::Load(format!("Failed to get an allocator. /// {}", e)))?
            .ok_or(LDNError::Load("Allocator not found.".to_string()))?;

        let db_multisig_address = db_allocator.multisig_address.ok_or(LDNError::Load(
            "Multisig address for the allocator not found.".to_string(),
        ))?;

        let blockchain_threshold = get_multisig_threshold_for_actor(&db_multisig_address)
            .await
            .ok();

        let db_threshold: u64 = db_allocator.multisig_threshold.unwrap_or(2) as u64;

        if let Some(blockchain_threshold) = blockchain_threshold {
            if blockchain_threshold != db_threshold {
                if let Err(e) =
                    update_allocator_threshold(&owner, &repo, blockchain_threshold as i32).await
                {
                    log::error!("Failed to update allocator threshold: {}", e);
                }
            }
        }

        let mut app_file: ApplicationFile = self.file().await?;
        let app_state_before_change = app_file.lifecycle.state.clone();
        if app_state_before_change != AppState::ReadyToSign
            && app_state_before_change != AppState::Granted
        {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected Granted or ReadyToSign",
                app_file.lifecycle.state
            )));
        }

        let threshold_to_use = blockchain_threshold.unwrap_or(db_threshold);
        let request_id = uuidv4::uuid::v4();
        let comment: &str;
        let app_state: AppState;
        if threshold_to_use < 2 {
            let sps_change_request =
                SpsChangeRequest::new(&request_id, allowed_sps, max_deviation, &signer, false);
            if let Some(active_allocation) = app_file.allocation.active() {
                app_state = AppState::ReadyToSign;
                app_file = app_file.handle_changing_sps_request(
                    &signer.github_username,
                    &sps_change_request,
                    &app_state,
                    &active_allocation.id,
                );
            } else {
                app_state = AppState::Granted;
                let request_id = uuidv4::uuid::v4();
                app_file = app_file.handle_changing_sps_request(
                    &signer.github_username,
                    &sps_change_request,
                    &app_state,
                    &request_id,
                );
            }
            comment = "Storage Providers have been changed successfully";
        } else {
            app_state = AppState::ChangingSP;
            let sps_change_request: SpsChangeRequest =
                SpsChangeRequest::new(&request_id, allowed_sps, max_deviation, &signer, true);
            app_file = app_file.handle_changing_sps_request(
                &signer.github_username,
                &sps_change_request,
                &app_state,
                &request_id,
            );
            comment =
                "Application is in the Changing Storage Providers state. Waiting for approval.";
        }

        let commit_message = if threshold_to_use < 2 {
            "Update Storage Providers".to_string()
        } else {
            "Start signing allowed storage providers".to_string()
        };

        if app_state_before_change == AppState::ReadyToSign {
            self.update_and_commit_application_state(
                app_file.clone(),
                owner,
                repo,
                self.file_sha.clone(),
                self.branch_name.clone(),
                self.file_name.clone(),
                commit_message,
            )
            .await?;
        } else {
            let pr_title = format!(
                "Set allowed Storage Providers for {}",
                app_file.client.name.clone()
            );
            LDNPullRequest::create_pr_for_existing_application(
                app_file.id.clone(),
                serde_json::to_string_pretty(&app_file).unwrap(),
                self.file_name.clone(),
                request_id.clone(),
                self.file_sha.clone(),
                owner,
                repo,
                true,
                app_file.issue_number.clone(),
                pr_title,
            )
            .await?;
        }

        self.issue_updates(&app_file.issue_number, comment, app_state.as_str())
            .await?;
        Ok(())
    }

    pub async fn complete_sps_change_approval(
        &self,
        signer: StorageProviderChangeVerifier,
        owner: String,
        repo: String,
        request_id: String,
    ) -> Result<(), LDNError> {
        let mut app_file: ApplicationFile = self.file().await?;

        if app_file.lifecycle.state != AppState::ChangingSP {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected Changing SP",
                app_file.lifecycle.state
            )));
        }

        let db_allocator = get_allocator(&owner, &repo)
            .await
            .map_err(|e| LDNError::Load(format!("Failed to get an allocator. /// {}", e)))?
            .ok_or(LDNError::Load("Allocator not found.".to_string()))?;

        let threshold_to_use = db_allocator.multisig_threshold.unwrap_or(2) as usize;

        app_file.allowed_sps = app_file
            .allowed_sps
            .clone()
            .map(|mut requests| requests.add_signer_to_active_request(&request_id, &signer));

        let active_change_request = app_file
            .allowed_sps
            .clone()
            .and_then(|requests| requests.get_active_change_request(&request_id))
            .ok_or(LDNError::Load(
                "Active change request not found. Please propose change firstly".to_string(),
            ))?;
        let app_state: AppState;
        let comment: String;
        let commit_message: String;
        if active_change_request.signers.0.len() == threshold_to_use {
            app_file.allowed_sps = app_file
                .allowed_sps
                .clone()
                .map(|mut requests| requests.complete_change_request(&request_id));
            if let Some(active_allocation) = app_file.allocation.active() {
                app_state = AppState::ReadyToSign;
                app_file.lifecycle = app_file.lifecycle.update_lifecycle_after_sign(
                    &app_state,
                    &signer.github_username,
                    &active_allocation.id,
                );
            } else {
                app_state = AppState::Granted;
                app_file.lifecycle = app_file.lifecycle.update_lifecycle_after_sign(
                    &app_state,
                    &signer.github_username,
                    &request_id,
                );
            }
            comment = "Storage Providers have been changed successfully.".to_string();
            commit_message = "Finalize request to change storage providers.".to_string();
        } else {
            app_state = AppState::ChangingSP;
            app_file.lifecycle = app_file.lifecycle.update_lifecycle_after_sign(
                &app_state,
                &signer.github_username,
                &request_id,
            );
            comment = format!(
                "Verifier {} signed a request to change storage providers.",
                signer.github_username
            );
            commit_message = "Add signer to request to change storage providers.".to_string();
        }

        self.update_and_commit_application_state(
            app_file.clone(),
            owner,
            repo,
            self.file_sha.clone(),
            self.branch_name.clone(),
            self.file_name.clone(),
            commit_message,
        )
        .await?;

        self.issue_updates(&app_file.issue_number, &comment, app_state.as_str())
            .await?;
        Ok(())
    }

    pub async fn complete_new_application_approval(
        &self,
        signer: VerifierInput,
        request_id: String,
        owner: String,
        repo: String,
        new_allocation_amount: Option<String>,
    ) -> Result<ApplicationFile, LDNError> {
        // Get multisig threshold from the database
        let db_allocator = match get_allocator(&owner, &repo).await {
            Ok(allocator) => allocator.unwrap(),
            Err(err) => {
                return Err(LDNError::New(format!("Database: get_allocator: {}", err)));
            }
        };
        let threshold_to_use = db_allocator.multisig_threshold.unwrap_or(2) as usize;

        let app_state = self.app_state().await?;

        if app_state != AppState::StartSignDatacap
            && !(threshold_to_use == 1 && app_state == AppState::ReadyToSign)
        {
            return Err(LDNError::New(format!(
                "Application issue {} cannot be approved in its current state",
                self.application_id
            )));
        }

        let mut app_file: ApplicationFile = self.file().await?;
        let app_lifecycle = app_file.lifecycle.finish_approval();

        // Find the signers that already signed
        let current_signers = app_file
            .allocation
            .0
            .iter()
            .find(|&alloc| alloc.id == request_id && alloc.is_active)
            .map_or(vec![], |alloc| alloc.signers.0.clone());

        // // Check if the signer has already signed
        if current_signers
            .iter()
            .any(|s| s.signing_address == signer.signing_address)
        {
            return Err(LDNError::New(format!(
                "Signer {} has already approved this application",
                signer.signing_address
            )));
        }

        // Check if the number of signers meets or exceeds the multisig threshold
        let multisig_threshold_usize = threshold_to_use as usize;
        if current_signers.len() >= multisig_threshold_usize {
            return Err(LDNError::New(
                "No additional signatures needed as the multisig threshold is already met"
                    .to_string(),
            ));
        }

        // Check the allowance for the address

        if new_allocation_amount.is_some() && app_file.allocation.0.len() > 1 {
            let new_allocation_amount_parsed =
                process_amount(new_allocation_amount.clone().unwrap());

            app_file.adjust_active_allocation_amount(new_allocation_amount_parsed)?;
        }

        // Add signer to signers array
        let app_file = app_file.add_signer_to_allocation_and_complete(
            signer.clone().into(),
            request_id.clone(),
            app_lifecycle,
        );

        let file_content = serde_json::to_string_pretty(&app_file).unwrap();
        let commit_result = LDNPullRequest::add_commit_to(
            self.file_name.to_string(),
            self.branch_name.clone(),
            LDNPullRequest::application_move_to_confirmed_commit(&signer.signing_address),
            file_content,
            self.file_sha.clone(),
            owner.clone(),
            repo.clone(),
        )
        .await;

        match commit_result {
            Some(()) => {
                match self
                    .github
                    .get_pull_request_by_head(&self.branch_name)
                    .await
                {
                    Ok(prs) => {
                        if let Some(pr) = prs.first() {
                            let number = pr.number;
                            if let Err(e) = database::applications::update_application(
                                app_file.id.clone(),
                                owner.clone(),
                                repo.clone(),
                                number,
                                serde_json::to_string_pretty(&app_file).unwrap(),
                                Some(self.file_name.clone()),
                                None,
                                app_file.client_contract_address.clone(),
                            )
                            .await
                            {
                                log::warn!("Failed to update application in database: {}", e);
                                return Err(LDNError::New(format!(
                                    "Database update failed for application issue {}",
                                    self.application_id
                                )));
                            }

                            Self::issue_datacap_request_signature(
                                app_file.clone(),
                                "approved".to_string(),
                                owner.clone(),
                                repo.clone(),
                            )
                            .await?;
                            Self::update_issue_labels(
                                app_file.issue_number.clone(),
                                &[AppState::Granted.as_str()],
                                owner.clone(),
                                repo.clone(),
                            )
                            .await?;
                            Self::issue_granted(
                                app_file.issue_number.clone(),
                                owner.clone(),
                                repo.clone(),
                            )
                            .await?;
                        }
                        Ok(app_file)
                    }
                    Err(e) => {
                        log::warn!("Failed to get pull request by head: {}", e);
                        Err(LDNError::New(format!(
                            "Pull request retrieval failed for application issue {}",
                            self.application_id
                        )))
                    }
                }
            }
            None => {
                log::warn!(
                    "Failed to add commit for application issue {}",
                    self.application_id
                );
                Err(LDNError::New(format!(
                                "Commit operation failed for application issue {} and no error details available",
                                self.application_id
                            )))
            }
        }
    }

    async fn parse_application_issue(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<(ParsedIssue, String), LDNError> {
        let gh = github_async_new(owner.to_string(), repo.to_string()).await;
        let issue = gh
            .list_issue(issue_number.parse().unwrap())
            .await
            .map_err(|e| {
                LDNError::Load(format!(
                    "Failed to retrieve issue {} from GitHub. Reason: {}",
                    issue_number, e
                ))
            })?;
        if let Some(issue_body) = issue.body {
            Ok((ParsedIssue::from_issue_body(&issue_body), issue.user.login))
        } else {
            Err(LDNError::Load(format!(
                "Failed to retrieve issue {} from GitHub. Reason: {}",
                issue_number, "No body"
            )))
        }
    }

    pub async fn check_application_exists(
        app_model: ApplicationModel,
        application_id: String,
    ) -> Result<bool, LDNError> {
        let app_str = app_model.application.ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an application field",
                application_id
            ))
        })?;

        let db_application: ApplicationFile = ApplicationFile::from_str(&app_str).map_err(|e| {
            LDNError::Load(format!("Failed to parse application file from DB: {}", e))
        })?;

        if db_application.id == application_id {
            Ok(true) // It exists
        } else {
            Ok(false) // Return false if application doesn't exist
        }
    }

    /// Return Application state
    async fn app_state(&self) -> Result<AppState, LDNError> {
        let f = self.file().await?;
        Ok(f.lifecycle.get_state())
    }

    /// Return Application state
    pub async fn total_dc_reached(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let merged = Self::merged(owner.clone(), repo.clone()).await?;
        let app = merged
            .par_iter()
            .find_first(|(_, app)| app.id == application_id);
        if app.is_some() && app.unwrap().1.lifecycle.get_state() == AppState::Granted {
            let app = app.unwrap().1.reached_total_datacap();
            let gh = github_async_new(owner.to_string(), repo.to_string()).await;
            let ldn_app =
                LDNApplication::load(application_id.clone(), owner.clone(), repo.clone()).await?;
            let ContentItems { items } = gh.get_file(&ldn_app.file_name, "main").await.unwrap();
            Self::issue_full_dc(app.issue_number.clone(), owner.clone(), repo.clone()).await?;
            Self::update_issue_labels(
                app.issue_number.clone(),
                &[AppState::TotalDatacapReached.as_str()],
                owner.clone(),
                repo.clone(),
            )
            .await?;

            let pr_title = format!("Total Datacap reached for {}", app.id);

            LDNPullRequest::create_pr_for_existing_application(
                app.id.clone(),
                serde_json::to_string_pretty(&app).unwrap(),
                ldn_app.file_name.clone(),
                format!("{}-total-dc-reached", app.id),
                items[0].sha.clone(),
                owner,
                repo,
                true,
                app.issue_number.clone(),
                pr_title,
            )
            .await?;
            Ok(true)
        } else {
            Err(LDNError::Load(format!(
                "Application issue {} does not exist",
                application_id
            )))
        }
    }

    fn content_items_to_app_file(file: ContentItems) -> Result<ApplicationFile, LDNError> {
        let f = &file
            .clone()
            .take_items()
            .first()
            .and_then(|f| f.content.clone())
            .and_then(|f| base64::decode_application_file(&f.replace('\n', "")))
            .ok_or(LDNError::Load("Application file is corrupted".to_string()))?;
        Ok(f.clone())
    }

    pub async fn file(&self) -> Result<ApplicationFile, LDNError> {
        match self
            .github
            .get_file(&self.file_name, &self.branch_name)
            .await
        {
            Ok(file) => LDNApplication::content_items_to_app_file(file),
            Err(e) => {
                dbg!(&e);
                Err(LDNError::Load(format!(
                    "Application issue {} file does not exist ///",
                    self.application_id
                )))
            }
        }
    }

    pub async fn fetch_verifiers(
        owner: String,
        repo: String,
    ) -> Result<ValidVerifierList, LDNError> {
        let allocator = database::allocators::get_allocator(&owner, &repo)
            .await
            .map_err(|e| LDNError::Load(format!("Failed to retrieve allocators /// {}", e)))?;

        let mut verifiers_handles = Vec::new();

        let allocator = match allocator {
            Some(a) => a,
            None => return Err(LDNError::Load("No allocator found".into())),
        };

        if let Some(handles) = allocator.verifiers_gh_handles {
            verifiers_handles.extend(handles.split(',').map(|s| s.trim().to_string()));
        }

        if verifiers_handles.is_empty() {
            return Err(LDNError::Load("No review team found".into()));
        }

        Ok(ValidVerifierList {
            verifiers: verifiers_handles,
        })
    }

    async fn single_merged(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<(ApplicationGithubInfo, ApplicationFile), LDNError> {
        LDNApplication::merged(owner, repo)
            .await?
            .into_iter()
            .find(|(_, app)| app.id == application_id)
            .map_or_else(
                || {
                    Err(LDNError::Load(format!(
                        "Application issue {} does not exist",
                        application_id
                    )))
                },
                Ok,
            )
    }

    async fn map_merged(item: Content) -> Result<Option<(Content, ApplicationFile)>, LDNError> {
        if item.download_url.is_none() {
            return Ok(None);
        }
        let file = reqwest::Client::new()
            .get(item.download_url.clone().unwrap())
            .send()
            .await
            .map_err(|e| LDNError::Load(format!("here {}", e)))?;
        let file = file
            .text()
            .await
            .map_err(|e| LDNError::Load(format!("here1 {}", e)))?;
        let app = match ApplicationFile::from_str(&file) {
            Ok(app) => {
                if app.lifecycle.is_active {
                    app
                } else {
                    return Ok(None);
                }
            }
            Err(_) => {
                return Ok(None);
            }
        };
        Ok(Some((item, app)))
    }

    pub async fn merged(
        owner: String,
        repo: String,
    ) -> Result<Vec<(ApplicationGithubInfo, ApplicationFile)>, LDNError> {
        // Retrieve all applications in the main branch from the database.
        let merged_apps_result = database::applications::get_merged_applications(
            Some(owner.clone()),
            Some(repo.clone()),
        )
        .await;

        // Handle errors in getting applications from the main branch.
        let merged_app_models = match merged_apps_result {
            Ok(apps) => apps,
            Err(e) => return Err(LDNError::Load(format!("Database error: {}", e))),
        };

        // Convert applications from the main branch.
        let mut merged_apps: Vec<(ApplicationGithubInfo, ApplicationFile)> = Vec::new();
        for app_model in merged_app_models {
            // Try to deserialize the `application` field to `ApplicationFile`.
            if let Some(app_json) = app_model.application {
                if let Ok(app) = from_str::<ApplicationFile>(&app_json) {
                    merged_apps.push((
                        ApplicationGithubInfo {
                            sha: app_model.sha.unwrap(),
                            path: app_model.path.unwrap(),
                        },
                        app,
                    ));
                }
            }
        }

        let active_apps = Self::active(owner, repo, None).await?;
        let mut apps: Vec<(ApplicationGithubInfo, ApplicationFile)> = vec![];
        for app in merged_apps {
            if !active_apps.iter().any(|a| a.id == app.1.id) && app.1.lifecycle.is_active {
                apps.push(app);
            }
        }

        Ok(apps)
    }

    async fn refill(verfier: &str, refill_info: RefillInfo) -> Result<bool, LDNError> {
        let apps =
            LDNApplication::merged(refill_info.owner.clone(), refill_info.repo.clone()).await?;
        if let Some((content, mut app)) = apps.into_iter().find(|(_, app)| app.id == refill_info.id)
        {
            let uuid = uuidv4::uuid::v4();
            let request_id = uuid.clone();
            let new_request = AllocationRequest::new(
                verfier.to_string(),
                request_id.clone(),
                AllocationRequestType::Refill(0),
                format!("{}{}", refill_info.amount, refill_info.amount_type),
            );
            let app_file = app.start_refill_request(new_request);
            Self::issue_refill(
                app.issue_number.clone(),
                refill_info.owner.clone(),
                refill_info.repo.clone(),
            )
            .await?;

            let pr_title = format!("Datacap for {}", app.client.name.clone());

            LDNPullRequest::create_pr_for_existing_application(
                app.id.clone(),
                serde_json::to_string_pretty(&app_file).unwrap(),
                content.path.clone(), // filename
                request_id.clone(),
                content.sha,
                refill_info.owner,
                refill_info.repo,
                true,
                app_file.issue_number.clone(),
                pr_title,
            )
            .await?;
            return Ok(true);
        }
        Err(LDNError::Load("Failed to get application file".to_string()))
    }

    pub async fn notify_refill(info: NotifyRefillInfo) -> Result<(), LDNError> {
        let label = "Refill needed";

        let gh = github_async_new(info.owner.clone(), info.repo.clone()).await;
        let issue_number = info.issue_number.parse().map_err(|e| {
            LDNError::Load(format!("Failed to parse issue number to number: {:?}", e))
        })?;
        let has_label = gh.issue_has_label(issue_number, label).await.map_err(|e| {
            LDNError::Load(format!(
                "Failed to check if issue has refill label: {:?}",
                e
            ))
        })?;
        if has_label {
            return Err(LDNError::Load(format!(
                "'{}' label present - already notified about refill!",
                label
            )));
        }

        let comment = String::from(
            "Client used 75% of the allocated DataCap. Consider allocating next tranche.",
        );
        Self::add_comment_to_issue(
            info.issue_number.clone(),
            info.owner.clone(),
            info.repo.clone(),
            comment,
        )
        .await?;
        Self::update_issue_labels(
            info.issue_number.clone(),
            &[label],
            info.owner.clone(),
            info.repo.clone(),
        )
        .await?;
        Ok(())
    }

    pub async fn validate_merge_application(
        pr_number: u64,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        log::info!("Starting validate_merge_application:");
        log::info!("- Validating merge for PR number {}", pr_number,);

        let application =
            match LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await {
                Ok(app) => {
                    log::info!("- Got application");
                    app
                }
                Err(err) => {
                    log::error!("- Failed to get application. Reason: {}", err);
                    return Err(LDNError::Load(format!(
                        "Failed to get application. Reason: {}",
                        err
                    )));
                }
            };

        // conditions for automerge:
        // 1. Application is in Granted state
        // 2. Application has Validated by and Validated at fields set
        // 3. Application doesn't have an active request
        // 4. Application does not have edited = true in lifecycle object
        if application.lifecycle.get_state() == AppState::Granted {
            if application.lifecycle.validated_by.is_empty() {
                log::warn!("- Application has not been validated");
                return Ok(false);
            }
            if application.lifecycle.validated_at.is_empty() {
                log::warn!("- Application has not been validated at");
                return Ok(false);
            }
            let active_request = application.allocation.active();
            if active_request.is_some() {
                log::warn!("- Application has an active request");
                return Ok(false);
            }
            if application.lifecycle.edited.unwrap_or(false) {
                log::warn!("Val Trigger - Application has been edited");
                return Ok(false);
            }
            log::info!("- Application is in a valid state!");

            Self::merge_application(pr_number, owner, repo).await?;
            return Ok(true);
        }

        log::warn!("- Application is not in a valid state");
        Ok(false)
    }

    pub async fn merge_application(
        pr_number: u64,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner.to_string(), repo.to_string()).await;

        gh.merge_pull_request(pr_number).await.map_err(|e| {
            LDNError::Load(format!(
                "Failed to merge pull request {}. Reason: {}",
                pr_number, e
            ))
        })?;

        database::applications::merge_application_by_pr_number(owner, repo, pr_number)
            .await
            .map_err(|e| {
                LDNError::Load(format!(
                    "Failed to update application in database. Reason: {}",
                    e
                ))
            })?;

        Ok(true)
    }

    pub async fn validate_flow(
        pr_number: u64,
        actor: &str,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        log::info!("Starting validate_flow:");
        log::info!(
            "- Validating flow for PR number {} with user handle {}",
            pr_number,
            actor
        );

        let gh = github_async_new(owner.to_string(), repo.to_string()).await;
        let author = match gh.get_last_commit_author(pr_number).await {
            Ok(author) => {
                log::info!("- Last commit author: {}", author);
                author
            }
            Err(err) => {
                log::error!("- Failed to get last commit author. Reason: {}", err);
                return Err(LDNError::Load(format!(
                    "Failed to get last commit author. Reason: {}",
                    err
                )));
            }
        };

        if author.is_empty() {
            log::warn!("- Author is empty");
            return Ok(false);
        }

        let (_, files) = match gh.get_pull_request_files(pr_number).await {
            Ok(files) => {
                log::info!("- Got Pull request files");
                files
            }
            Err(err) => {
                log::error!("- Failed to get pull request files. Reason: {}", err);
                return Err(LDNError::Load(format!(
                    "Failed to get pull request files. Reason: {}",
                    err
                )));
            }
        };

        if files.len() != 1 {
            log::warn!("- Number of files in pull request is not equal to 1");
            return Ok(false);
        }

        let branch_name = match gh.get_branch_name_from_pr(pr_number).await {
            Ok(branch_name) => {
                log::info!("- Branch name: {}", branch_name);
                branch_name
            }
            Err(err) => {
                log::error!(
                    "- Failed to get branch name from pull request. Reason: {}",
                    err
                );
                return Err(LDNError::Load(format!(
                    "Failed to get branch name from pull request. Reason: {}",
                    err
                )));
            }
        };

        let application = match gh.get_file(&files[0].filename, &branch_name).await {
            Ok(file) => {
                log::info!("- Got File content");
                LDNApplication::content_items_to_app_file(file)?
            }
            Err(err) => {
                log::error!("- Failed to get file content. Reason: {}", err);
                return Err(LDNError::Load(format!(
                    "Failed to get file content. Reason: {}",
                    err
                )));
            }
        };

        // Check if application is in Submitted state
        let state = application.lifecycle.get_state();
        if state == AppState::KYCRequested
            || state == AppState::Submitted
            || state == AppState::AdditionalInfoRequired
            || state == AppState::AdditionalInfoSubmitted
        {
            if !application.lifecycle.validated_by.is_empty() {
                log::warn!(
                    "- Application has already been validated by: {}",
                    application.lifecycle.validated_by
                );
                return Ok(false);
            }
            if !application.lifecycle.validated_at.is_empty() {
                log::warn!(
                    "- Application has already been validated at: {}",
                    application.lifecycle.validated_at
                );
                return Ok(false);
            }
            let active_request = application.allocation.active();
            if active_request.is_some() {
                log::warn!("- Application has an active request");
                return Ok(false);
            }
            if !application.allocation.0.is_empty() {
                log::warn!("- Application has allocations");
                return Ok(false);
            }
            log::info!("- Application is in a valid state!");
            return Ok(true);
        }
        // let bot_user = get_env_var_or_default("BOT_USER");
        // if author != bot_user {
        //     log::warn!("- Author is not the bot user");
        //     return Ok(false);
        // }

        log::info!("- Application is in a valid state");
        Ok(true)
    }

    pub async fn validate_trigger(
        pr_number: u64,
        actor: &str,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        log::info!("Starting validate_trigger:");
        log::info!(
            "- Validating trigger for PR number {} with user handle {}",
            pr_number,
            actor
        );

        if let Ok(application_file) =
            LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await
        {
            if !application_file.lifecycle.get_active_status() {
                log::info!("No trigger to validate. Application lifecycle is inactive so the Total DC was reached.");
                return Ok(true);
            }
            let validated_by = application_file.lifecycle.validated_by.clone();
            let validated_at = application_file.lifecycle.validated_at.clone();
            let app_state = application_file.lifecycle.get_state();
            let valid_verifier_list = Self::fetch_verifiers(owner.clone(), repo.clone()).await?;
            // let bot_user = get_env_var_or_default("BOT_USER");

            if application_file.lifecycle.edited.unwrap_or(false) {
                log::warn!("Val Trigger - Application has been edited");
                return Ok(false);
            }

            let res: bool = match app_state {
                AppState::KYCRequested => {
                    log::warn!("Val Trigger (RtS) - Application state is KYCRequested");
                    return Ok(false);
                }
                AppState::AdditionalInfoRequired => {
                    log::warn!("Val Trigger (RtS) - Application state is MoreInfoNeeded");
                    return Ok(false);
                }
                AppState::AdditionalInfoSubmitted => {
                    log::warn!("Val Trigger (RtS) - Application state is MoreInfoNeeded");
                    return Ok(false);
                }
                AppState::Submitted => {
                    log::warn!("Val Trigger (RtS) - Application state is Submitted");
                    return Ok(false);
                }
                AppState::ChangesRequested => {
                    log::warn!("Val Trigger (RtS) - Application state is ChangesRequested");
                    return Ok(false);
                }
                AppState::ReadyToSign => {
                    if application_file.allocation.0.is_empty() {
                        log::warn!("Val Trigger (RtS) - No allocations found");
                        false
                    } else {
                        let active_allocation = application_file.get_active_allocation();

                        if active_allocation.is_none() {
                            log::warn!("Val Trigger (RtS) - Active allocation not found");
                            false
                        } else if !active_allocation.unwrap().signers.0.is_empty() {
                            log::warn!("Val Trigger (RtS) - Active allocation has signers");
                            false
                        } else if validated_at.is_empty() {
                            log::warn!(
                                "Val Trigger (RtS) - Not ready to sign - validated_at is empty"
                            );
                            false
                        } else if validated_by.is_empty() {
                            log::warn!(
                                "Val Trigger (RtS) - Not ready to sign - validated_by is empty"
                            );
                            false
                        } else if !valid_verifier_list.is_valid(&validated_by) {
                            log::warn!("Val Trigger (RtS) - Not ready to sign - valid_verifier_list is not valid");
                            false
                        } else {
                            log::info!("Val Trigger (RtS) - Validated!");
                            true
                        }
                        // else if actor != bot_user {
                        //     log::warn!(
                        //         "Val Trigger (RtS) - Not ready to sign - actor is not the bot user"
                        //     );
                        //     false
                        // }
                    }
                }
                AppState::StartSignDatacap => {
                    if !validated_at.is_empty()
                        && !validated_by.is_empty()
                        && valid_verifier_list.is_valid(&validated_by)
                    {
                        log::info!("Val Trigger (SSD) - Validated!");
                        true
                    } else {
                        if validated_at.is_empty() {
                            log::warn!("Val Trigger (SSD) - AppState: StartSignDatacap, validation failed: validated_at is empty");
                        }
                        if validated_by.is_empty() {
                            log::warn!("Val Trigger (SSD) - AppState: StartSignDatacap, validation failed: validated_by is empty");
                        }
                        if !valid_verifier_list.is_valid(&validated_by) {
                            log::warn!("Val Trigger (SSD) - AppState: StartSignDatacap, validation failed: valid_verifier_list is not valid");
                        }
                        false
                    }
                }
                AppState::Granted => {
                    if !validated_at.is_empty()
                        && !validated_by.is_empty()
                        && valid_verifier_list.is_valid(&validated_by)
                    {
                        log::info!("Val Trigger (G) - Application is granted");
                        true
                    } else {
                        if validated_at.is_empty() {
                            log::warn!(
                                "Val Trigger (G) - AppState: Granted, validation failed: validated_at is empty"
                            );
                        }
                        if validated_by.is_empty() {
                            log::warn!(
                                "Val Trigger (G) - AppState: Granted, validation failed: validated_by is empty"
                            );
                        }
                        if !valid_verifier_list.is_valid(&validated_by) {
                            log::warn!(
                                "Val Trigger (G) - AppState: Granted, validation failed: valid_verifier_list is not valid"
                            );
                        }
                        false
                    }
                }
                AppState::TotalDatacapReached => {
                    log::info!("Val Trigger (TDR) - Application state is TotalDatacapReached");
                    true
                }
                AppState::ChangingSP => {
                    log::warn!("Val Trigger (RtS) - Application state is ChangingSP");
                    return Ok(false);
                }
                AppState::Error => {
                    log::warn!("Val Trigger (TDR) - Application state is Error");
                    return Ok(false);
                }
            };

            if res {
                log::info!("Validated!");
                return Ok(true);
            }

            let app_file = application_file.move_back_to_governance_review();
            let ldn_application =
                LDNApplication::load(app_file.id.clone(), owner.clone(), repo.clone()).await?;

            if let Some(()) = LDNPullRequest::add_commit_to(
                ldn_application.file_name.clone(),
                ldn_application.branch_name.clone(),
                "Move application back to review".to_string(),
                serde_json::to_string_pretty(&app_file).unwrap(),
                ldn_application.file_sha.clone(),
                owner.clone(),
                repo.clone(),
            )
            .await
            {
                let gh = github_async_new(owner.to_string(), repo.to_string()).await;
                match gh
                    .get_pull_request_by_head(&ldn_application.branch_name)
                    .await
                {
                    Ok(prs) => {
                        if let Some(pr) = prs.first() {
                            let number = pr.number;
                            let _ = database::applications::update_application(
                                app_file.id.clone(),
                                owner,
                                repo,
                                number,
                                serde_json::to_string_pretty(&app_file).unwrap(),
                                Some(ldn_application.file_name.clone()),
                                None,
                                app_file.client_contract_address,
                            )
                            .await;
                        }
                    }
                    Err(e) => log::warn!("Failed to get pull request by head: {}", e),
                };
            };

            return Ok(false);
        };

        log::info!("Failed to fetch Application File");
        Ok(false)
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_and_commit_application_state(
        &self,
        db_application_file: ApplicationFile,
        owner: String,
        repo: String,
        sha: String,
        branch_name: String,
        filename: String,
        commit_message: String,
    ) -> Result<ApplicationFile, LDNError> {
        // Changed return type to include ApplicationFile

        // Serialize the updated application file
        let file_content = match serde_json::to_string_pretty(&db_application_file) {
            Ok(f) => f,
            Err(e) => {
                Self::add_error_label(
                    db_application_file.issue_number.clone(),
                    "".to_string(),
                    owner.clone(),
                    repo.clone(),
                )
                .await?;
                return Err(LDNError::New(format!(
                    "Application issue file is corrupted /// {}",
                    e
                )));
            }
        };

        // Commit the changes to the branch
        match LDNPullRequest::add_commit_to(
            filename.clone(),
            branch_name.clone(),
            commit_message,
            file_content,
            sha.clone(),
            owner.clone(),
            repo.clone(),
        )
        .await
        {
            Some(()) => {
                // Retrieve and update the pull request
                match self.github.get_pull_request_by_head(&branch_name).await {
                    Ok(prs) => {
                        if let Some(pr) = prs.first() {
                            let number = pr.number;
                            let update_result = database::applications::update_application(
                                db_application_file.id.clone(),
                                owner.clone(),
                                repo.clone(),
                                number,
                                serde_json::to_string_pretty(&db_application_file).unwrap(),
                                Some(filename.clone()),
                                None,
                                db_application_file.client_contract_address.clone(),
                            )
                            .await;

                            match update_result {
                                Ok(_) => Ok(db_application_file), // Return the updated ApplicationFile
                                Err(e) => {
                                    log::error!("Failed to update application: {}", e);
                                    Err(LDNError::New(
                                        "Failed to update the application in the database"
                                            .to_string(),
                                    ))
                                }
                            }
                        } else {
                            Err(LDNError::New(
                                "No pull request found for the given branch".to_string(),
                            ))
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to get pull request by head: {}", e);
                        Err(LDNError::New(format!("Failed to get pull request: {}", e)))
                    }
                }
            }
            None => Err(LDNError::New(
                "Adding commit in approve changes failed".to_string(),
            )),
        }
    }

    pub async fn approve_changes(self, owner: String, repo: String) -> Result<String, LDNError> {
        let sha: String = self.file_sha.clone();
        let filename: String = self.file_name.clone();
        let branch_name: String = self.branch_name.clone();
        let application_id: String = self.application_id.clone();

        let db_application_file_str_result = database::applications::get_application(
            application_id,
            owner.clone(),
            repo.clone(),
            None,
        )
        .await;
        let db_application_file_str = match db_application_file_str_result {
            Ok(file) => file
                .application
                .unwrap_or_else(|| panic!("Application data is missing")), // Consider more graceful error handling here
            Err(e) => {
                return Err(LDNError::New(format!(
                    "Failed to fetch application data from the database: {}",
                    e
                )));
            }
        };

        let mut db_application_file: ApplicationFile =
            serde_json::from_str::<ApplicationFile>(&db_application_file_str.clone()).unwrap();
        let application_state = db_application_file.lifecycle.state.clone();

        if application_state != AppState::ChangesRequested {
            return Err(LDNError::New(
                "Application is not in the correct state".to_string(),
            ));
        }

        let allocation_count = db_application_file.allocation.0.len();

        if allocation_count == 0 {
            return Err(LDNError::New(
                "Application does not have any allocations".to_string(),
            ));
        }

        let active_allocation = db_application_file.allocation.active();
        let mut remove_allocation = false;

        let active_allocation_ref = match active_allocation.as_ref() {
            Some(allocation) => allocation,
            None => {
                db_application_file.lifecycle.state = AppState::Granted;
                db_application_file.lifecycle.edited = Some(false);

                let _ = self
                    .finalize_changes_approval(
                        db_application_file,
                        owner,
                        repo,
                        sha,
                        branch_name,
                        filename,
                    )
                    .await;
                return Ok("Changes approved".to_string()); // or return an error if appropriate
            }
        };

        if allocation_count == 1 && active_allocation_ref.signers.0.is_empty() {
            // case with exactly ONE allocation which is active, but not signed yet
            remove_allocation = true;
            db_application_file.lifecycle.state = AppState::Submitted
        } else if active_allocation_ref.signers.0.is_empty() {
            // case with more than one allocations one of which is active, but not signed yet
            remove_allocation = true;
            db_application_file.lifecycle.state = AppState::Granted
        } else {
            // case with more than one allocations one of which is active and signed, and the number of signatures is 2 because otherwise there'd be no active one
            db_application_file.lifecycle.state = AppState::StartSignDatacap
        };

        db_application_file.lifecycle.edited = Some(false);

        if remove_allocation {
            db_application_file.remove_active_allocation();
        }

        let _ = self
            .finalize_changes_approval(db_application_file, owner, repo, sha, branch_name, filename)
            .await;

        Ok("Changes approved".to_string())
    }

    async fn finalize_changes_approval(
        self,
        db_application_file: ApplicationFile,
        owner: String,
        repo: String,
        sha: String,
        branch_name: String,
        filename: String,
    ) -> Result<String, LDNError> {
        self.update_and_commit_application_state(
            db_application_file.clone(),
            owner.clone(),
            repo.clone(),
            sha.clone(),
            branch_name.clone(),
            filename.clone(),
            "Changes approved".to_string(),
        )
        .await?;
        Self::issue_changes_approved(
            db_application_file.issue_number.clone(),
            owner,
            repo,
            db_application_file.lifecycle.state.clone(),
        )
        .await?;
        Ok("Changes approved".to_string())
    }

    pub async fn check_for_changes(
        pr_number: u64,
        author: &str,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        log::info!("Starting check_for_changes:");

        let bot_user = get_env_var_or_default("BOT_USER");
        if author != bot_user {
            log::warn!("- Author is not the bot user");
            return Err(LDNError::New("PR File edited by user".to_string()));
        }

        let gh: GithubWrapper = github_async_new(owner.clone(), repo.clone()).await;
        let result = Self::get_pr_files_and_app(owner.clone(), repo.clone(), pr_number).await;

        let sha: String;
        let filename: String;
        let mut application_file: ApplicationFile;

        match result {
            Ok(Some(((_, files), app))) => {
                if let Some(file) = files.first() {
                    sha = file.sha.clone();
                    filename = file.filename.clone();
                    application_file = app;
                } else {
                    return Err(LDNError::New(
                        "No files found in the pull request".to_string(),
                    ));
                }
            }
            Ok(None) => {
                return Err(LDNError::New(
                    "Failed to fetch PR files or application file".to_string(),
                ));
            }
            Err(e) => {
                return Err(e);
            }
        };

        if !application_file.lifecycle.edited.unwrap_or(false) {
            log::warn!("Val Trigger - Application has not been edited");
            return Ok(true);
        }

        let allocation_count = application_file.allocation.0.len();

        if allocation_count == 0 {
            return Err(LDNError::New(
                "Application does not have any allocations".to_string(),
            ));
        }

        let application_id: String = application_file.id.clone();

        let db_application_file_str_result = database::applications::get_application(
            application_file.id.clone(),
            owner.clone(),
            repo.clone(),
            None,
        )
        .await;
        let db_application_file_str = match db_application_file_str_result {
            Ok(file) => file
                .application
                .unwrap_or_else(|| panic!("Application data is missing")), // Consider more graceful error handling here
            Err(e) => {
                return Err(LDNError::New(format!(
                    "Failed to fetch application data from the database: {}",
                    e
                )));
            }
        };
        let db_application_file: ApplicationFile =
            serde_json::from_str::<ApplicationFile>(&db_application_file_str.clone()).unwrap();

        application_file.lifecycle.edited = Some(false);
        let commit_message = if allocation_count == 1
            && application_file.allocation.active().is_some()
            && application_file
                .allocation
                .active()
                .unwrap()
                .signers
                .0
                .is_empty()
        {
            application_file.lifecycle.state = AppState::Submitted;
            application_file.allocation = Allocations(Vec::new());
            "Updated application state to Verifier Review due to changes requested on the issue and no signed allocations."
        } else {
            application_file.lifecycle.state = AppState::ChangesRequested;
            "Updated application state to Changes Requested due to changes requested on the issue and at leasts one partially or fully signed allocation."
        };
        let file_content = match serde_json::to_string_pretty(&application_file) {
            Ok(f) => f,
            Err(e) => {
                Self::add_error_label(
                    application_file.issue_number.clone(),
                    "".to_string(),
                    owner.clone(),
                    repo.clone(),
                )
                .await?;
                return Err(LDNError::New(format!(
                    "Application issue file is corrupted /// {}",
                    e
                )));
            }
        };
        let branch_name = match gh.get_branch_name_from_pr(pr_number).await {
            Ok(branch_name) => {
                log::info!("- Branch name: {}", branch_name);
                branch_name
            }
            Err(err) => {
                log::error!(
                    "- Failed to get branch name from pull request. Reason: {}",
                    err
                );
                return Err(LDNError::Load(format!(
                    "Failed to get branch name from pull request. Reason: {}",
                    err
                )));
            }
        };

        match database::applications::get_application_by_pr_number(
            owner.clone(),
            repo.clone(),
            pr_number,
        )
        .await
        {
            Ok(_) => {
                let _ = database::applications::update_application(
                    application_id,
                    owner.clone(),
                    repo.clone(),
                    pr_number,
                    file_content.clone(),
                    Some(filename.clone()),
                    None,
                    application_file.client_contract_address.clone(),
                )
                .await
                .map_err(|e| {
                    LDNError::Load(format!(
                        "Failed to update application in the database: {}",
                        e
                    ))
                })?;
            }
            Err(_) => {
                let issue_number = application_file.issue_number.parse::<i64>().map_err(|e| {
                    LDNError::New(format!(
                        "Parse issue number: {} to i64 failed. {}",
                        application_file.issue_number, e
                    ))
                })?;
                database::applications::create_application(
                    application_id,
                    owner.clone(),
                    repo.clone(),
                    pr_number,
                    issue_number,
                    file_content.clone(),
                    filename.clone(),
                )
                .await
                .map_err(|e| {
                    LDNError::Load(format!(
                        "Failed to create application in the database: {}",
                        e
                    ))
                })?;
            }
        }

        gh.update_file(&filename, commit_message, &file_content, &branch_name, &sha)
            .await
            .map_err(|e| {
                LDNError::Load(format!(
                    "Failed to update file in GitHub repo {}/{}. Reason: {} in file {}",
                    gh.owner.clone(),
                    gh.repo.clone(),
                    e,
                    filename
                ))
            })?;

        let differences = application_file.compare(&db_application_file);

        Self::issue_changes_requested(
            application_file.clone(),
            owner.clone(),
            repo.clone(),
            differences,
        )
        .await?;

        Ok(true)
    }

    pub async fn validate_approval(
        pr_number: u64,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        log::info!("Starting validate_approval:");
        log::info!("Validating approval for PR number {}", pr_number);
        match LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await {
            Ok(application_file) => {
                if !application_file.lifecycle.get_active_status() {
                    log::info!("No approval to validate. Application lifecycle is inactive so the Total DC was reached.");
                    return Ok(true);
                }
                let app_state: AppState = application_file.lifecycle.get_state();
                if application_file.lifecycle.edited.unwrap_or(false) {
                    log::warn!("Val Trigger - Application has been edited");
                    return Ok(false);
                }

                log::info!("Val Approval - App state is {:?}", app_state.as_str());
                if app_state < AppState::Granted {
                    log::warn!("Val Approval < (G)- State is less than Granted");
                    Ok(false)
                } else if app_state == AppState::Granted {
                    let active_request_id = match application_file
                        .clone()
                        .lifecycle
                        .get_active_allocation_id()
                    {
                        Some(id) => id,
                        None => {
                            log::warn!("Val Approval (G) - No active request");
                            return Ok(false);
                        }
                    };
                    let active_request =
                        match application_file.allocation.find_one(active_request_id) {
                            Some(request) => request,
                            None => {
                                log::warn!("Val Approval (G) - No active request");
                                return Ok(false);
                            }
                        };

                    let db_allocator = match get_allocator(&owner, &repo).await {
                        Ok(allocator) => allocator.unwrap(),
                        Err(err) => {
                            return Err(LDNError::New(format!("Database: get_allocator: {}", err)));
                        }
                    };
                    let db_multisig_threshold =
                        db_allocator.multisig_threshold.unwrap_or(2) as usize;
                    let signers: application::file::Verifiers = active_request.signers.clone();

                    // Check if the number of signers meets or exceeds the multisig threshold
                    if signers.0.len() < db_multisig_threshold {
                        log::warn!("Not enough signers for approval");
                        return Ok(false);
                    }
                    let signer_index = if db_multisig_threshold <= 1 { 0 } else { 1 };

                    let signer = signers.0.get(signer_index).unwrap();
                    let signer_gh_handle = signer.github_username.clone();

                    let valid_verifiers: ValidVerifierList =
                        Self::fetch_verifiers(owner.clone(), repo.clone()).await?;

                    if valid_verifiers.is_valid(&signer_gh_handle) {
                        log::info!("Val Approval (G)- Validated!");
                        return Ok(true);
                    }

                    log::warn!("Val Approval (G) - Not validated!");
                    Ok(false)
                } else {
                    log::info!("Val Approval > (G) - State is greater than Granted");
                    Ok(true)
                }
            }
            Err(e) => Err(LDNError::Load(format!(
                "PR number {} not found: {}",
                pr_number, e
            ))),
        }
    }

    pub async fn validate_proposal(
        pr_number: u64,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        log::info!("Starting validate_proposal:");
        log::info!("- Validating proposal for PR number {}", pr_number);
        match LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await {
            Ok(application_file) => {
                if !application_file.lifecycle.get_active_status() {
                    log::info!("No proposal to validate. Application lifecycle is inactive so the Total DC was reached.");
                    return Ok(true);
                }
                let app_state: AppState = application_file.lifecycle.get_state();
                log::info!("Val Proposal - App state is {:?}", app_state.as_str());
                if application_file.lifecycle.edited.unwrap_or(false) {
                    log::warn!("Val Trigger - Application has been edited");
                    return Ok(false);
                }

                if app_state < AppState::StartSignDatacap {
                    log::warn!("Val Proposal (< SSD) - State is less than StartSignDatacap");
                    Ok(false)
                } else if app_state == AppState::StartSignDatacap {
                    let active_request = application_file.allocation.active();
                    if active_request.is_none() {
                        log::warn!("Val Proposal (SSD)- No active request");
                        return Ok(false);
                    }
                    let active_request = active_request.unwrap();
                    let signers = active_request.signers.clone();
                    if signers.0.len() != 1 {
                        log::warn!("Val Proposal (SSD) - Not enough signers");
                        return Ok(false);
                    }
                    let signer = signers.0.first().unwrap();
                    let signer_gh_handle = signer.github_username.clone();
                    let valid_verifiers =
                        Self::fetch_verifiers(owner.clone(), repo.clone()).await?;
                    if valid_verifiers.is_valid(&signer_gh_handle) {
                        log::info!("Val Proposal (SSD) - Validated!");
                        return Ok(true);
                    }
                    log::warn!("Val Proposal (SSD) - Not validated!");
                    Ok(false)
                } else {
                    log::info!("Val Proposal (> SSD) - State is greater than StartSignDatacap");
                    Ok(true)
                }
            }
            Err(e) => Err(LDNError::Load(format!(
                "PR number {} not found: {}",
                pr_number, e
            ))),
        }
    }

    /**
     * Updates the application when an issue is modified. It searches for the PR through the issue number and updates the application file.
     *
     * # Arguments
     * `info` - The information to update the application with.
     *
     * # Returns
     * `Result<LDNApplication, LDNError>` - The updated application.
     */
    pub async fn update_from_issue(info: CreateApplicationInfo) -> Result<Self, LDNError> {
        // Get the PR number from the issue number.
        let issue_number = info.issue_number.clone();
        let (mut parsed_ldn, _) = LDNApplication::parse_application_issue(
            issue_number.clone(),
            info.owner.clone(),
            info.repo.clone(),
        )
        .await?;
        let application_id = parsed_ldn.id.clone();

        let application_model = match Self::get_application_model(
            application_id.clone(),
            info.owner.clone(),
            info.repo.clone(),
        )
        .await
        {
            Ok(app) => app,
            Err(e) => {
                log::warn!("Failed to get application model: {}. ", e);
                let parsed_issue_number = issue_number.parse::<i64>().map_err(|e| {
                    LDNError::New(format!(
                        "Parse issue number: {} to i64 failed. {}",
                        issue_number, e
                    ))
                })?;
                //Application Id has not been found. That means the user has modified the wallet address
                let application = database::applications::get_application_by_issue_number(
                    info.owner.clone(),
                    info.repo.clone(),
                    parsed_issue_number,
                )
                .await;
                if application.is_ok() {
                    Self::add_comment_to_issue(issue_number, info.owner.clone(),info.repo.clone(), "Application exist. If you have modified the wallet address, please create a new application.".to_string()).await?;
                    return Err(LDNError::New(format!(
                        "Application exist: {}",
                        application_id
                    )));
                } else {
                    return Self::new_from_issue(info).await;
                }
            }
        };

        parsed_ldn.datacap.total_requested_amount =
            process_amount(parsed_ldn.datacap.total_requested_amount.clone());
        parsed_ldn.datacap.weekly_allocation =
            process_amount(parsed_ldn.datacap.weekly_allocation.clone());

        //Application was granted. Create a new PR with the updated application file, as if it was a new application
        if application_model.pr_number == 0 {
            return Self::create_pr_from_issue_modification(parsed_ldn, application_model).await;
        }

        //Application was in another state. Update PR and add "edited = true" to the application file
        Self::edit_pr_from_issue_modification(parsed_ldn, application_model).await
    }

    pub async fn edit_pr_from_issue_modification(
        parsed_ldn: ParsedIssue,
        application_model: ApplicationModel,
    ) -> Result<Self, LDNError> {
        //Get existing application file
        let mut pr_application = ApplicationFile::from_str(&application_model.application.unwrap())
            .map_err(|e| LDNError::Load(format!("Failed to parse application file from DB: {}", e)))
            .unwrap();

        if pr_application.lifecycle.get_state() == AppState::AdditionalInfoRequired {
            pr_application.lifecycle.state = AppState::AdditionalInfoSubmitted;
            let _ = Self::issue_additional_info_submitted(
                pr_application.issue_number.clone(),
                application_model.owner.clone(),
                application_model.repo.clone(),
            )
            .await;
        }

        let application_id = parsed_ldn.id.clone();

        //Edit the application file with the new info from the issue
        let mut app_file = ApplicationFile::edited(
            pr_application.issue_number.clone(),
            parsed_ldn.version,
            application_id.clone(),
            parsed_ldn.client.clone(),
            parsed_ldn.project,
            parsed_ldn.datacap,
            pr_application.allocation.clone(),
            pr_application.lifecycle.clone(),
            pr_application.client_contract_address.clone(),
            pr_application.allowed_sps,
        )
        .await;

        if app_file.allocation.0.is_empty() {
            app_file.lifecycle.edited = Some(false);
        }

        let file_content = match serde_json::to_string_pretty(&app_file) {
            Ok(f) => f,
            Err(e) => {
                Self::add_error_label(
                    app_file.issue_number.clone(),
                    "".to_string(),
                    application_model.owner.clone(),
                    application_model.repo.clone(),
                )
                .await?;
                return Err(LDNError::New(format!(
                    "Application issue file is corrupted /// {}",
                    e
                )));
            }
        };

        //Create a new commit with the updated application file
        let gh = github_async_new(
            application_model.owner.to_string(),
            application_model.repo.to_string(),
        )
        .await;
        let branch_name = gh
            .get_branch_name_from_pr(application_model.pr_number as u64)
            .await
            .unwrap();
        match LDNPullRequest::add_commit_to(
            application_model.path.clone().unwrap(),
            branch_name.clone(),
            format!(
                "Update application from issue #{}",
                pr_application.issue_number
            ),
            file_content.clone(),
            application_model.sha.clone().unwrap(),
            application_model.owner.clone(),
            application_model.repo.clone(),
        )
        .await
        {
            Some(()) => {
                if app_file.allocation.0.is_empty() {
                    match gh.get_pull_request_by_head(&branch_name).await {
                        Ok(prs) => {
                            if let Some(pr) = prs.first() {
                                let number = pr.number;

                                database::applications::update_application(
                                    app_file.id.clone(),
                                    application_model.owner.clone(),
                                    application_model.repo.clone(),
                                    number,
                                    serde_json::to_string_pretty(&app_file).unwrap(),
                                    Some(application_model.path.clone().unwrap()),
                                    None,
                                    app_file.client_contract_address,
                                )
                                .await
                                .map_err(|e| {
                                    LDNError::Load(format!(
                                        "Failed to update application: {} /// {}",
                                        app_file.id, e
                                    ))
                                })?;
                            }
                        }
                        Err(e) => log::warn!("Failed to get pull request by head: {}", e),
                    };
                }
                Ok(LDNApplication {
                    github: gh,
                    application_id,
                    file_sha: application_model.sha.clone().unwrap(),
                    file_name: application_model.path.clone().unwrap(),
                    branch_name,
                })
            }
            None => Err(LDNError::New(format!(
                "Application issue {} cannot be modified",
                app_file.issue_number
            ))),
        }
    }

    async fn check_and_handle_allowance(
        db_multisig_address: &str,
        new_allocation_amount: Option<String>,
    ) -> Result<(), LDNError> {
        match get_allowance_for_address(db_multisig_address).await {
            Ok(allowance) if allowance != "0" => {
                log::info!("Allowance found and is not zero. Value is {}", allowance);
                match compare_allowance_and_allocation(&allowance, new_allocation_amount) {
                    Some(result) => {
                        if result {
                            println!("Allowance is sufficient.");
                            Ok(())
                        } else {
                            println!("Allowance is not sufficient.");
                            Err(LDNError::New("Multisig address has less allowance than the new allocation amount".to_string()))
                        }
                    }
                    None => {
                        println!("Error parsing sizes.");
                        Err(LDNError::New("Error parsing sizes".to_string()))
                    }
                }
            }
            Ok(_) => Err(LDNError::New(
                "Multisig address has no remaining allowance".to_string(),
            )),
            Err(e) => {
                log::error!("Failed to retrieve allowance: {:?}", e);
                Err(LDNError::New("Failed to retrieve allowance".to_string()))
            }
        }
    }

    pub async fn create_pr_from_issue_modification(
        parsed_ldn: ParsedIssue,
        application_model: ApplicationModel,
    ) -> Result<Self, LDNError> {
        let merged_application = ApplicationFile::from_str(&application_model.application.unwrap())
            .map_err(|e| LDNError::Load(format!("Failed to parse application file from DB: {}", e)))
            .unwrap();

        let application_id = parsed_ldn.id.clone();

        //Create new application file with updated info from issue
        let application_file = ApplicationFile::edited(
            merged_application.issue_number.clone(),
            parsed_ldn.version,
            application_id.clone(),
            parsed_ldn.client.clone(),
            parsed_ldn.project,
            parsed_ldn.datacap,
            merged_application.allocation.clone(),
            merged_application.lifecycle.clone(),
            merged_application.client_contract_address.clone(),
            merged_application.allowed_sps.clone(),
        )
        .await;

        let file_content = match serde_json::to_string_pretty(&application_file) {
            Ok(f) => f,
            Err(e) => {
                Self::add_error_label(
                    application_file.issue_number.clone(),
                    "".to_string(),
                    application_model.owner.clone(),
                    application_model.repo.clone(),
                )
                .await?;
                return Err(LDNError::New(format!(
                    "Application issue file is corrupted /// {}",
                    e
                )));
            }
        };

        let file_name = LDNPullRequest::application_path(&application_id);
        let branch_name = LDNPullRequest::application_branch_name(&application_id);
        let uuid = uuidv4::uuid::v4();

        let pr_title = format!("Issue modification for {}", parsed_ldn.client.name.clone());

        LDNPullRequest::create_pr_for_existing_application(
            application_id.clone(),
            file_content.clone(),
            LDNPullRequest::application_path(&application_id),
            uuid,
            application_model.sha.clone().unwrap(),
            application_model.owner.clone(),
            application_model.repo.clone(),
            false,
            application_file.issue_number.clone(),
            pr_title,
        )
        .await?;

        let gh = github_async_new(
            application_model.owner.to_string(),
            application_model.repo.to_string(),
        )
        .await;

        Ok(LDNApplication {
            github: gh,
            application_id,
            file_sha: application_model.sha.clone().unwrap(),
            file_name,
            branch_name,
        })
    }

    pub async fn delete_branch(
        owner: String,
        repo: String,
        branch_name: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        let request = gh.build_remove_ref_request(branch_name.clone()).unwrap();

        gh.remove_branch(request).await.map_err(|e| {
            LDNError::New(format!("Error deleting branch {} /// {}", branch_name, e))
        })?;

        Ok(true)
    }

    async fn add_comment_to_issue(
        issue_number: String,
        owner: String,
        repo: String,
        comment: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })?;

        Ok(true)
    }

    async fn issue_waiting_for_gov_review(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        Self::add_comment_to_issue(
            issue_number,
            owner,
            repo,
            "Application is waiting for allocator review".to_string(),
        )
        .await?;

        Ok(true)
    }

    async fn issue_pathway_mismatch_comment(
        issue_number: String,
        info_owner: String,
        info_repo: String,
        db_model: Option<ApplicationModel>,
    ) -> Result<bool, LDNError> {
        let mut comment = "The wallet address in this application has previously received datacap from another source. Please update the application to use a new client wallet address, so that it is clear that datacap usage is associated with this application.".to_string();

        if let Some(db_model) = db_model {
            //Load application from db_model.application string json
            let application =
                ApplicationFile::from_str(&db_model.application.unwrap()).map_err(|e| {
                    LDNError::Load(format!("Failed to parse application file from DB: {}", e))
                })?;

            comment = if db_model.owner == info_owner
                && db_model.repo == info_repo
                && application.issue_number == issue_number
            {
                //if issue number is the same, do not add the comment
                "There's no need to retry this application. File already exists.".to_string()
            } else if db_model.owner == info_owner && db_model.repo == info_repo {
                // Application already exists in the same repository
                format!(
                    "This wallet address was already used in application #{} for this pathway. Please continue in that application instead.",
                    application.issue_number
                )
            } else {
                // Application exists in a different repository
                format!(
                    "This client address has also applied for datacap at http://github.com/{}/{}/issues/{} - Please use a new, distinct client address for this application, so that usage can be clearly understood as relating to this application.",
                    db_model.owner, db_model.repo, application.issue_number
                )
            };
        }

        dbg!(&comment);
        let gh = github_async_new(info_owner.clone(), info_repo.clone()).await;
        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })?;

        Ok(true)
    }

    async fn issue_datacap_request_trigger(
        application_file: ApplicationFile,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;

        let client_address = application_file.lifecycle.client_on_chain_address.clone();
        let total_requested = application_file.datacap.total_requested_amount.clone();
        let weekly_allocation = application_file.datacap.weekly_allocation.clone();
        let allocation_amount = application_file
            .allocation
            .0
            .iter()
            .find(|obj| Some(&obj.id) == application_file.lifecycle.active_request.as_ref())
            .unwrap()
            .amount
            .clone();

        let issue_number = application_file.issue_number.clone();

        let comment = format!(
            "### Datacap Request Trigger
**Total DataCap requested**
> {}

**Expected weekly DataCap usage rate**
> {}

**DataCap Amount - First Tranche**
> {}

**Client address**
> {}",
            total_requested, weekly_allocation, allocation_amount, client_address
        );

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        Ok(true)
    }

    async fn issue_changes_requested(
        application_file: ApplicationFile,
        owner: String,
        repo: String,
        differences: Vec<String>,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner.clone(), repo.clone()).await;

        let issue_number = application_file.issue_number.clone();

        let comment = format!(
            "### Issue has been modified. Changes below:

_(NEW vs OLD)_

\n>{}",
            differences.join("\n>")
        );

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        Self::update_issue_labels(
            application_file.issue_number.clone(),
            &[AppState::ChangesRequested.as_str()],
            owner,
            repo,
        )
        .await?;
        Ok(true)
    }

    async fn issue_changes_approved(
        issue_number: String,
        owner: String,
        repo: String,
        new_state: AppState,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner.clone(), repo.clone()).await;

        let comment = "#### Issue information change request has been approved.".to_string();

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        Self::update_issue_labels(issue_number, &[new_state.as_str()], owner, repo).await?;
        Ok(true)
    }

    async fn issue_datacap_allocation_requested(
        application_file: ApplicationFile,
        active_allocation: Option<&Allocation>,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;

        let issue_number = application_file.issue_number.clone();

        let mut datacap_allocation_requested = String::new();
        let mut id = String::new();

        if let Some(allocation) = active_allocation {
            datacap_allocation_requested.clone_from(&allocation.amount);
            id.clone_from(&allocation.id);
        }

        let comment = format!(
            "## DataCap Allocation requested

#### Multisig Notary address
> {}

#### Client address
> {}

#### DataCap allocation requested
> {}

#### Id
> {}",
            application_file.datacap.identifier.clone(),
            application_file.lifecycle.client_on_chain_address.clone(),
            datacap_allocation_requested,
            id
        );

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        Ok(true)
    }

    async fn issue_datacap_request_signature(
        application_file: ApplicationFile,
        signature_step: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let active_allocation: Option<&Allocation> =
            application_file.allocation.0.iter().find(|obj| {
                Some(&obj.id) == application_file.lifecycle.active_request.clone().as_ref()
            });

        let gh = github_async_new(owner, repo).await;

        let issue_number = application_file.issue_number.clone();

        let signature_step_capitalized = signature_step
            .chars()
            .nth(0)
            .unwrap()
            .to_uppercase()
            .to_string()
            + &signature_step.chars().skip(1).collect::<String>();

        let mut datacap_allocation_requested = String::new();
        let mut id = String::new();
        let mut signing_address = String::new();
        let mut message_cid = String::new();
        let mut increase_allowance_cid: Option<String> = None;

        if let Some(allocation) = active_allocation {
            datacap_allocation_requested.clone_from(&allocation.amount);
            id.clone_from(&allocation.id);

            if let Some(first_verifier) = allocation.signers.0.first() {
                signing_address.clone_from(&first_verifier.signing_address);
                message_cid.clone_from(&first_verifier.message_cid);
                increase_allowance_cid = first_verifier.increase_allowance_cid.clone();
            }
        }

        let additional_status_message =
            increase_allowance_cid
                .clone()
                .map_or("".to_string(), |increase_allowance_cid| {
                    format!(
                        ", and here https://filfox.info/en/message/{}",
                        increase_allowance_cid
                    )
                });

        let comment = format!(
            "## Request {}
Your Datacap Allocation Request has been {} by the Notary
#### Message sent to Filecoin Network
> {} {}
#### Address
> {}
#### Datacap Allocated
> {}
#### Signer Address
> {}
#### Id
> {}
#### You can check the status here https://filfox.info/en/message/{}{}",
            signature_step_capitalized,
            signature_step,
            message_cid,
            increase_allowance_cid.unwrap_or_default(),
            application_file.lifecycle.client_on_chain_address.clone(),
            datacap_allocation_requested,
            signing_address,
            id,
            message_cid,
            additional_status_message
        );

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        Ok(true)
    }

    async fn issue_ready_to_sign(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            "Application is ready to sign",
        )
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ))
        })
        .unwrap();
        Ok(true)
    }

    async fn issue_start_sign_dc(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            "Application is in the process of signing datacap",
        )
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ))
        })
        .unwrap();
        Ok(true)
    }

    async fn issue_granted(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(issue_number.parse().unwrap(), "Application is Granted")
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        Ok(true)
    }
    async fn issue_refill(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(issue_number.parse().unwrap(), "Application is in Refill")
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        gh.replace_issue_labels(issue_number.parse().unwrap(), &["Refill".to_string()])
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        Ok(true)
    }
    async fn issue_full_dc(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(issue_number.parse().unwrap(), "Application is Completed")
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        Ok(true)
    }

    async fn issue_additional_info_required(
        issue_number: String,
        owner: String,
        repo: String,
        verifier_message: String,
    ) -> Result<bool, LDNError> {
        let comment = format!(
            "## Additional Information Requested
#### A verifier has reviewed your application and has issued the following message:

> {}

_The initial issue can be edited in order to solve the request of the verifier. The changes will be reflected in the application and an automatic comment will be posted in order to let the verifiers know the updated application can be reviewed._",
            verifier_message
        );
        let gh = github_async_new(owner, repo).await;
        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        gh.replace_issue_labels(
            issue_number.parse().unwrap(),
            &["Additional Info Required".to_string()],
        )
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ))
        })
        .unwrap();

        Ok(true)
    }

    async fn issue_additional_info_submitted(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;

        let issue_number = issue_number.clone();

        let comment =
            "#### The application's issue was edited after additional information was requested"
                .to_string();

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        gh.replace_issue_labels(
            issue_number.parse().unwrap(),
            &["Additional Info Submitted".to_string()],
        )
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ))
        })
        .unwrap();
        Ok(true)
    }

    async fn issue_application_declined(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<bool, LDNError> {
        let gh = github_async_new(owner, repo).await;

        let issue_number = issue_number.clone();

        let comment = "### The application has been declined.".to_string();

        gh.add_comment_to_issue(issue_number.parse().unwrap(), &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        gh.replace_issue_labels(issue_number.parse().unwrap(), &["Declined".to_string()])
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();
        Ok(true)
    }

    async fn add_error_label(
        issue_number: String,
        comment: String,
        owner: String,
        repo: String,
    ) -> Result<(), LDNError> {
        let gh = github_async_new(owner, repo).await;
        let num: u64 = issue_number.parse().expect("Not a valid integer");
        gh.add_error_label(num, comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding labels to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        Ok(())
    }

    async fn update_issue_labels(
        issue_number: String,
        new_labels: &[&str],
        owner: String,
        repo: String,
    ) -> Result<(), LDNError> {
        let gh = github_async_new(owner, repo).await;
        let num: u64 = issue_number.parse().expect("Not a valid integer");
        let new_labels: Vec<String> = new_labels.iter().map(|&s| s.to_string()).collect();
        gh.replace_issue_labels(num, &new_labels)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding labels t to issue {} /// {}",
                    issue_number, e
                ))
            })
            .unwrap();

        Ok(())
    }

    pub async fn cache_renewal_active(owner: String, repo: String) -> Result<(), LDNError> {
        let active_from_gh: Vec<ApplicationFileWithDate> =
            LDNApplication::active_apps_with_last_update(owner.clone(), repo.clone(), None).await?;
        let active_from_db: Vec<ApplicationModel> =
            database::applications::get_active_applications(
                Some(owner.clone()),
                Some(repo.clone()),
            )
            .await
            .unwrap();

        let mut db_apps_set: HashSet<String> = HashSet::new();
        let mut processed_gh_apps: HashSet<String> = HashSet::new();

        for db_app in active_from_db.iter() {
            db_apps_set.insert(db_app.id.clone());
            if let Some(gh_app) = active_from_gh.iter().find(|&x| {
                x.application_file.id == db_app.id && x.pr_number == db_app.pr_number as u64
            }) {
                if gh_app.updated_at > db_app.updated_at {
                    database::applications::update_application(
                        db_app.id.clone(),
                        owner.clone(),
                        repo.clone(),
                        db_app.pr_number as u64,
                        serde_json::to_string_pretty(&gh_app.application_file).unwrap(),
                        None,
                        Some(gh_app.sha.clone()),
                        gh_app.application_file.client_contract_address.clone(),
                    )
                    .await
                    .unwrap();
                }
                // If the app is in GH, remove it from the set to not consider it for deletion
                db_apps_set.remove(&db_app.id);
                processed_gh_apps.insert(gh_app.application_file.id.clone());
            } else {
                // If the app is not in GH, call the delete_application function
                database::applications::delete_application(
                    db_app.id.clone(),
                    owner.clone(),
                    repo.clone(),
                    db_app.pr_number as u64,
                )
                .await
                .unwrap();
            }
        }

        // Iterates over the active apps in GitHub to create the ones that are not in the database
        for gh_app in active_from_gh {
            if !db_apps_set.contains(&gh_app.application_file.id)
                && !processed_gh_apps.contains(&gh_app.application_file.id)
            {
                let issue_number = gh_app
                    .application_file
                    .issue_number
                    .parse::<i64>()
                    .map_err(|e| {
                        LDNError::New(format!(
                            "Parse issue number: {} to i64 failed. {}",
                            gh_app.application_file.issue_number, e
                        ))
                    })?;
                // Call the create_application function if the GH app is not in DB
                database::applications::create_application(
                    gh_app.application_file.id.clone(),
                    owner.clone(),
                    repo.clone(),
                    gh_app.pr_number,
                    issue_number,
                    serde_json::to_string_pretty(&gh_app.application_file).unwrap(),
                    gh_app.path,
                )
                .await
                .unwrap();
            }
        }

        Ok(())
    }

    pub async fn cache_renewal_merged(owner: String, repo: String) -> Result<(), LDNError> {
        let merged_from_gh: Vec<ApplicationFileWithDate> =
            LDNApplication::merged_apps_with_last_update(owner.clone(), repo.clone(), None).await?;
        let merged_from_db: Vec<ApplicationModel> =
            database::applications::get_merged_applications(
                Some(owner.clone()),
                Some(repo.clone()),
            )
            .await
            .unwrap();

        let mut db_apps_set: HashSet<String> = HashSet::new();
        let mut processed_gh_apps: HashSet<String> = HashSet::new();

        for db_app in merged_from_db.iter() {
            db_apps_set.insert(db_app.id.clone());
            if let Some(gh_app) = merged_from_gh
                .iter()
                .find(|&x| x.application_file.id == db_app.id)
            {
                if gh_app.updated_at > db_app.updated_at {
                    database::applications::update_application(
                        db_app.id.clone(),
                        owner.clone(),
                        repo.clone(),
                        0,
                        serde_json::to_string_pretty(&gh_app.application_file).unwrap(),
                        Some(gh_app.path.clone()),
                        Some(gh_app.sha.clone()),
                        gh_app.application_file.client_contract_address.clone(),
                    )
                    .await
                    .unwrap();
                }
                // If the app is in GH, remove it from the set to not consider it for deletion
                db_apps_set.remove(&db_app.id);
                processed_gh_apps.insert(gh_app.application_file.id.clone());
            } else {
                // If the app is not in GH, call the delete_application function
                database::applications::delete_application(
                    db_app.id.clone(),
                    owner.clone(),
                    repo.clone(),
                    db_app.pr_number as u64,
                )
                .await
                .unwrap();
            }
        }

        // Iterates over the active apps in GitHub to create the ones that are not in the database
        for gh_app in merged_from_gh {
            if !db_apps_set.contains(&gh_app.application_file.id)
                && !processed_gh_apps.contains(&gh_app.application_file.id)
            {
                let issue_number = gh_app
                    .application_file
                    .issue_number
                    .parse::<i64>()
                    .map_err(|e| {
                        LDNError::New(format!(
                            "Parse issue number: {} to i64 failed. {}",
                            gh_app.application_file.issue_number, e
                        ))
                    })?;
                // Call the create_application function if the GH app is not in DB
                database::applications::create_application(
                    gh_app.application_file.id.clone(),
                    owner.clone(),
                    repo.clone(),
                    0,
                    issue_number,
                    serde_json::to_string_pretty(&gh_app.application_file).unwrap(),
                    gh_app.path,
                )
                .await
                .unwrap();
            }
        }

        Ok(())
    }

    pub async fn decline_application(&self, owner: String, repo: String) -> Result<(), LDNError> {
        // Retrieve the application model from the database.
        let app_model = database::applications::get_application(
            self.application_id.clone(),
            owner.clone(),
            repo.clone(),
            None,
        )
        .await
        .map_err(|_| LDNError::Load("No application found".to_string()))?;

        // Check if the application is associated with a PR.
        if app_model.pr_number == 0 {
            return Err(LDNError::New("Application is not in a PR".to_string()));
        }

        let db_application_file: ApplicationFile = serde_json::from_str::<ApplicationFile>(
            &app_model.clone().application.unwrap().clone(),
        )
        .unwrap();

        if db_application_file.lifecycle.get_state() > AppState::Submitted {
            return Err(LDNError::New(
                "Application is in a state that cannot be declined".to_string(),
            ));
        }

        let issue_number = self
            .file()
            .await
            .map_err(|_| LDNError::Load("Failed to retrieve file details".into()))?
            .issue_number;
        LDNApplication::issue_application_declined(
            issue_number.clone(),
            owner.clone(),
            repo.clone(),
        )
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Failed to issue application declined notification: {}",
                e
            ))
        })?;

        // Attempt to close the associated pull request.
        LDNPullRequest::close_pull_request(owner.clone(), repo.clone(), app_model.pr_number as u64)
            .await
            .map_err(|e| LDNError::New(format!("Failed to close PR: {}", e)))?;

        // Attempt to delete the associated branch.
        LDNPullRequest::delete_branch(app_model.id, owner.clone(), repo.clone())
            .await
            .map_err(|e| LDNError::New(format!("Failed to delete branch: {}", e)))?;

        // Delete the application from the database.
        database::applications::delete_application(
            self.application_id.clone(),
            owner.clone(),
            repo.clone(),
            app_model.pr_number as u64,
        )
        .await
        .map_err(|_| LDNError::New("Failed to delete application".to_string()))?;

        // Issue a notification about the application decline.

        Ok(())
    }

    pub async fn additional_info_required(
        self,
        owner: String,
        repo: String,
        verifier_message: String,
    ) -> Result<ApplicationFile, LDNError> {
        // Adjusted return type to include ApplicationFile
        let sha: String = self.file_sha.clone();
        let filename: String = self.file_name.clone();
        let branch_name: String = self.branch_name.clone();
        let application_id: String = self.application_id.clone();

        let db_application_file_str_result = database::applications::get_application(
            application_id,
            owner.clone(),
            repo.clone(),
            None,
        )
        .await;
        let db_application_file_str = match db_application_file_str_result {
            Ok(file) => file
                .application
                .unwrap_or_else(|| panic!("Application data is missing")), // Consider more graceful error handling here
            Err(e) => {
                return Err(LDNError::New(format!(
                    "Failed to fetch application data from the database: {}",
                    e
                )));
            }
        };

        let mut db_application_file: ApplicationFile =
            serde_json::from_str::<ApplicationFile>(&db_application_file_str.clone()).unwrap();
        db_application_file.lifecycle.state = AppState::AdditionalInfoRequired;

        if db_application_file.lifecycle.get_state() > AppState::Submitted {
            return Err(LDNError::New(
                "Application is in a state in which additional info cannot be requested"
                    .to_string(),
            ));
        }

        // Adjusted to capture the result of update_and_commit_application_state
        let updated_application = self
            .update_and_commit_application_state(
                db_application_file.clone(),
                owner.clone(),
                repo.clone(),
                sha.clone(),
                branch_name.clone(),
                filename.clone(),
                "Additional information required".to_string(),
            )
            .await?;

        let _ = Self::issue_additional_info_required(
            db_application_file.issue_number.clone(),
            owner.clone(),
            repo.clone(),
            verifier_message.clone(),
        )
        .await;

        Ok(updated_application) // Return the updated ApplicationFile
    }

    pub async fn request_kyc(self, id: &str, owner: &str, repo: &str) -> Result<(), LDNError> {
        let app_model =
            Self::get_application_model(id.to_string(), owner.to_string(), repo.to_string())
                .await?;

        let app_str = app_model.application.ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an application field",
                id
            ))
        })?;
        let application_file = serde_json::from_str::<ApplicationFile>(&app_str).unwrap();
        if application_file.lifecycle.state != AppState::Submitted {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected Submitted",
                application_file.lifecycle.state
            )));
        }

        let application_file = application_file.kyc_request();

        database::applications::update_application(
            id.to_string(),
            owner.to_string(),
            repo.to_string(),
            app_model.pr_number.try_into().map_err(|e| {
                LDNError::Load(format!(
                    "Parse PR number: {} to u64 failed  /// {}",
                    app_model.pr_number, e
                ))
            })?,
            serde_json::to_string_pretty(&application_file).unwrap(),
            app_model.path.clone(),
            None,
            application_file.client_contract_address.clone(),
        )
        .await
        .expect("Failed to update_application in DB!");

        self.issue_updates_for_kyc(&application_file.issue_number.parse().unwrap())
            .await?;

        self.update_and_commit_application_state(
            application_file.clone(),
            owner.to_string(),
            repo.to_string(),
            app_model.sha.unwrap(),
            LDNPullRequest::application_branch_name(&application_file.id),
            app_model.path.unwrap(),
            "KYC requested".to_string(),
        )
        .await?;
        Ok(())
    }

    pub async fn issue_updates_for_kyc(&self, issue_number: &u64) -> Result<(), LDNError> {
        let comment = format!(
            "KYC has been requested. Please complete KYC at {}/?owner={}&repo={}&client={}&issue={}", 
            get_env_var_or_default("KYC_URL"),
            &self.github.owner,
            &self.github.repo,
            &self.application_id,
            issue_number,
        );

        self.github
            .add_comment_to_issue(*issue_number, &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })?;

        self.github
            .replace_issue_labels(*issue_number, &["kyc requested".to_string()])
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding labels to issue {} /// {}",
                    issue_number, e
                ))
            })?;
        Ok(())
    }

    pub async fn trigger_ssa(
        id: &str,
        owner: &str,
        repo: &str,
        verifier: &str,
        info: TriggerSSAInfo,
    ) -> Result<(), LDNError> {
        let app_model = Self::get_application_model(id.into(), owner.into(), repo.into()).await?;

        let app_str = app_model.application.ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an application field",
                id
            ))
        })?;
        let application_file = serde_json::from_str::<ApplicationFile>(&app_str).unwrap();

        if application_file.lifecycle.state != AppState::Granted {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected Granted",
                application_file.lifecycle.state
            )));
        }
        let last_allocation = application_file
            .get_last_request_allowance()
            .ok_or(LDNError::Load("Last allocation not found".into()))?;
        if last_allocation.is_active {
            return Err(LDNError::Load("Last active allocation ID is active".into()));
        }

        let requested_so_far = application_file.allocation.total_requested();
        let total_requested = parse_size_to_bytes(&application_file.datacap.total_requested_amount)
            .ok_or(LDNError::Load(
                "Can not parse total requested amount to bytes".into(),
            ))?;
        let ssa_amount =
            parse_size_to_bytes((format!("{}{}", &info.amount, &info.amount_type)).as_str())
                .ok_or(LDNError::Load(
                    "Can not parse requested amount to bytes".into(),
                ))?;
        if requested_so_far + ssa_amount > total_requested {
            return Err(LDNError::Load("The sum of datacap requested so far and requested amount exceeds total requested amount".into()));
        }
        let refill_info = RefillInfo {
            id: id.into(),
            amount: info.amount,
            amount_type: info.amount_type,
            owner: app_model.owner,
            repo: app_model.repo,
        };
        Self::refill(verifier, refill_info).await?;
        Ok(())
    }

    pub async fn submit_kyc(self, info: &SubmitKYCInfo) -> Result<(), LDNError> {
        let client_id = &info.message.client_id;
        let repo = &info.message.allocator_repo_name;
        let owner = &info.message.allocator_repo_owner;
        let app_model =
            Self::get_application_model(client_id.clone(), owner.clone(), repo.clone()).await?;

        let app_str = app_model.application.ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an application field",
                client_id
            ))
        })?;
        let application_file = serde_json::from_str::<ApplicationFile>(&app_str).unwrap();

        if application_file.lifecycle.state.clone() != AppState::KYCRequested {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected RequestKYC",
                application_file.lifecycle.state
            )));
        }

        let address_from_signature =
            LDNApplication::verify_kyc_data_and_get_eth_address(&info.message, &info.signature)?;

        let score = verify_on_gitcoin(&address_from_signature).await?;
        let application_file = application_file.move_back_to_submit_state();
        database::applications::update_application(
            client_id.clone(),
            owner.clone(),
            repo.clone(),
            app_model.pr_number.try_into().map_err(|e| {
                LDNError::Load(format!(
                    "Parse PR number: {} to u64 failed  /// {}",
                    app_model.pr_number, e
                ))
            })?,
            serde_json::to_string_pretty(&application_file).unwrap(),
            app_model.path.clone(),
            None,
            application_file.client_contract_address.clone(),
        )
        .await
        .expect("Failed to update_application in DB!");

        self.issue_updates_for_kyc_submit(
            &application_file.issue_number.parse().unwrap(),
            &score,
            &address_from_signature,
        )
        .await?;

        self.update_and_commit_application_state(
            application_file.clone(),
            owner.clone(),
            repo.clone(),
            app_model.sha.unwrap(),
            LDNPullRequest::application_branch_name(&application_file.id),
            app_model.path.unwrap(),
            "KYC submitted".to_string(),
        )
        .await?;
        Ok(())
    }

    async fn issue_updates_for_kyc_submit(
        &self,
        issue_number: &u64,
        score: &f64,
        eth_address: &Address,
    ) -> Result<(), LDNError> {
        let comment = format!(
            "KYC completed for client address `{}` with Optimism address `{}` and passport score `{}`.", &self.application_id, eth_address, score.round() as i64
        );

        self.github
            .add_comment_to_issue(*issue_number, &comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })?;

        self.github
            .replace_issue_labels(
                *issue_number,
                &[
                    AppState::Submitted.as_str().into(),
                    "waiting for allocator review".into(),
                ],
            )
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding labels to issue {} /// {}",
                    issue_number, e
                ))
            })?;
        Ok(())
    }

    fn date_is_expired(
        expiration_date: &str,
        current_timestamp: &DateTime<Local>,
    ) -> Result<bool, LDNError> {
        let expiration_date_to_datetime = DateTime::parse_from_rfc3339(expiration_date)
            .map_err(|e| LDNError::New(format!("Parse &str to DateTime failed: {e:?}")))?;
        Ok(current_timestamp > &expiration_date_to_datetime)
    }

    fn date_is_from_future(
        issued_date: &str,
        current_timestamp: &DateTime<Local>,
    ) -> Result<bool, LDNError> {
        let issued_date_to_datetime = DateTime::parse_from_rfc3339(issued_date)
            .map_err(|e| LDNError::New(format!("Parse &str to DateTime failed: {e:?}")))?;
        Ok(current_timestamp < &issued_date_to_datetime)
    }

    fn verify_kyc_data_and_get_eth_address<T: ExpirableSolStruct>(
        message: &T,
        signature: &str,
    ) -> Result<Address, LDNError> {
        let address_from_signature = get_address_from_signature(message, signature)?;

        let current_timestamp = Local::now();
        if LDNApplication::date_is_expired(message.get_expires_at(), &current_timestamp)? {
            return Err(LDNError::Load(format!(
                "Message expired at {}",
                message.get_expires_at()
            )));
        }
        if LDNApplication::date_is_from_future(message.get_issued_at(), &current_timestamp)? {
            return Err(LDNError::Load(format!(
                "Message issued date {} is from future",
                message.get_issued_at()
            )));
        }
        Ok(address_from_signature)
    }

    pub async fn remove_pending_allocation(
        &self,
        client_id: &str,
        owner: &str,
        repo: &str,
    ) -> Result<(), LDNError> {
        let app_model =
            Self::get_application_model(client_id.into(), owner.into(), repo.into()).await?;

        let application_file =
            Self::get_application_file_with_active_allocation(&app_model).await?;

        if application_file.lifecycle.state != AppState::ReadyToSign {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected ReadyToSign",
                application_file.lifecycle.state
            )));
        }
        let is_first = application_file.get_active_allocation_request_type()? == "First";
        if is_first {
            self.remove_first_pending_allocation(&application_file)
                .await?;
        } else {
            self.remove_pending_refill(&app_model.pr_number).await?;
        }

        let comment = format!(
            "Last pending allocation reverted for an application `{}`.",
            &self.application_id
        );

        let app_state = if is_first {
            AppState::Submitted.as_str()
        } else {
            AppState::Granted.as_str()
        };

        self.issue_updates(&application_file.issue_number, &comment, app_state)
            .await?;
        Ok(())
    }

    pub async fn revert_to_ready_to_sign(
        &self,
        client_id: &str,
        owner: &str,
        repo: &str,
    ) -> Result<(), LDNError> {
        let app_model =
            Self::get_application_model(client_id.into(), owner.into(), repo.into()).await?;

        let application_file =
            Self::get_application_file_with_active_allocation(&app_model).await?;

        if application_file.lifecycle.state != AppState::StartSignDatacap {
            return Err(LDNError::Load(format!(
                "Application state is {:?}. Expected StartSignDatacap",
                application_file.lifecycle.state
            )));
        }

        self.remove_signers_from_active_request(&application_file)
            .await?;

        let comment = format!(
            "Allocation transaction failed on chain, application {:?} reverted to ReadyToSign state. Please try again.",
            &self.application_id
        );
        self.issue_updates(
            &application_file.issue_number,
            &comment,
            AppState::ReadyToSign.as_str(),
        )
        .await?;
        Ok(())
    }

    async fn remove_first_pending_allocation(
        &self,
        application_file: &ApplicationFile,
    ) -> Result<(), LDNError> {
        let updated_application = application_file.move_back_to_governance_review();
        self.update_and_commit_application_state(
            updated_application.clone(),
            self.github.owner.clone(),
            self.github.repo.clone(),
            self.file_sha.clone(),
            LDNPullRequest::application_branch_name(&application_file.id),
            self.file_name.clone(),
            "Revert last pending allocation".to_string(),
        )
        .await?;
        Ok(())
    }

    async fn remove_pending_refill(&self, pr_number: &i64) -> Result<(), LDNError> {
        Self::delete_branch(
            self.github.owner.clone(),
            self.github.repo.clone(),
            self.branch_name.clone(),
        )
        .await?;
        database::applications::delete_application(
            self.application_id.clone(),
            self.github.owner.clone(),
            self.github.repo.clone(),
            *pr_number as u64,
        )
        .await
        .map_err(|e| {
            LDNError::New(format!(
                "Removing application with PR number: {} failed. {:?}",
                pr_number, e
            ))
        })?;
        Ok(())
    }

    async fn remove_signers_from_active_request(
        &self,
        application_file: &ApplicationFile,
    ) -> Result<(), LDNError> {
        let updated_application = application_file.clone().move_back_to_ready_to_sign();
        self.update_and_commit_application_state(
            updated_application.clone(),
            self.github.owner.clone(),
            self.github.repo.clone(),
            self.file_sha.clone(),
            self.branch_name.clone(),
            self.file_name.clone(),
            "Revert pending allocation to ReadyToSign".to_string(),
        )
        .await?;
        Ok(())
    }

    async fn issue_updates(
        &self,
        issue_number: &str,
        comment: &str,
        label: &str,
    ) -> Result<(), LDNError> {
        let issue_number = issue_number.parse::<u64>().map_err(|e| {
            LDNError::New(format!(
                "Parse issue number: {} to u64 failed. {}",
                issue_number, e
            ))
        })?;
        self.github
            .add_comment_to_issue(issue_number, comment)
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ))
            })?;
        self.github
            .replace_issue_labels(issue_number, &[label.into()])
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Error adding labels to issue {} /// {}",
                    issue_number, e
                ))
            })?;
        Ok(())
    }

    async fn get_application_file_with_active_allocation(
        app_model: &ApplicationModel,
    ) -> Result<ApplicationFile, LDNError> {
        if app_model.pr_number == 0 {
            return Err(LDNError::Load("Active pull request not found".to_string()));
        }

        let app_str = app_model.application.as_ref().ok_or_else(|| {
            LDNError::New(format!(
                "Application {} does not have an application field",
                app_model.id
            ))
        })?;

        let application_file = serde_json::from_str::<ApplicationFile>(app_str).map_err(|e| {
            LDNError::New(format!("Failed to parse string to ApplicationFile: {}", e))
        })?;

        application_file.get_active_allocation().ok_or_else(|| {
            LDNError::Load(format!(
                "Application {} does not have an active allocation",
                app_model.id
            ))
        })?;

        Ok(application_file)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LDNPullRequest {
    pub branch_name: String,
    pub title: String,
    pub body: String,
    pub path: String,
}

impl LDNPullRequest {
    async fn create_pr_for_new_application(
        issue_number: String,
        owner_name: String,
        app_branch_name: String,
        file_name: String,
        file_content: String,
        owner: String,
        repo: String,
    ) -> Result<String, LDNError> {
        let initial_commit = Self::application_initial_commit(&owner_name, &issue_number);
        let gh: GithubWrapper = github_async_new(owner.to_string(), repo.to_string()).await;
        let head_hash = gh.get_main_branch_sha().await.unwrap();
        let create_ref_request = gh
            .build_create_ref_request(app_branch_name.clone(), head_hash)
            .map_err(|e| {
                LDNError::New(format!(
                    "Application issue {} cannot create branch /// {}",
                    issue_number, e
                ))
            })?;

        let issue_link = format!(
            "https://github.com/{}/{}/issues/{}",
            owner, repo, issue_number
        );

        let (_pr, file_sha) = gh
            .create_merge_request(CreateMergeRequestData {
                issue_link,
                branch_name: app_branch_name,
                file_name,
                owner_name,
                ref_request: create_ref_request,
                file_content,
                commit: initial_commit,
            })
            .await
            .map_err(|e| {
                LDNError::New(format!(
                    "Application issue {} cannot create merge request /// {}",
                    issue_number, e
                ))
            })?;

        Ok(file_sha)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_pr_for_existing_application(
        application_id: String,
        file_content: String,
        file_name: String,
        branch_name: String,
        file_sha: String,
        owner: String,
        repo: String,
        should_create_in_db: bool,
        issue_number: String,
        pr_title: String,
    ) -> Result<u64, LDNError> {
        let gh = github_async_new(owner.to_string(), repo.to_string()).await;
        let head_hash = gh.get_main_branch_sha().await.unwrap();
        let create_ref_request = gh
            .build_create_ref_request(branch_name.clone(), head_hash)
            .map_err(|e| {
                LDNError::New(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                ))
            })?;

        let issue_link = format!(
            "https://github.com/{}/{}/issues/{}",
            owner, repo, issue_number
        );

        let pr = match gh
            .create_refill_merge_request(CreateRefillMergeRequestData {
                issue_link,
                file_name: file_name.clone(),
                file_sha: file_sha.clone(),
                ref_request: create_ref_request,
                branch_name,
                file_content: file_content.clone(),
                commit: pr_title,
            })
            .await
        {
            Ok(pr) => {
                if should_create_in_db {
                    let issue_number = issue_number.parse::<i64>().map_err(|e| {
                        LDNError::New(format!("Parse issue number to i64 failed: {}", e))
                    })?;
                    database::applications::create_application(
                        application_id.clone(),
                        owner,
                        repo,
                        pr.0.number,
                        issue_number,
                        file_content,
                        file_name,
                    )
                    .await
                    .map_err(|e| {
                        LDNError::New(format!(
                            "Application issue {} cannot create application in DB /// {}",
                            application_id, e
                        ))
                    })?;
                }
                pr
            }
            Err(e) => {
                return Err(LDNError::New(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                )));
            }
        };
        Ok(pr.0.number)
    }

    pub async fn add_commit_to(
        path: String,
        branch_name: String,
        commit_message: String,
        new_content: String,
        file_sha: String,
        owner: String,
        repo: String,
    ) -> Option<()> {
        let gh = github_async_new(owner.to_string(), repo.to_string()).await;
        match gh
            .update_file_content(
                &path,
                &commit_message,
                &new_content,
                &branch_name,
                &file_sha,
            )
            .await
        {
            Ok(_) => Some(()),
            Err(e) => {
                log::error!("Failed to add commit: {}", e);
                None
            }
        }
    }

    pub async fn close_pull_request(
        owner: String,
        repo: String,
        pr_number: u64,
    ) -> Result<(), LDNError> {
        let gh = github_async_new(owner.clone(), repo.clone()).await;
        gh.close_pull_request(pr_number).await.map_err(|e| {
            LDNError::New(format!(
                "Error closing pull request {} /// {}",
                pr_number, e
            ))
        })?;
        Ok(())
    }

    pub async fn delete_branch(
        application_id: String,
        owner: String,
        repo: String,
    ) -> Result<(), LDNError> {
        let gh = github_async_new(owner.clone(), repo.clone()).await;
        let branch_name = LDNPullRequest::application_branch_name(&application_id);
        let request = gh.build_remove_ref_request(branch_name.clone()).unwrap();
        gh.remove_branch(request).await.map_err(|e| {
            LDNError::New(format!("Error deleting branch {} /// {}", branch_name, e))
        })?;
        Ok(())
    }

    pub(super) fn application_branch_name(application_id: &str) -> String {
        format!("Application/{}", application_id)
    }

    pub(super) fn application_path(application_id: &str) -> String {
        format!("{}/{}.json", "applications", application_id)
    }

    pub(super) fn application_initial_commit(owner_name: &str, application_id: &str) -> String {
        format!("Start Application: {}-{}", owner_name, application_id)
    }

    pub(super) fn application_move_to_proposal_commit(actor: &str) -> String {
        format!(
            "User {} Moved Application to Proposal State from Allocator Review State",
            actor
        )
    }

    pub(super) fn application_move_to_approval_commit(actor: &str) -> String {
        format!(
            "Notary User {} Moved Application to Approval State from Proposal State",
            actor
        )
    }

    pub(super) fn application_move_to_confirmed_commit(actor: &str) -> String {
        format!(
            "Notary User {} Moved Application to Confirmed State from Proposal Approval",
            actor
        )
    }
}

pub fn get_file_sha(content: &ContentItems) -> Option<String> {
    match content.items.first() {
        Some(item) => {
            let sha = item.sha.clone();
            Some(sha)
        }
        None => None,
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_update_pr_from_isue() {
//         let _ = fplus_database::setup_test_environment().await;
//         let info = CreateApplicationInfo {
//             issue_number: "28".to_string(),
//             owner: "clriesco".to_string(),
//             repo: "king-charles-staging".to_string(),
//         };
//         match LDNApplication::update_from_issue(info).await {
//             Ok(app) => {
//                 log::info!("Application updated: {:?}", app);
//             }
//             Err(e) => {
//                 log::error!("Failed to update application: {}", e);
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_app_state_to_kyc_requested() {
        let application_file = ApplicationFile::new(
            "1".into(),
            "adres".into(),
            application::file::Version::Text("1.3".to_string()),
            "adres2".into(),
            Default::default(),
            Default::default(),
            application::file::Datacap {
                _group: application::file::DatacapGroup::DA,
                data_type: application::file::DataType::Slingshot,
                total_requested_amount: "1 TiB".into(),
                single_size_dataset: "1 GiB".into(),
                replicas: 2,
                weekly_allocation: "1 TiB".into(),
                custom_multisig: "adres".into(),
                identifier: "id".into(),
            },
        )
        .await;
        let application_file = application_file.kyc_request();
        assert_eq!(application_file.lifecycle.state, AppState::KYCRequested);
    }

    #[tokio::test]
    async fn test_update_app_state_to_submitted_after_kyc() {
        let mut application_file = ApplicationFile::new(
            "1".into(),
            "adres".into(),
            application::file::Version::Text("1.3".to_string()),
            "adres2".into(),
            Default::default(),
            Default::default(),
            application::file::Datacap {
                _group: application::file::DatacapGroup::DA,
                data_type: application::file::DataType::Slingshot,
                total_requested_amount: "1 TiB".into(),
                single_size_dataset: "1 GiB".into(),
                replicas: 2,
                weekly_allocation: "1 TiB".into(),
                custom_multisig: "adres".into(),
                identifier: "id".into(),
            },
        )
        .await;
        application_file.lifecycle = application_file.lifecycle.finish_approval();
        assert_eq!(application_file.lifecycle.state, AppState::Granted);
        let application_file = application_file.move_back_to_submit_state();
        assert_eq!(application_file.lifecycle.state, AppState::Submitted);
    }

    #[tokio::test]
    async fn test_date_is_expired() {
        let message: KycApproval = KycApproval {
            message: "Connect your Fil+ application with your wallet and give access to your Gitcoin passport".into(),
            client_id: "test".into(),
            issued_at: "2024-05-28T09:02:51.126Z".into(),
            expires_at: "2024-05-29T09:02:51.126Z".into(),
            allocator_repo_name: "test".into(),
            allocator_repo_owner: "test".into()
        };
        let fixed_current_date = "2024-05-28T09:04:51.126Z";
        let fixed_current_date = DateTime::parse_from_rfc3339(fixed_current_date)
            .map_err(|e| LDNError::New(format!("Parse &str to DateTime failed: {e:?}")));
        let is_expired = LDNApplication::date_is_expired(
            &message.expires_at,
            &fixed_current_date.unwrap().into(),
        );
        assert!(!is_expired.unwrap())
    }

    #[tokio::test]
    async fn test_date_is_from_future() {
        let message: KycApproval = KycApproval {
            message: "Connect your Fil+ application with your wallet and give access to your Gitcoin passport".into(),
            client_id: "test".into(),
            issued_at: "2024-05-28T09:02:51.126Z".into(),
            expires_at: "2024-05-29T09:02:51.126Z".into(),
            allocator_repo_name: "test".into(),
            allocator_repo_owner: "test".into()
        };
        let fixed_current_date = "2024-05-28T09:04:51.126Z";
        let fixed_current_date = DateTime::parse_from_rfc3339(fixed_current_date)
            .map_err(|e| LDNError::New(format!("Parse &str to DateTime failed: {e:?}")));
        let is_from_future = LDNApplication::date_is_from_future(
            &message.issued_at,
            &fixed_current_date.unwrap().into(),
        );
        assert!(!is_from_future.unwrap())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use env_logger::{Builder, Env};
//     use tokio::time::{sleep, Duration};

//     static OWNER: &str = "keyko-io";
//     static REPO: &str = "test-philip-second";

//     #[tokio::test]
//     async fn end_to_end() {
//         // Set logging
//         Builder::from_env(Env::default().default_filter_or("info")).init();
//         log::info!("Starting end-to-end test");

//         // Test Creating an application
//         let _ = fplus_database::setup_test_environment().await;
//         let gh = github_async_new(OWNER.to_string(), REPO.to_string()).await;

//         log::info!("Creating a new LDNApplication from issue");
//         let ldn_application: LDNApplication = match LDNApplication::new_from_issue(CreateApplicationInfo {
//             issue_number: "37".to_string(),
//             owner: OWNER.to_string(),
//             repo: REPO.to_string()
//         })
//         .await
//         {
//             Ok(app) => app,
//             Err(e) => {
//                 log::error!("Failed to create LDNApplication: {}", e);
//                 return;
//             }
//         };

//         let application_id = ldn_application.application_id.to_string();
//         log::info!("LDNApplication created with ID: {}", application_id);

//         // Validate file creation
//         log::info!("Validating file creation for application");
//         if let Err(e) = gh
//             .get_file(&ldn_application.file_name, &ldn_application.branch_name)
//             .await
//         {
//             log::warn!(
//                 "File validation failed for application ID {}: {}",
//                 application_id,
//                 e
//             );
//         }

//         // Validate pull request creation
//         log::info!("Validating pull request creation for application");
//         if let Err(e) = gh
//             .get_pull_request_by_head(&LDNPullRequest::application_branch_name(
//                 application_id.as_str(),
//             ))
//             .await
//         {
//             log::warn!(
//                 "Pull request validation failed for application ID {}: {}",
//                 application_id,
//                 e
//             );
//         }

//         sleep(Duration::from_millis(2000)).await;

//         // Test Triggering an application
//         log::info!("Loading application for triggering");
//         let ldn_application_before_trigger =
//             match LDNApplication::load(application_id.clone(), OWNER.to_string(), REPO.to_string()).await {
//                 Ok(app) => app,
//                 Err(e) => {
//                     log::error!("Failed to load application for triggering: {}", e);
//                     return;
//                 }
//             };

//         log::info!("Completing allocator review");
//         if let Err(e) = ldn_application_before_trigger
//             .complete_governance_review(
//                 "actor_address".to_string(),
//                 OWNER.to_string(),
//                 REPO.to_string())
//             .await
//         {
//             log::error!("Failed to complete allocator review: {}", e);
//             return;
//         }

//         let ldn_application_after_trigger = match LDNApplication::load(
//             application_id.clone(),
//             OWNER.to_string(),
//             REPO.to_string()
//         ).await
//         {
//             Ok(app) => app,
//             Err(e) => {
//                 log::error!("Failed to load application after triggering: {}", e);
//                 return;
//             }
//         };

//         assert_eq!(
//             ldn_application_after_trigger.app_state().await.unwrap(),
//             AppState::ReadyToSign
//         );
//         log::info!("Application state updated to ReadyToSign");
//         sleep(Duration::from_millis(2000)).await;

//         // Cleanup
//         log::info!("Starting cleanup process");
//         let head = &LDNPullRequest::application_branch_name(&application_id);
//         match gh.get_pull_request_by_head(head).await {
//             Ok(prs) => {
//                 if let Some(pr) = prs.get(0) {
//                     let number = pr.number;
//                     match gh.merge_pull_request(number).await {
//                         Ok(_) => log::info!("Merged pull request {}", number),
//                         Err(_) => log::info!("Pull request {} was already merged", number),
//                     };
//                 }
//             }
//             Err(e) => log::warn!("Failed to get pull request by head: {}", e),
//         };

//         sleep(Duration::from_millis(3000)).await;

//         let file = match gh.get_file(&ldn_application.file_name, "main").await {
//             Ok(f) => f,
//             Err(e) => {
//                 log::error!("Failed to get file: {}", e);
//                 return;
//             }
//         };

//         let file_sha = file.items[0].sha.clone();
//         let remove_file_request = gh
//             .delete_file(&ldn_application.file_name, "main", "remove file", &file_sha)
//             .await;
//         let remove_branch_request = gh
//             .build_remove_ref_request(LDNPullRequest::application_branch_name(&application_id))
//             .unwrap();

//         if let Err(e) = gh.remove_branch(remove_branch_request).await {
//             log::warn!("Failed to remove branch: {}", e);
//         }
//         if let Err(e) = remove_file_request {
//             log::warn!("Failed to remove file: {}", e);
//         }

//         log::info!(
//             "End-to-end test completed for application ID: {}",
//             application_id
//         );
//     }
// }
