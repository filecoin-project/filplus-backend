use std::str::FromStr;

use futures::future;
use octocrab::models::{
    pulls::PullRequest,
    repos::{Content, ContentItems},
};
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    base64::{self},
    config::get_env_var_or_default,
    error::LDNError,
    external_services::github::{
        CreateMergeRequestData, CreateRefillMergeRequestData, GithubWrapper,
    },
    parsers::ParsedIssue,
};
use fplus_database::database;

use self::application::file::{
    AllocationRequest, AllocationRequestType, AppState, ApplicationFile, ValidVerifierList, VerifierInput
};
use rayon::prelude::*;
use crate::core::application::file::Allocation;

pub mod application;
pub mod allocator;

#[derive(Deserialize)]
pub struct CreateApplicationInfo {
    pub issue_number: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VerifierList(pub Vec<String>);

#[derive(Deserialize, Serialize, Debug)]
pub struct CompleteNewApplicationProposalInfo {
    pub signer: VerifierInput,
    pub request_id: String,
    pub owner: String,
    pub repo: String
}

#[derive(Debug)]
pub struct LDNApplication {
    github: GithubWrapper,
    pub application_id: String,
    pub file_sha: String,
    pub file_name: String,
    pub branch_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteGovernanceReviewInfo {
    pub actor: String,
    pub owner: String, 
    pub repo: String,
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
pub struct DcReachedInfo {
    pub owner: String,
    pub repo: String
}

#[derive(Deserialize)]
pub struct ValidationPullRequestData {
    pub pr_number: String,
    pub user_handle: String,
    pub owner: String,
    pub repo: String
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
#[derive(Deserialize)]
pub struct ChangedAllocator {
    pub file_changed: String
}

#[derive(Deserialize)]
pub struct AllocatorUpdateInfo {
    pub installation_id: Option<i64>,
    pub multisig_address: Option<String>,
    pub verifiers_gh_handles: Option<String>,
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

impl LDNApplication {

    pub async fn single_active(pr_number: u64, owner: String, repo: String) -> Result<ApplicationFile, LDNError> {
        let gh: GithubWrapper = GithubWrapper::new(owner, repo);
        let (_, pull_request) = gh.get_pull_request_files(pr_number).await.unwrap();
        let pull_request = pull_request.get(0).unwrap();
        let pull_request: Response = reqwest::Client::new()
            .get(&pull_request.raw_url.to_string())
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

    async fn load_pr_files(
        pr: PullRequest,
        owner: String,
        repo: String
    ) -> Result<Option<(String, String, ApplicationFile, PullRequest)>, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        let files = match gh.get_pull_request_files(pr.number).await {
            Ok(files) => files,
            Err(_) => return Ok(None),
        };
        let raw_url = match files.1.get(0) {
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
        Ok(Some((
            files.1.get(0).unwrap().sha.clone(),
            files.1.get(0).unwrap().filename.clone(),
            app,
            pr.clone(),
        )))
    }

    pub async fn load(application_id: String, owner: String, repo: String) -> Result<Self, LDNError> {
        let gh: GithubWrapper = GithubWrapper::new(owner.clone(), repo.clone());
        let pull_requests = gh.list_pull_requests().await.unwrap();
        let pull_requests = future::try_join_all(
            pull_requests
                .into_iter()
                .map(|pr: PullRequest| (LDNApplication::load_pr_files(pr, owner.clone(), repo.clone())))
                .collect::<Vec<_>>(),
        )
        .await?;
        let result = pull_requests
            .par_iter()
            .filter(|pr| {
                if let Some(r) = pr {
                    if String::from(r.2.id.clone()) == application_id.clone() {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            })
            .collect::<Vec<_>>();
        if let Some(r) = result.get(0) {
            if let Some(r) = r {
                return Ok(Self {
                    github: gh,
                    application_id: r.2.id.clone(),
                    file_sha: r.0.clone(),
                    file_name: r.1.clone(),
                    branch_name: r.3.head.ref_field.clone(),
                });
            }
        }

        let app = Self::single_merged(application_id, owner.clone(), repo.clone()).await?;
        return Ok(Self {
            github: gh,
            application_id: app.1.id.clone(),
            file_sha: app.0.sha.clone(),
            file_name: app.0.path.clone(),
            branch_name: "main".to_string(),
        });
    }

    pub async fn active(owner: String, repo: String, filter: Option<String>) -> Result<Vec<ApplicationFile>, LDNError> {
        let gh: GithubWrapper = GithubWrapper::new(owner.clone(), repo.clone());
        let mut apps: Vec<ApplicationFile> = Vec::new();
        let pull_requests = gh.list_pull_requests().await.unwrap();
        let pull_requests = future::try_join_all(
            pull_requests
                .into_iter()
                .map(|pr: PullRequest| LDNApplication::load_pr_files(pr, owner.clone(), repo.clone()))
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
        for r in pull_requests {
            if r.is_some() {
                let r = r.unwrap();
                if filter.is_none() {
                    apps.push(r.2)
                } else {
                    if r.2.id == filter.clone().unwrap() {
                        apps.push(r.2)
                    }
                }
            }
        }
        Ok(apps)
    }

    /// Create New Application
    pub async fn new_from_issue(info: CreateApplicationInfo) -> Result<Self, LDNError> {
        let issue_number = info.issue_number;
        let gh: GithubWrapper = GithubWrapper::new(info.owner.clone(), info.repo.clone());
        let (parsed_ldn, _) = LDNApplication::parse_application_issue(
            issue_number.clone(), 
            info.owner.clone(), 
            info.repo.clone()
        ).await?;
        let application_id = parsed_ldn.id.clone();
        let file_name = LDNPullRequest::application_path(&application_id);
        let branch_name = LDNPullRequest::application_branch_name(&application_id);

        let multisig_address = if parsed_ldn.datacap.custom_multisig == "[X] Use Custom Multisig" {
            "true".to_string()
        } else {
            "false".to_string()
        };

        match gh.get_file(&file_name, &branch_name).await {
            Err(_) => {
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
                let file_content = match serde_json::to_string_pretty(&application_file) {
                    Ok(f) => f,
                    Err(e) => {
                        Self::add_error_label(
                            application_file.issue_number.clone(), 
                            "".to_string(),
                            info.owner.clone(),
                            info.repo.clone()
                        ).await?;
                        return Err(LDNError::New(format!(
                            "Application issue file is corrupted /// {}",
                            e
                        )))
                    }
                };
                let app_id = parsed_ldn.id.clone();
                let file_sha = LDNPullRequest::create_pr(
                    issue_number.clone(),
                    parsed_ldn.client.name.clone(),
                    branch_name.clone(),
                    LDNPullRequest::application_path(&app_id),
                    file_content.clone(),
                    info.owner.clone(), 
                    info.repo.clone()
                )
                .await?;
                Self::issue_waiting_for_gov_review(issue_number.clone(), info.owner.clone(), info.repo.clone()).await?;
                Self::update_issue_labels(
                    application_file.issue_number.clone(), 
                    &[AppState::Submitted.as_str(), "waiting for governance review"],
                    info.owner.clone(), 
                    info.repo.clone()
                ).await?;
                Ok(LDNApplication {
                    github: gh,
                    application_id,
                    file_sha,
                    file_name,
                    branch_name,
                })
            }
            Ok(_) => {
                return Err(LDNError::New(format!(
                    "Application issue {} already exists",
                    application_id
                )))
            }
        }
    }

    /// Move application from Governance Review to Proposal
    pub async fn complete_governance_review(
        &self,
        actor: String,
        owner: String,
        repo: String,
    ) -> Result<ApplicationFile, LDNError> {
        match self.app_state().await {
            Ok(s) => match s {
                AppState::Submitted => {
                    let app_file: ApplicationFile = self.file().await?;
                    let uuid = uuidv4::uuid::v4();
                    let request = AllocationRequest::new(
                        actor.clone(),
                        uuid,
                        AllocationRequestType::First,
                        app_file.datacap.weekly_allocation.clone(),
                    );
                    let app_file = app_file.complete_governance_review(actor.clone(), request);
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    let app_path = &self.file_name.clone();
                    let app_branch = self.branch_name.clone();
                    Self::issue_datacap_request_trigger(
                        app_file.clone(),
                        owner.clone(),
                        repo.clone()
                    ).await?;
                    match LDNPullRequest::add_commit_to(
                        app_path.to_string(),
                        app_branch,
                        LDNPullRequest::application_move_to_proposal_commit(&actor),
                        file_content,
                        self.file_sha.clone(),
                        owner, 
                        repo
                    )
                    .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNError::New(format!(
                                "Application issue {} cannot be triggered(1)",
                                self.application_id
                            )))
                        }
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
        repo: String
    ) -> Result<ApplicationFile, LDNError> {
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
                    let app_file = app_file.add_signer_to_allocation(
                        signer.clone().into(),
                        request_id,
                        app_lifecycle,
                    );
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    match LDNPullRequest::add_commit_to(
                        self.file_name.to_string(),
                        self.branch_name.clone(),
                        LDNPullRequest::application_move_to_approval_commit(
                            &signer.signing_address,
                        ),
                        file_content,
                        self.file_sha.clone(),
                        owner,
                        repo
                    )
                    .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNError::New(format!(
                                "Application issue {} cannot be proposed(1)",
                                self.application_id
                            )))
                        }
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

    pub async fn complete_new_application_approval(
        &self,
        signer: VerifierInput,
        request_id: String,
        owner: String,
        repo: String
    ) -> Result<ApplicationFile, LDNError> {
        match self.app_state().await {
            Ok(s) => match s {
                AppState::StartSignDatacap => {
                    let app_file: ApplicationFile = self.file().await?;
                    let app_lifecycle = app_file.lifecycle.finish_approval();
                    let app_file = app_file.add_signer_to_allocation_and_complete(
                        signer.clone().into(),
                        request_id,
                        app_lifecycle,
                    );
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    match LDNPullRequest::add_commit_to(
                        self.file_name.to_string(),
                        self.branch_name.clone(),
                        LDNPullRequest::application_move_to_confirmed_commit(
                            &signer.signing_address,
                        ),
                        file_content,
                        self.file_sha.clone(),
                        owner,
                        repo
                    )
                    .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNError::New(format!(
                                "Application issue {} cannot be proposed(1)",
                                self.application_id
                            )))
                        }
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

    async fn parse_application_issue(
        issue_number: String,
        owner: String,
        repo: String,
    ) -> Result<(ParsedIssue, String), LDNError> {
        let gh: GithubWrapper = GithubWrapper::new(owner, repo);
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

    /// Return Application state
    async fn app_state(&self) -> Result<AppState, LDNError> {
        let f = self.file().await?;
        Ok(f.lifecycle.get_state())
    }

    /// Return Application state
    pub async fn total_dc_reached(application_id: String, owner: String, repo: String) -> Result<bool, LDNError> {
        let merged = Self::merged(owner.clone(), repo.clone()).await?;
        let app = merged
            .par_iter()
            .find_first(|(_, app)| app.id == application_id);
        if app.is_some() && app.unwrap().1.lifecycle.get_state() == AppState::Granted {
            let app = app.unwrap().1.reached_total_datacap();
            let gh: GithubWrapper = GithubWrapper::new(owner.clone(), repo.clone());
            let ldn_app = LDNApplication::load(application_id.clone(), owner.clone(), repo.clone()).await?;
            let ContentItems { items } = gh.get_file(&ldn_app.file_name, "main").await.unwrap();
            Self::issue_full_dc(app.issue_number.clone(), owner.clone(), repo.clone()).await?;
            Self::update_issue_labels(
                app.issue_number.clone(), 
                &[AppState::TotalDatacapReached.as_str()],
                owner.clone(),
                repo.clone()
            ).await?;
            LDNPullRequest::create_refill_pr(
                app.id.clone(),
                app.client.name.clone(),
                serde_json::to_string_pretty(&app).unwrap(),
                ldn_app.file_name.clone(),
                format!("{}-total-dc-reached", app.id),
                items[0].sha.clone(),
                owner,
                repo
            )
            .await?;
            Ok(true)
        } else {
            return Err(LDNError::Load(format!(
                "Application issue {} does not exist",
                application_id
            )));
        }
    }

    fn content_items_to_app_file(file: ContentItems) -> Result<ApplicationFile, LDNError> {
        let f = &file
            .clone()
            .take_items()
            .get(0)
            .and_then(|f| f.content.clone())
            .and_then(|f| base64::decode_application_file(&f.replace("\n", "")))
            .ok_or(LDNError::Load(format!("Application file is corrupted",)))?;
        return Ok(ApplicationFile::from(f.clone()));
    }

    pub async fn file(&self) -> Result<ApplicationFile, LDNError> {
        match self
            .github
            .get_file(&self.file_name, &self.branch_name)
            .await
        {
            Ok(file) => {
                return Ok(LDNApplication::content_items_to_app_file(file)?);
            }
            Err(e) => {
                dbg!(&e);
                return Err(LDNError::Load(format!(
                    "Application issue {} file does not exist ///",
                    self.application_id
                )));
            }
        }
    }

    pub async fn fetch_verifiers(owner: String, repo: String) -> Result<ValidVerifierList, LDNError> {
        let allocator = database::get_allocator(&owner, &repo).await.map_err(|e| LDNError::Load(format!("Failed to retrieve allocators /// {}", e)))?;
    
        let mut verifiers_handles = Vec::new();

        let allocator = match allocator {
            Some(a) => a,
            None => return Err(LDNError::Load("No allocator found".into())),
        };

        if let Some(handles) = allocator.verifiers_gh_handles {
            verifiers_handles.extend(handles.split(',').map(|s| s.trim().to_string()));
        }
    
        if verifiers_handles.is_empty() {
            return Err(LDNError::Load("No governance team found".into()));
        }
    
        Ok(ValidVerifierList { verifiers: verifiers_handles })
    }

    async fn single_merged(application_id: String, owner: String, repo: String) -> Result<(Content, ApplicationFile), LDNError> {
        Ok(LDNApplication::merged(owner, repo)
            .await?
            .into_iter()
            .find(|(_, app)| app.id == application_id)
            .map_or_else(
                || {
                    return Err(LDNError::Load(format!(
                        "Application issue {} does not exist",
                        application_id
                    )));
                },
                |app| Ok(app),
            )?)
    }

    async fn map_merged(item: Content) -> Result<Option<(Content, ApplicationFile)>, LDNError> {
        if item.download_url.is_none() {
            return Ok(None);
        }
        let file = reqwest::Client::new()
            .get(&item.download_url.clone().unwrap())
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

    pub async fn merged(owner: String, repo: String) -> Result<Vec<(Content, ApplicationFile)>, LDNError> {
        let gh = GithubWrapper::new(owner.clone(), repo.clone());
        let applications_path = "applications";
        let mut all_files = gh.get_files(applications_path).await.map_err(|e| {
            LDNError::Load(format!(
                "Failed to retrieve all files from GitHub. Reason: {}",
                e
            ))
        })?;
        all_files
            .items
            .retain(|item| item.download_url.is_some() && item.name.ends_with(".json"));
        let all_files = future::try_join_all(
            all_files
                .items
                .into_iter()
                .map(|fd| LDNApplication::map_merged(fd))
                .collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| {
            LDNError::Load(format!(
                "Failed to fetch application files from their URLs. Reason: {}",
                e
            ))
        })?;

        let mut apps: Vec<(Content, ApplicationFile)> = vec![];
        let active: Vec<ApplicationFile> = Self::active(owner.clone(), repo.clone(), None).await?;
        for app in all_files {
            if app.is_some() {
                let app = app.unwrap();
                if active.iter().find(|a| a.id == app.1.id).is_none() && app.1.lifecycle.is_active {
                    apps.push(app);
                }
            }
        }
        Ok(apps)
    }

    pub async fn refill(refill_info: RefillInfo) -> Result<bool, LDNError> {
        let apps = LDNApplication::merged(refill_info.owner.clone(), refill_info.repo.clone()).await?;
        if let Some((content, mut app)) = apps.into_iter().find(|(_, app)| app.id == refill_info.id)
        {
            let uuid = uuidv4::uuid::v4();
            let request_id = uuid.clone();
            let new_request = AllocationRequest::new(
                "SSA Bot".to_string(),
                request_id.clone(),
                AllocationRequestType::Refill(0),
                format!("{}{}", refill_info.amount, refill_info.amount_type),
            );
            let app_file = app.start_refill_request(new_request);
            Self::issue_refill(app.issue_number.clone(), refill_info.owner.clone(), refill_info.repo.clone()).await?;
            LDNPullRequest::create_refill_pr(
                app.id.clone(),
                app.client.name.clone(),
                serde_json::to_string_pretty(&app_file).unwrap(),
                content.path.clone(), // filename
                request_id.clone(),
                content.sha,
                refill_info.owner,
                refill_info.repo
            )
            .await?;
            return Ok(true);
        }
        Err(LDNError::Load("Failed to get application file".to_string()))
    }

    pub async fn validate_flow(pr_number: u64, actor: &str, owner: String, repo: String) -> Result<bool, LDNError> {
        log::info!("Starting validate_flow:");
        log::info!(
            "- Validating flow for PR number {} with user handle {}",
            pr_number,
            actor
        );

        let gh = GithubWrapper::new(owner, repo);
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
        if application.lifecycle.get_state() == AppState::Submitted {
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
        let bot_user = get_env_var_or_default("BOT_USER");
        if author != bot_user {
            log::warn!("- Author is not the bot user");
            return Ok(false);
        }

        log::info!("- Application is in a valid state");
        return Ok(true);
    }

    pub async fn validate_trigger(pr_number: u64, actor: &str, owner: String, repo: String) -> Result<bool, LDNError> {
        log::info!("Starting validate_trigger:");
        log::info!(
            "- Validating trigger for PR number {} with user handle {}",
            pr_number,
            actor
        );

        if let Ok(application_file) = LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await {
            if application_file.lifecycle.get_active_status() == false {
                log::info!("No trigger to validate. Application lifecycle is inactive so the Total DC was reached.");
                return Ok(true);
            }
            let validated_by = application_file.lifecycle.validated_by.clone();
            let validated_at = application_file.lifecycle.validated_at.clone();
            let app_state = application_file.lifecycle.get_state();
            let active_request_id = application_file.lifecycle.active_request.clone();
            let valid_verifier_list = Self::fetch_verifiers(owner.clone(), repo.clone()).await?;
            let bot_user = get_env_var_or_default("BOT_USER");

            let res: bool = match app_state {
                AppState::Submitted => {
                    log::warn!("- Application state is Submitted");
                    return Ok(false);
                }
                AppState::ReadyToSign => {
                    if application_file.allocation.0.is_empty() {
                        log::warn!("- No allocations found");
                        false
                    } else {
                        let active_allocation = application_file
                            .allocation
                            .0
                            .iter()
                            .find(|obj| Some(&obj.id) == active_request_id.as_ref());

                        if active_allocation.is_none() {
                            log::warn!("- Active allocation not found");
                            false
                        } else if active_allocation.unwrap().signers.0.len() > 0 {
                            log::warn!("- Active allocation has signers");
                            false
                        } else if validated_at.is_empty() {
                            log::warn!("- Not ready to sign - validated_at is empty");
                            false
                        } else if validated_by.is_empty() {
                            log::warn!("- Not ready to sign - validated_by is empty");
                            false
                        } else if actor != bot_user {
                            log::warn!("- Not ready to sign - actor is not the bot user");
                            false
                        } else if !valid_verifier_list.is_valid(&validated_by) {
                            log::warn!("- Not ready to sign - valid_verifier_list is not valid");
                            false
                        } else {
                            log::info!("- Validated!");
                            Self::issue_datacap_allocation_requested(
                                application_file.clone(), 
                                active_allocation.clone(),
                                owner.clone(), 
                                repo.clone()
                            ).await?;
                            Self::update_issue_labels(
                                application_file.issue_number.clone(), 
                                &[AppState::ReadyToSign.as_str()],
                                owner.clone(), 
                                repo.clone()
                            ).await?;
                            Self::issue_ready_to_sign(
                                application_file.issue_number.clone(),
                                owner.clone(), 
                                repo.clone()
                            ).await?;
                            true
                        }
                    }
                }
                AppState::StartSignDatacap => {
                    if !validated_at.is_empty()
                        && !validated_by.is_empty()
                        && valid_verifier_list.is_valid(&validated_by)
                    {
                        log::info!("- Validated!");
                        true
                    } else {
                        if validated_at.is_empty() {
                            log::warn!("- AppState: StartSignDatacap, validation failed: validated_at is empty");
                        }
                        if validated_by.is_empty() {
                            log::warn!("- AppState: StartSignDatacap, validation failed: validated_by is empty");
                        }
                        if !valid_verifier_list.is_valid(&validated_by) {
                            log::warn!("- AppState: StartSignDatacap, validation failed: valid_verifier_list is not valid");
                        }
                        false
                    }
                }
                AppState::Granted => {
                    if !validated_at.is_empty()
                        && !validated_by.is_empty()
                        && valid_verifier_list.is_valid(&validated_by)
                    {
                        log::info!("- Application is granted");
                        true
                    } else {
                        if validated_at.is_empty() {
                            log::warn!(
                                "- AppState: Granted, validation failed: validated_at is empty"
                            );
                        }
                        if validated_by.is_empty() {
                            log::warn!(
                                "- AppState: Granted, validation failed: validated_by is empty"
                            );
                        }
                        if !valid_verifier_list.is_valid(&validated_by) {
                            log::warn!(
                                "- AppState: Granted, validation failed: valid_verifier_list is not valid"
                            );
                        }
                        false
                    }
                }
                AppState::TotalDatacapReached => {
                    log::info!("- Application state is TotalDatacapReached");
                    true
                }
                AppState::Error => {
                    log::warn!("- Application state is Error");
                    return Ok(false);
                }
            };

            if res {
                log::info!("Validated!");
                return Ok(true);
            }

            let app_file = application_file.move_back_to_governance_review();
            let ldn_application = LDNApplication::load(app_file.id.clone(), owner.clone(), repo.clone()).await?;

            match LDNPullRequest::add_commit_to(
                ldn_application.file_name,
                ldn_application.branch_name.clone(),
                format!("Move application back to governance review"),
                serde_json::to_string_pretty(&app_file).unwrap(),
                ldn_application.file_sha.clone(), 
                owner, 
                repo
            )
            .await
            {
                Some(()) => {}
                None => {}
            };

            return Ok(false);
        };

        log::info!("Failed to fetch Application File");
        Ok(false)
    }

    pub async fn validate_approval(pr_number: u64, owner: String, repo: String) -> Result<bool, LDNError> {
        log::info!("Starting validate_approval:");
        log::info!("Validating approval for PR number {}", pr_number);
        match LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await {
            Ok(application_file) => {
                if application_file.lifecycle.get_active_status() == false {
                    log::info!("No approval to validate. Application lifecycle is inactive so the Total DC was reached.");
                    return Ok(true);
                }
                let app_state: AppState = application_file.lifecycle.get_state();

                log::info!("- App state is {:?}", app_state.as_str());
                if app_state < AppState::Granted {
                    log::warn!("- State is less than Granted");
                    return Ok(false);
                } else if app_state == AppState::Granted {
                    let active_request_id = match application_file.clone().lifecycle.get_active_allocation_id() {

                        Some(id) => id,
                        None => {
                            log::warn!("- No active request");
                            return Ok(false);
                        }
                    };
                    let active_request = match application_file.allocation.find_one(active_request_id) {
                        Some(request) => request,
                        None => {
                            log::warn!("- No active request");
                            return Ok(false);
                        }
                    };
                    let signers: application::file::Verifiers = active_request.signers.clone();
                    if signers.0.len() != 2 {
                        log::warn!("- Not enough signers");
                        return Ok(false);
                    }
                    let signer = signers.0.get(1).unwrap();
                    let signer_gh_handle = signer.github_username.clone();
                    let valid_verifiers = Self::fetch_verifiers(owner.clone(), repo.clone()).await?;
                    if valid_verifiers.is_valid(&signer_gh_handle) {
                        log::info!("- Validated!");
                        Self::issue_datacap_request_signature(
                            application_file.clone(), 
                            "approved".to_string(),
                            owner.clone(), 
                            repo.clone()
                        ).await?;
                        Self::update_issue_labels(
                            application_file.issue_number.clone(), 
                            &[AppState::Granted.as_str()],
                            owner.clone(), 
                            repo.clone()
                        ).await?;
                        Self::issue_granted(
                            application_file.issue_number.clone(),
                            owner.clone(), 
                            repo.clone()
                        ).await?;
                        return Ok(true);
                    }

                    log::warn!("- Not validated!");
                    Ok(false)
                } else {
                    log::info!("- State is greater than Granted");
                    Ok(true)
                }
            }
            Err(e) => Err(LDNError::Load(format!(
                "PR number {} not found: {}",
                pr_number, e
            ))),
        }
    }

    pub async fn validate_proposal(pr_number: u64, owner: String, repo: String) -> Result<bool, LDNError> {
        log::info!("Starting validate_proposal:");
        log::info!("- Validating proposal for PR number {}", pr_number);
        match LDNApplication::single_active(pr_number, owner.clone(), repo.clone()).await {
            Ok(application_file) => {
                if application_file.lifecycle.get_active_status() == false {
                    log::info!("No proposal to validate. Application lifecycle is inactive so the Total DC was reached.");
                    return Ok(true);
                }
                let app_state: AppState = application_file.lifecycle.get_state();
                log::info!("- App state is {:?}", app_state.as_str());
                if app_state < AppState::StartSignDatacap {
                    log::warn!("- State is less than StartSignDatacap");
                    return Ok(false);
                } else if app_state == AppState::StartSignDatacap {
                    let active_request = application_file.allocation.active();
                    if active_request.is_none() {
                        log::warn!("- No active request");
                        return Ok(false);
                    }
                    let active_request = active_request.unwrap();
                    let signers = active_request.signers.clone();
                    if signers.0.len() != 1 {
                        log::warn!("- Not enough signers");
                        return Ok(false);
                    }
                    let signer = signers.0.get(0).unwrap();
                    let signer_gh_handle = signer.github_username.clone();
                    let valid_verifiers = Self::fetch_verifiers(owner.clone(), repo.clone()).await?;
                    if valid_verifiers.is_valid(&signer_gh_handle) {
                        Self::issue_start_sign_dc(
                            application_file.issue_number.clone(),
                            owner.clone(), 
                            repo.clone()
                        ).await?;
                        Self::issue_datacap_request_signature(
                            application_file.clone(), 
                            "proposed".to_string(),
                            owner.clone(), 
                            repo.clone()
                        ).await?;
                        Self::update_issue_labels(
                            application_file.issue_number.clone(), 
                            &[AppState::StartSignDatacap.as_str()],
                            owner.clone(), 
                            repo.clone()
                        ).await?;

                        log::info!("- Validated!");
                        return Ok(true);
                    }
                    log::warn!("- Not validated!");
                    Ok(false)
                } else {
                    log::info!("- State is greater than StartSignDatacap");
                    Ok(true)
                }
            }
            Err(e) => Err(LDNError::Load(format!(
                "PR number {} not found: {}",
                pr_number, e
            ))),
        }
    }

    async fn issue_waiting_for_gov_review(issue_number: String, owner: String, repo: String) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            "Application is waiting for governance review",
        )
        .await
        .map_err(|e| {
            return LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ));
        })?;

        Ok(true)
    }

    async fn issue_datacap_request_trigger(application_file: ApplicationFile, owner: String, repo: String) -> Result<bool, LDNError> {
        let gh: GithubWrapper = GithubWrapper::new(owner, repo);

        let client_address =  application_file.lifecycle.client_on_chain_address.clone();
        let total_requested =  application_file.datacap.total_requested_amount.clone();
        let weekly_allocation =  application_file.datacap.weekly_allocation.clone();

        let issue_number = application_file.issue_number.clone();

        let comment = format!(
            "### Datacap Request Trigger
**Total DataCap requested**
> {}

**Expected weekly DataCap usage rate**
> {}

**Client address**
> {}",
            total_requested,
            weekly_allocation,
            client_address
        );

        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            &comment,
        )
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ));
            })
            .unwrap();
        Ok(true)
    }

    async fn issue_datacap_allocation_requested(
        application_file: ApplicationFile, 
        active_allocation: Option<&Allocation>, 
        owner: String, repo: String
    ) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);

        let issue_number = application_file.issue_number.clone();

        let mut datacap_allocation_requested = String::new();
        let mut id = String::new();

        if let Some(allocation) = active_allocation {
            datacap_allocation_requested = allocation.amount.clone();
            id = allocation.id.clone();
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

        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            &comment,
        )
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ));
            })
            .unwrap();
        Ok(true)
    }

    async fn issue_datacap_request_signature(
        application_file: ApplicationFile, 
        signature_step: String,
        owner: String,
        repo: String
    ) -> Result<bool, LDNError> {
        let active_allocation: Option<&Allocation> = application_file.allocation
            .0
            .iter()
            .find(|obj| Some(&obj.id) == application_file.lifecycle.active_request.clone().as_ref());

        let gh = GithubWrapper::new(owner, repo);

        let issue_number = application_file.issue_number.clone();

        let signature_step_capitalized = signature_step.chars().nth(0).unwrap().to_uppercase().to_string() + &signature_step.chars().skip(1).collect::<String>();

        let mut datacap_allocation_requested = String::new();
        let mut id = String::new();
        let mut signing_address = String::new();
        let mut message_cid = String::new();

        if let Some(allocation) = active_allocation {
            datacap_allocation_requested = allocation.amount.clone();
            id = allocation.id.clone();

            if let Some(first_verifier) = allocation.signers.0.get(0) {
                signing_address = first_verifier.signing_address.clone();
                message_cid = first_verifier.message_cid.clone();
            }
        }

        let comment = format!(
            "## Request {}
Your Datacap Allocation Request has been {} by the Notary
#### Message sent to Filecoin Network
> {}
#### Address
> {}
#### Datacap Allocated
> {}
#### Signer Address
> {}
#### Id
> {}
#### You can check the status of the message here: https://filfox.info/en/message/{}",
            signature_step_capitalized,
            signature_step,
            message_cid,
            application_file.lifecycle.client_on_chain_address.clone(),
            datacap_allocation_requested,
            signing_address,
            id,
            message_cid
        );

        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            &comment,
        )
        .await
        .map_err(|e| {
            return LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ));
        })
        .unwrap();

        Ok(true)
    }


    async fn issue_ready_to_sign(
        issue_number: String,
        owner: String,
        repo: String
    ) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            "Application is ready to sign",
        )
        .await
        .map_err(|e| {
            return LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ));
        })
        .unwrap();
        Ok(true)
    }

    async fn issue_start_sign_dc(issue_number: String, owner: String, repo: String) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        gh.add_comment_to_issue(
            issue_number.parse().unwrap(),
            "Application is in the process of signing datacap",
        )
        .await
        .map_err(|e| {
            return LDNError::New(format!(
                "Error adding comment to issue {} /// {}",
                issue_number, e
            ));
        })
        .unwrap();
        Ok(true)
    }
    async fn issue_granted(issue_number: String, owner: String, repo: String) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        gh.add_comment_to_issue(issue_number.parse().unwrap(), "Application is Granted")
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ));
            })
            .unwrap();
        Ok(true)
    }
    async fn issue_refill(issue_number: String, owner: String, repo: String) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        gh.add_comment_to_issue(issue_number.parse().unwrap(), "Application is in Refill")
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ));
            })
            .unwrap();
        gh.replace_issue_labels(issue_number.parse().unwrap(), &["Refill".to_string()])
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ));
            })
            .unwrap();
        Ok(true)
    }
    async fn issue_full_dc(issue_number: String, owner: String, repo: String) -> Result<bool, LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        gh.add_comment_to_issue(issue_number.parse().unwrap(), "Application is Completed")
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Error adding comment to issue {} /// {}",
                    issue_number, e
                ));
            })
            .unwrap();
        Ok(true)
    }

    async fn add_error_label(issue_number: String, comment: String, owner: String, repo: String) -> Result<(), LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        let num: u64 = issue_number.parse().expect("Not a valid integer");
        gh.add_error_label(num, comment).await
        .map_err(|e| {
            return LDNError::New(format!(
                "Error adding labels t to issue {} /// {}",
                issue_number, e
            ));
        })
        .unwrap();
        
        Ok(())
    }

    async fn update_issue_labels(issue_number: String, new_labels: &[&str], owner: String, repo: String) -> Result<(), LDNError> {
        let gh = GithubWrapper::new(owner, repo);
        let num: u64 = issue_number.parse().expect("Not a valid integer");
        gh.update_issue_labels(
            num, 
            new_labels
        ).await
        .map_err(|e| {
            return LDNError::New(format!(
                "Error adding labels t to issue {} /// {}",
                issue_number, e
            ));
        })
        .unwrap();

        Ok(())
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
    async fn create_pr(
        application_id: String,
        owner_name: String,
        app_branch_name: String,
        file_name: String,
        file_content: String,
        owner: String,
        repo: String
    ) -> Result<String, LDNError> {
        let initial_commit = Self::application_initial_commit(&owner_name, &application_id);
        let gh: GithubWrapper = GithubWrapper::new(owner, repo);
        let head_hash = gh.get_main_branch_sha().await.unwrap();
        let create_ref_request = gh
            .build_create_ref_request(app_branch_name.clone(), head_hash)
            .map_err(|e| {
                return LDNError::New(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                ));
            })?;

        let (_pr, file_sha) = gh
            .create_merge_request(CreateMergeRequestData {
                application_id: application_id.clone(),
                branch_name: app_branch_name,
                file_name,
                owner_name,
                ref_request: create_ref_request,
                file_content,
                commit: initial_commit,
            })
            .await
            .map_err(|e| {
                return LDNError::New(format!(
                    "Application issue {} cannot create merge request /// {}",
                    application_id, e
                ));
            })?;

        Ok(file_sha)
    }

    async fn create_refill_pr(
        application_id: String,
        owner_name: String,
        file_content: String,
        file_name: String,
        branch_name: String,
        file_sha: String,
        owner: String,
        repo: String
    ) -> Result<u64, LDNError> {
        let initial_commit = Self::application_initial_commit(&owner_name, &application_id);
        let gh: GithubWrapper = GithubWrapper::new(owner, repo);
        let head_hash = gh.get_main_branch_sha().await.unwrap();
        let create_ref_request = gh
            .build_create_ref_request(branch_name.clone(), head_hash)
            .map_err(|e| {
                return LDNError::New(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                ));
            })?;
        let pr = match gh
            .create_refill_merge_request(CreateRefillMergeRequestData {
                application_id: application_id.clone(),
                owner_name,
                file_name,
                file_sha,
                ref_request: create_ref_request,
                branch_name,
                file_content,
                commit: initial_commit,
            })
            .await
        {
            Ok(pr) => pr,
            Err(e) => {
                return Err(LDNError::New(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                )));
            }
        };
        Ok(pr.0.number)
    }

    pub(super) async fn add_commit_to(
        path: String,
        branch_name: String,
        commit_message: String,
        new_content: String,
        file_sha: String,
        owner: String,
        repo: String
    ) -> Option<()> {
        let gh: GithubWrapper = GithubWrapper::new(owner, repo);
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
            Err(_) => None,
        }
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
            "Governance Team User {} Moved Application to Proposal State from Governance Review State",
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
    match content.items.get(0) {
        Some(item) => {
            let sha = item.sha.clone();
            Some(sha)
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger::{Builder, Env};
    use tokio::time::{sleep, Duration};

    static OWNER: &str = "filecoin-project";
    static REPO: &str = "filplus-tooling-backend-test";

    #[tokio::test]
    async fn end_to_end() {
        // Set logging
        Builder::from_env(Env::default().default_filter_or("info")).init();
        log::info!("Starting end-to-end test");

        // Test Creating an application
        let gh: GithubWrapper = GithubWrapper::new(OWNER.to_string(), REPO.to_string());

        log::info!("Creating a new LDNApplication from issue");
        let ldn_application = match LDNApplication::new_from_issue(CreateApplicationInfo {
            issue_number: "706".to_string(),
            owner: OWNER.to_string(),
            repo: REPO.to_string()
        })
        .await
        {
            Ok(app) => app,
            Err(e) => {
                log::error!("Failed to create LDNApplication: {}", e);
                return;
            }
        };

        let application_id = ldn_application.application_id.to_string();
        log::info!("LDNApplication created with ID: {}", application_id);

        // Validate file creation
        log::info!("Validating file creation for application");
        if let Err(e) = gh
            .get_file(&ldn_application.file_name, &ldn_application.branch_name)
            .await
        {
            log::warn!(
                "File validation failed for application ID {}: {}",
                application_id,
                e
            );
        }

        // Validate pull request creation
        log::info!("Validating pull request creation for application");
        if let Err(e) = gh
            .get_pull_request_by_head(&LDNPullRequest::application_branch_name(
                application_id.as_str(),
            ))
            .await
        {
            log::warn!(
                "Pull request validation failed for application ID {}: {}",
                application_id,
                e
            );
        }

        sleep(Duration::from_millis(2000)).await;

        // Test Triggering an application
        log::info!("Loading application for triggering");
        let ldn_application_before_trigger =
            match LDNApplication::load(application_id.clone(), OWNER.to_string(), REPO.to_string()).await {
                Ok(app) => app,
                Err(e) => {
                    log::error!("Failed to load application for triggering: {}", e);
                    return;
                }
            };

        log::info!("Completing governance review");
        if let Err(e) = ldn_application_before_trigger
            .complete_governance_review(
                "actor_address".to_string(),
                OWNER.to_string(),
                REPO.to_string())
            .await
        {
            log::error!("Failed o complete governance review: {}", e);
            return;
        }

        let ldn_application_after_trigger = match LDNApplication::load(
            application_id.clone(), 
            OWNER.to_string(), 
            REPO.to_string()
        ).await
        {
            Ok(app) => app,
            Err(e) => {
                log::error!("Failed to load application after triggering: {}", e);
                return;
            }
        };

        assert_eq!(
            ldn_application_after_trigger.app_state().await.unwrap(),
            AppState::ReadyToSign
        );
        log::info!("Application state updated to ReadyToSign");
        sleep(Duration::from_millis(2000)).await;

        // Cleanup
        log::info!("Starting cleanup process");
        let head = &LDNPullRequest::application_branch_name(&application_id);
        match gh.get_pull_request_by_head(head).await {
            Ok(prs) => {
                if let Some(pr) = prs.get(0) {
                    let number = pr.number;
                    match gh.merge_pull_request(number).await {
                        Ok(_) => log::info!("Merged pull request {}", number),
                        Err(_) => log::info!("Pull request {} was already merged", number),
                    };
                }
            }
            Err(e) => log::warn!("Failed to get pull request by head: {}", e),
        };

        sleep(Duration::from_millis(3000)).await;

        let file = match gh.get_file(&ldn_application.file_name, "main").await {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to get file: {}", e);
                return;
            }
        };

        let file_sha = file.items[0].sha.clone();
        let remove_file_request = gh
            .delete_file(&ldn_application.file_name, "main", "remove file", &file_sha)
            .await;
        let remove_branch_request = gh
            .build_remove_ref_request(LDNPullRequest::application_branch_name(&application_id))
            .unwrap();

        if let Err(e) = gh.remove_branch(remove_branch_request).await {
            log::warn!("Failed to remove branch: {}", e);
        }
        if let Err(e) = remove_file_request {
            log::warn!("Failed to remove file: {}", e);
        }

        log::info!(
            "End-to-end test completed for application ID: {}",
            application_id
        );
    }
}
