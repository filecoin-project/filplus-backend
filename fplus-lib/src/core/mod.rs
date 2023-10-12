pub mod application;
use actix_web::{
    body::{BodySize, MessageBody},
    web::Bytes,
};
use std::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};

use chrono::Utc;
use futures::future;
use octocrab::models::{
    pulls::{FileDiff, PullRequest},
    repos::ContentItems,
};
use reqwest::Response;

use self::application::{
    allocations::{AllocationRequest, ApplicationAllocationTypes, ApplicationAllocationsSigner},
    lifecycle::ApplicationFileState,
    ApplicationFile,
};

use crate::{
    base64,
    core::application::{
        allocations::ApplicationAllocations,
        core_info::{ApplicationCoreInfo, ApplicationInfo},
        lifecycle::ApplicationLifecycle,
    },
    external_services::github::{CreateMergeRequestData, GithubWrapper},
    parsers::{parse_ldn_app_body, ParsedLDN},
};

const _VALID_ADDRESSES: [&str; 1] = ["t1v2"];

#[derive(serde::Deserialize)]
pub struct CreateApplicationInfo {
    pub application_id: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct CompleteNewApplicationProposalInfo {
    signer: ApplicationAllocationsSigner,
    request_id: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ProposeApplicationInfo {
    uuid: String,
    client_address: String,
    notary_address: String,
    time_of_signature: String,
    message_cid: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ApproveApplicationInfo {
    uuid: String,
    client_address: String,
    notary_address: String,
    allocation_amount: String,
    time_of_signature: String,
    message_cid: String,
}

#[derive(Debug)]
pub struct LDNApplication {
    github: GithubWrapper<'static>,
    pub application_id: String,
    file_sha: String,
}

#[derive(Debug)]
pub enum LDNApplicationError {
    NewApplicationError(String),
    LoadApplicationError(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CompleteGovernanceReviewInfo {
    actor: String,
}

impl Display for LDNApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LDNApplicationError::LoadApplicationError(e) => {
                write!(f, "LoadApplicationError: {}", e)
            }
            LDNApplicationError::NewApplicationError(e) => {
                write!(f, "NewApplicationError: {}", e)
            }
        }
    }
}

impl MessageBody for LDNApplicationError {
    type Error = std::convert::Infallible;

    fn size(&self) -> BodySize {
        match self {
            LDNApplicationError::LoadApplicationError(e) => BodySize::Sized(e.len() as u64),
            LDNApplicationError::NewApplicationError(e) => BodySize::Sized(e.len() as u64),
        }
    }

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        match Pin::<&mut LDNApplicationError>::into_inner(self) {
            LDNApplicationError::LoadApplicationError(e) => {
                Poll::Ready(Some(Ok(Bytes::from(e.clone()))))
            }
            LDNApplicationError::NewApplicationError(e) => {
                Poll::Ready(Some(Ok(Bytes::from(e.clone()))))
            }
        }
    }
}

impl LDNApplication {
    /// Get Active Applications
    /// Returns a list of all active applications
    /// New Implementation for get_all_active_applications
    /// we want to get all the pull requests, validate and then get the files from them
    /// we need to know how to build the path paramer the is used to get the file along with the
    /// branch name.
    pub async fn get_all_active_applications() -> Result<Vec<ApplicationFile>, LDNApplicationError>
    {
        let gh: GithubWrapper = GithubWrapper::new();
        let mut apps: Vec<ApplicationFile> = Vec::new();
        let pull_requests = gh.list_pull_requests().await.unwrap();
        let pull_requests = future::try_join_all(
            pull_requests
                .into_iter()
                .map(|pr: PullRequest| {
                    let number = pr.number;
                    gh.get_pull_request_files(number)
                })
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap()
        .into_iter()
        .flatten();
        let pull_requests: Vec<Response> = match future::try_join_all(
            pull_requests
                .into_iter()
                .map(|fd: FileDiff| reqwest::Client::new().get(&fd.raw_url.to_string()).send())
                .collect::<Vec<_>>(),
        )
        .await
        {
            Ok(res) => res,
            Err(_) => {
                return Err(LDNApplicationError::LoadApplicationError(
                    "Failed to get pull request files".to_string(),
                ))
            }
        };
        let pull_requests = match future::try_join_all(
            pull_requests
                .into_iter()
                .map(|r: Response| r.text())
                .collect::<Vec<_>>(),
        )
        .await
        {
            Ok(res) => res,
            Err(_) => {
                return Err(LDNApplicationError::LoadApplicationError(
                    "Failed to get pull request files".to_string(),
                ))
            }
        };
        for r in pull_requests {
            dbg!(&r);
            match serde_json::from_str::<ApplicationFile>(&r) {
                Ok(app) => apps.push(app),
                Err(_) => continue,
            }
        }
        Ok(apps)
    }

    /// Get Application by Pull Request Number
    pub async fn get_by_pr_number(pr_number: u64)   -> Result<ApplicationFile, LDNApplicationError> {
        let gh: GithubWrapper = GithubWrapper::new();
        let pr = gh.get_pull_request_files(pr_number).await.unwrap();
        // we should only have single file in the PR
        let file: FileDiff = pr[0].clone();
        let file = reqwest::Client::new()
            .get(&file.raw_url.to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let file = serde_json::from_str::<ApplicationFile>(&file).unwrap();
        Ok(file)
    }

    /// Load Application By ID
    /// TODO: FIXME!
    pub async fn load(application_id: String) -> Result<Self, LDNApplicationError> {
        let gh: GithubWrapper = GithubWrapper::new();
        let app_path = LDNPullRequest::application_path(&application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&application_id);

        match gh.get_file(&app_path, &app_branch_name).await {
            Ok(file) => {
                println!("Loading issue: {}", &application_id);
                let file_sha = match GithubWrapper::get_file_sha(&file) {
                    Some(file_sha) => file_sha,
                    None => {
                        return Err(LDNApplicationError::LoadApplicationError(format!(
                            "Application issue {} file does not exist",
                            application_id
                        )))
                    }
                };
                Ok(LDNApplication {
                    github: gh,
                    application_id,
                    file_sha,
                })
            }
            Err(_) => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} file does not exist",
                    application_id
                )))
            }
        }
    }

    /// Create New Application
    pub async fn new(info: CreateApplicationInfo) -> Result<Self, LDNApplicationError> {
        let application_id = info.application_id;
        let gh: GithubWrapper = GithubWrapper::new();
        let (parsed_ldn, _) = LDNApplication::parse(application_id.clone()).await?;
        let app_path = LDNPullRequest::application_path(&application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&application_id);

        match gh.get_file(&app_path, &app_branch_name).await {
            Err(_) => {
                let (pr_number, file_sha) = LDNPullRequest::create_empty_pr(
                    application_id.clone(),
                    parsed_ldn.name.clone(),
                    LDNPullRequest::application_branch_name(&application_id),
                    None,
                )
                .await?;

                let app_lifecycle = ApplicationLifecycle::governance_review_state(pr_number);
                let app_core_info: ApplicationCoreInfo = ApplicationCoreInfo::new(
                    parsed_ldn.name.clone(),
                    parsed_ldn.region,
                    "GithubHandleTodo".to_string(),
                    "TODO".to_string(), // industry
                    parsed_ldn.address,
                    parsed_ldn.datacap_requested,
                    parsed_ldn.datacap_weekly_allocation,
                    parsed_ldn.website,
                    "TODO".to_string(), // social media
                );
                let app_allocations = ApplicationAllocations::default();
                let app_info = ApplicationInfo::new(app_core_info, app_lifecycle, app_allocations);
                let application_file = ApplicationFile::new(app_info, application_id.clone()).await;
                let file_content = match serde_json::to_string_pretty(&application_file) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(LDNApplicationError::NewApplicationError(format!(
                            "Application issue file is corrupted /// {}",
                            e
                        )))
                    }
                };
                let pr_handler = LDNPullRequest::load(&application_id, &parsed_ldn.name);
                pr_handler
                    .add_commit(
                        LDNPullRequest::application_move_to_governance_review(),
                        file_content,
                        file_sha.clone(),
                    )
                    .await;
                Ok(LDNApplication {
                    github: gh,
                    application_id,
                    file_sha,
                })
            }
            Ok(_) => {
                return Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} already exists",
                    application_id
                )))
            }
        }
    }

    /// Move application from Governance Review to Proposal
    pub async fn complete_governance_review(
        &self,
        info: CompleteGovernanceReviewInfo,
    ) -> Result<ApplicationFile, LDNApplicationError> {
        match self.app_state().await {
            Ok(s) => match s {
                ApplicationFileState::GovernanceReview => {
                    let app_file: ApplicationFile = self.app_file().await?;
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id,
                        &app_file.info.core_information.data_owner_name,
                    );
                    let app_file = app_file.complete_governance_review(info.actor.clone());
                    let (parsed_ldn, issue_creator) =
                        Self::parse(self.application_id.clone()).await?;
                    let new_alloc = AllocationRequest::new(
                        issue_creator,
                        "TODO".to_string(), // ?
                        "random request id".to_string(),
                        ApplicationAllocationTypes::New,
                        parsed_ldn.address,
                        Utc::now().to_string(),
                        parsed_ldn.datacap_requested,
                    );
                    let app_file = app_file.start_new_allocation(new_alloc);
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    match app_pull_request
                        .add_commit(
                            LDNPullRequest::application_move_to_proposal_commit(&info.actor),
                            file_content,
                            self.file_sha.clone(),
                        )
                        .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNApplicationError::NewApplicationError(format!(
                                "Application issue {} cannot be triggered(1)",
                                self.application_id
                            )))
                        }
                    }
                }
                _ => Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot be triggered(2)",
                    self.application_id
                ))),
            },
            Err(e) => Err(LDNApplicationError::NewApplicationError(format!(
                "Application issue {} cannot be triggered {}(3)",
                self.application_id, e
            ))),
        }
    }

    /// Move application from Proposal to Approved
    pub async fn complete_new_application_proposal(
        &self,
        info: CompleteNewApplicationProposalInfo,
    ) -> Result<ApplicationFile, LDNApplicationError> {
        let CompleteNewApplicationProposalInfo { signer, request_id } = info;
        match self.app_state().await {
            Ok(s) => match s {
                ApplicationFileState::Proposal => {
                    let app_file: ApplicationFile = self.app_file().await?;
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id,
                        &app_file.info.core_information.data_owner_name.clone(),
                    );
                    let app_lifecycle = app_file.info.application_lifecycle.set_approval_state();

                    let app_file = app_file.add_signer_to_allocation(
                        signer.clone(),
                        request_id,
                        app_lifecycle,
                    );
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    match app_pull_request
                        .add_commit(
                            LDNPullRequest::application_move_to_approval_commit(
                                &signer.signing_address,
                            ),
                            file_content,
                            self.file_sha.clone(),
                        )
                        .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNApplicationError::NewApplicationError(format!(
                                "Application issue {} cannot be proposed(1)",
                                self.application_id
                            )))
                        }
                    }
                }
                _ => Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot be proposed(2)",
                    self.application_id
                ))),
            },
            Err(e) => Err(LDNApplicationError::NewApplicationError(format!(
                "Application issue {} cannot be proposed {}(3)",
                self.application_id, e
            ))),
        }
    }

    /// Merge Application Pull Request
    pub async fn merge_new_application_pr(&self) -> Result<ApplicationFile, LDNApplicationError> {
        match self.app_state().await {
            Ok(s) => match s {
                ApplicationFileState::Confirmed => {
                    let app_file: ApplicationFile = self.app_file().await?;
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id,
                        &app_file.info.core_information.data_owner_name,
                    );
                    match app_pull_request
                        .merge_pr(
                            app_file
                                .info
                                .application_lifecycle
                                .initial_pr_number
                                .clone(),
                        )
                        .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNApplicationError::NewApplicationError(format!(
                                "Application issue {} cannot be proposed(1)",
                                self.application_id
                            )))
                        }
                    }
                }
                _ => Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot be proposed(2)",
                    self.application_id
                ))),
            },
            Err(e) => Err(LDNApplicationError::NewApplicationError(format!(
                "Application issue {} cannot be proposed {}(3)",
                self.application_id, e
            ))),
        }
    }

    /// Move application from Governance Review to Proposal
    pub async fn complete_new_application_approval(
        &self,
        info: CompleteNewApplicationProposalInfo,
    ) -> Result<ApplicationFile, LDNApplicationError> {
        let CompleteNewApplicationProposalInfo { signer, request_id } = info;
        match self.app_state().await {
            Ok(s) => match s {
                ApplicationFileState::Approval => {
                    let app_file: ApplicationFile = self.app_file().await?;
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id.clone(),
                        &app_file.info.core_information.data_owner_name,
                    );
                    let app_lifecycle = app_file.info.application_lifecycle.set_confirmed_state();

                    let app_file = app_file.add_signer_to_allocation(
                        signer.clone(),
                        request_id,
                        app_lifecycle,
                    );
                    let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                    match app_pull_request
                        .add_commit(
                            LDNPullRequest::application_move_to_confirmed_commit(
                                &signer.signing_address,
                            ),
                            file_content,
                            self.file_sha.clone(),
                        )
                        .await
                    {
                        Some(()) => Ok(app_file),
                        None => {
                            return Err(LDNApplicationError::NewApplicationError(format!(
                                "Application issue {} cannot be proposed(1)",
                                self.application_id
                            )))
                        }
                    }
                }
                _ => Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot be proposed(2)",
                    self.application_id
                ))),
            },
            Err(e) => Err(LDNApplicationError::NewApplicationError(format!(
                "Application issue {} cannot be proposed {}(3)",
                self.application_id, e
            ))),
        }
    }

    async fn parse(application_id: String) -> Result<(ParsedLDN, String), LDNApplicationError> {
        let gh: GithubWrapper = GithubWrapper::new();
        let issue = match gh.list_issue(application_id.parse().unwrap()).await {
            Ok(issue) => issue,
            Err(e) => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} does not exist /// {}",
                    application_id, e
                )))
            }
        };
        let issue_creator = issue.user.login;
        let issue_body = match issue.body {
            Some(body) => body,
            None => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} is empty",
                    application_id
                )))
            }
        };
        Ok((parse_ldn_app_body(&issue_body), issue_creator))
    }

    /// Return Application state
    async fn app_state(&self) -> Result<ApplicationFileState, LDNApplicationError> {
        let f = self.app_file().await?;
        Ok(f.info.application_lifecycle.get_state())
    }

    fn content_items_to_app_file(
        file: ContentItems,
    ) -> Result<ApplicationFile, LDNApplicationError> {
        let f = match &file.items[0].content {
            Some(f) => f,
            None => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application file is corrupted",
                )))
            }
        };
        match base64::decode(&f.replace("\n", "")) {
            Some(f) => {
                return Ok(ApplicationFile::from(f));
            }
            None => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue file is corrupted",
                )))
            }
        }
    }

    async fn app_file(&self) -> Result<ApplicationFile, LDNApplicationError> {
        let app_path = LDNPullRequest::application_path(&self.application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&self.application_id);
        dbg!(&app_path);
        dbg!(&app_branch_name);
        match self.github.get_file(&app_path, &app_branch_name).await {
            Ok(file) => Ok(LDNApplication::content_items_to_app_file(file)?),
            Err(e) => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} file does not exist /// {}",
                    self.application_id, e
                )))
            }
        }
    }

    pub async fn app_file_without_load(
        application_id: String,
    ) -> Result<ApplicationFile, LDNApplicationError> {
        let gh: GithubWrapper = GithubWrapper::new();
        let app_path = LDNPullRequest::application_path(&application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&application_id);

        match gh.get_file(&app_path, &app_branch_name).await {
            Ok(f) => Ok(LDNApplication::content_items_to_app_file(f))?,
            Err(_) => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} file does not exist",
                    application_id
                )))
            }
        }
    }

    pub async fn get_merged_applications() -> Result<Vec<ApplicationFile>, LDNApplicationError> {
        let gh: GithubWrapper<'_> = GithubWrapper::new();
        let mut all_files = gh.get_all_files().await.map_err(|e| {
            LDNApplicationError::LoadApplicationError(format!(
                "Failed to retrieve all files from GitHub. Reason: {}",
                e
            ))
        })?;
        all_files
            .items
            .retain(|item| item.download_url.is_some() && item.name.starts_with("Application"));
        let all_files = future::try_join_all(
            all_files
                .items
                .into_iter()
                .map(|fd| reqwest::Client::new().get(&fd.download_url.unwrap()).send())
                .collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| {
            LDNApplicationError::LoadApplicationError(format!(
                "Failed to fetch application files from their URLs. Reason: {}",
                e
            ))
        })?;

        let mut apps: Vec<ApplicationFile> = vec![];
        for f in all_files {
            let f = match f.text().await {
                Ok(f) => f,
                Err(_) => {
                    continue;
                }
            };
            match serde_json::from_str::<ApplicationFile>(&f) {
                Ok(app) => {
                    apps.push(app);
                }
                Err(_) => {
                    continue;
                }
            };
        }
        Ok(apps)
    }
}

impl From<String> for ParsedApplicationDataFields {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Data Owner Name" => ParsedApplicationDataFields::Name,
            "Data Owner Country/Region" => ParsedApplicationDataFields::Region,
            "Website" => ParsedApplicationDataFields::Website,
            // "Custom multisig" => ParsedApplicationDataFields::CustomNotary,
            "Identifier" => ParsedApplicationDataFields::Identifier,
            "Data Type of Application" => ParsedApplicationDataFields::DataType,
            "Total amount of DataCap being requested" => {
                ParsedApplicationDataFields::DatacapRequested
            }
            "Weekly allocation of DataCap requested" => {
                ParsedApplicationDataFields::DatacapWeeklyAllocation
            }
            "On-chain address for first allocation" => ParsedApplicationDataFields::Address,
            _ => ParsedApplicationDataFields::InvalidField,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ParsedApplicationDataFields {
    Name,
    Region,
    Website,
    DatacapRequested,
    DatacapWeeklyAllocation,
    Address,
    // CustomNotary,
    Identifier,
    DataType,
    InvalidField,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LDNPullRequest {
    pub branch_name: String,
    pub title: String,
    pub body: String,
    pub path: String,
}

impl LDNPullRequest {
    async fn create_empty_pr(
        application_id: String,
        owner_name: String,
        app_branch_name: String,
        base_hash: Option<String>,
    ) -> Result<(u64, String), LDNApplicationError> {
        let initial_commit = Self::application_initial_commit(&owner_name, &application_id);
        let create_ref_request = match GithubWrapper::new()
            .build_create_ref_request(app_branch_name.clone(), base_hash)
        {
            Ok(req) => req,
            Err(e) => {
                return Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue cannot create branch request object /// {}",
                    e
                )))
            }
        };

        let merge_request_data: CreateMergeRequestData = CreateMergeRequestData {
            application_id: application_id.clone(),
            owner_name,
            ref_request: create_ref_request,
            file_content: "{}".to_string(),
            commit: initial_commit,
        };

        let gh: GithubWrapper = GithubWrapper::new();
        let (pr, file_sha) = match gh.create_merge_request(merge_request_data).await {
            Ok((pr, file_sha)) => (pr, file_sha),
            Err(e) => {
                return Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                )));
            }
        };
        Ok((pr.number, file_sha))
    }

    pub(super) async fn add_commit(
        &self,
        commit_message: String,
        new_content: String,
        file_sha: String,
    ) -> Option<()> {
        let gh: GithubWrapper = GithubWrapper::new();
        match gh
            .update_file_content(
                &self.path,
                &commit_message,
                &new_content,
                &self.branch_name,
                &file_sha,
            )
            .await
        {
            Ok(_) => Some(()),
            Err(_) => None,
        }
    }

    pub(super) async fn merge_pr(&self, pr_number: u64) -> Option<()> {
        let gh: GithubWrapper = GithubWrapper::new();
        match gh.merge_pull_request(pr_number).await {
            Ok(_) => Some(()),
            Err(_) => None,
        }
    }

    pub(super) fn load(application_id: &str, owner_name: &str) -> Self {
        LDNPullRequest {
            branch_name: LDNPullRequest::application_branch_name(application_id),
            title: LDNPullRequest::application_title(application_id, owner_name),
            body: LDNPullRequest::application_body(application_id),
            path: LDNPullRequest::application_path(application_id),
        }
    }

    pub(super) fn application_branch_name(application_id: &str) -> String {
        format!("Application/{}", application_id)
    }

    pub(super) fn application_title(application_id: &str, owner_name: &str) -> String {
        format!("Application:{}:{}", application_id, owner_name)
    }

    pub(super) fn application_body(application_id: &str) -> String {
        format!("resolves #{}", application_id)
    }

    pub(super) fn application_path(application_id: &str) -> String {
        format!("Application:{}.json", application_id)
    }

    pub(super) fn application_initial_commit(owner_name: &str, application_id: &str) -> String {
        format!("Start Application: {}-{}", owner_name, application_id)
    }

    pub(super) fn application_move_to_governance_review() -> String {
        format!("Application is under review of governance team")
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

#[cfg(test)]
mod tests {
    use super::*;
    use octocrab::models::issues::Issue;
    use tokio::time::{sleep, Duration};

    #[ignore]
    #[tokio::test]
    async fn ldnapplication() {
        let res: Result<Vec<ApplicationFile>, LDNApplicationError> =
            LDNApplication::get_merged_applications().await;
        dbg!(&res);
        assert!(false);
    }
    #[ignore]
    #[tokio::test]
    async fn end_to_end() {
        // Test Creating an application
        let gh: GithubWrapper = GithubWrapper::new();

        // let branches = gh.list_branches().await.unwrap();
        let issue = gh.list_issue(63).await.unwrap();
        let test_issue: Issue = gh
            .create_issue("from test", &issue.body.unwrap())
            .await
            .unwrap();
        assert!(LDNApplication::new(CreateApplicationInfo {
            application_id: test_issue.number.to_string(),
        })
        .await
        .is_ok());

        let application_id = test_issue.number.to_string();

        // validate file was created
        assert!(gh
            .get_file(
                &LDNPullRequest::application_path(application_id.as_str()),
                &LDNPullRequest::application_branch_name(application_id.as_str())
            )
            .await
            .is_ok());

        // validate pull request was created
        assert!(gh
            .get_pull_request_by_head(&LDNPullRequest::application_branch_name(
                application_id.as_str()
            ))
            .await
            .is_ok());

        // Test Triggering an application
        let ldn_application_before_trigger =
            LDNApplication::load(application_id.clone()).await.unwrap();
        ldn_application_before_trigger
            .complete_governance_review(CompleteGovernanceReviewInfo {
                actor: "actor_address".to_string(),
            })
            .await
            .unwrap();
        let ldn_application_after_trigger =
            LDNApplication::load(application_id.clone()).await.unwrap();
        assert_eq!(
            ldn_application_after_trigger.app_state().await.unwrap(),
            ApplicationFileState::Proposal
        );
        dbg!("waiting for 2 second");
        sleep(Duration::from_millis(1000)).await;

        // // Test Proposing an application
        let ldn_application_after_trigger_success =
            LDNApplication::load(application_id.clone()).await.unwrap();
        ldn_application_after_trigger_success
            .complete_new_application_proposal(CompleteNewApplicationProposalInfo {
                request_id: "request_id".to_string(),
                signer: ApplicationAllocationsSigner {
                    signing_address: "signing_address".to_string(),
                    time_of_signature: "time_of_signature".to_string(),
                    message_cid: "message_cid".to_string(),
                    username: "gh_username".to_string(),
                },
            })
            .await
            .unwrap();
        let ldn_application_after_proposal =
            LDNApplication::load(application_id.clone()).await.unwrap();
        assert_eq!(
            ldn_application_after_proposal.app_state().await.unwrap(),
            ApplicationFileState::Approval
        );
        dbg!("waiting for 2 second");
        sleep(Duration::from_millis(1000)).await;

        // Test Approving an application
        let ldn_application_after_proposal_success =
            LDNApplication::load(application_id.clone()).await.unwrap();
        ldn_application_after_proposal_success
            .complete_new_application_approval(CompleteNewApplicationProposalInfo {
                request_id: "request_id".to_string(),
                signer: ApplicationAllocationsSigner {
                    signing_address: "signing_address".to_string(),
                    time_of_signature: "time_of_signature".to_string(),
                    message_cid: "message_cid".to_string(),
                    username: "gh_username".to_string(),
                },
            })
            .await
            .unwrap();
        let ldn_application_after_approval =
            LDNApplication::load(application_id.clone()).await.unwrap();
        assert_eq!(
            ldn_application_after_approval.app_state().await.unwrap(),
            ApplicationFileState::Confirmed
        );
        dbg!("waiting for 2 second");
        sleep(Duration::from_millis(1000)).await;

        // // Cleanup
        assert!(gh.close_issue(test_issue.number).await.is_ok());
        assert!(gh
            .close_pull_request(
                gh.get_pull_request_by_head(&LDNPullRequest::application_branch_name(
                    &application_id.clone()
                ))
                .await
                .unwrap()[0]
                    .number,
            )
            .await
            .is_ok());
        let remove_branch_request = gh
            .build_remove_ref_request(LDNPullRequest::application_branch_name(
                &application_id.clone(),
            ))
            .unwrap();
        assert!(gh.remove_branch(remove_branch_request).await.is_ok());
    }
}
