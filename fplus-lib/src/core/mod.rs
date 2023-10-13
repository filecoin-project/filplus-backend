use chrono::Utc;
use futures::future;
use octocrab::models::{
    pulls::{FileDiff, PullRequest},
    repos::ContentItems,
};
use reqwest::Response;
use serde::{Deserialize, Serialize};

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
    error::LDNApplicationError,
    external_services::github::{
        CreateMergeRequestData, CreateRefillMergeRequestData, GithubWrapper,
    },
    parsers::{parse_ldn_app_body, ParsedLDN},
};

pub mod application;

#[derive(Deserialize)]
pub struct CreateApplicationInfo {
    pub application_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CompleteNewApplicationProposalInfo {
    signer: ApplicationAllocationsSigner,
    request_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposeApplicationInfo {
    uuid: String,
    client_address: String,
    notary_address: String,
    time_of_signature: String,
    message_cid: String,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteGovernanceReviewInfo {
    actor: String,
}

#[derive(Deserialize, Debug)]
pub struct RefillInfo {
    pub id: String,
    pub amount: String,
    pub amount_type: String,
}

impl LDNApplication {
    pub async fn active(
        filter: Option<String>,
    ) -> Result<Vec<ApplicationFile>, LDNApplicationError> {
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
            match serde_json::from_str::<ApplicationFile>(&r) {
                Ok(app) => {
                    if filter.is_none() {
                        apps.push(app)
                    } else {
                        if app.id == filter.clone().unwrap() {
                            apps.push(app)
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(apps)
    }
    pub async fn load(application_id: String) -> Result<Self, LDNApplicationError> {
        let gh: GithubWrapper = GithubWrapper::new();
        let app_path = LDNPullRequest::application_path(&application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&application_id);
        let file = gh
            .get_file(&app_path, &app_branch_name)
            .await
            .map_err(|e| {
                LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} file does not exist /// {}",
                    application_id, e
                ))
            })?;
        let file_sha = get_file_sha(&file).unwrap();
        Ok(LDNApplication {
            github: gh,
            application_id,
            file_sha,
        })
    }

    /// Create New Application
    pub async fn new(info: CreateApplicationInfo) -> Result<Self, LDNApplicationError> {
        let application_id = info.application_id;
        let gh: GithubWrapper = GithubWrapper::new();
        let (parsed_ldn, _) =
            LDNApplication::parse_application_issue(application_id.clone()).await?;
        let app_path = LDNPullRequest::application_path(&application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&application_id);

        match gh.get_file(&app_path, &app_branch_name).await {
            Err(_) => {
                let file_sha = LDNPullRequest::create_empty_pr(
                    application_id.clone(),
                    parsed_ldn.name.clone(),
                    LDNPullRequest::application_branch_name(&application_id),
                    None,
                )
                .await?;
                let app_allocations = ApplicationAllocations::default();
                let app_lifecycle = ApplicationLifecycle::governance_review_state(None);
                let app_core_info: ApplicationCoreInfo = ApplicationCoreInfo::new(
                    parsed_ldn.name.clone(),
                    parsed_ldn.region,
                    "GithubHandleTodo".to_string(),
                    "industry".to_string(),
                    parsed_ldn.address,
                    parsed_ldn.datacap_requested,
                    parsed_ldn.datacap_weekly_allocation,
                    parsed_ldn.website,
                    "social_media".to_string(),
                );
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
                    let app_file: ApplicationFile = self.file().await?;
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id,
                        &app_file.info.core_information.data_owner_name,
                    );
                    let uuid = uuidv4::uuid::v4();
                    let app_file =
                        app_file.complete_governance_review(info.actor.clone(), uuid.clone());
                    let (parsed_ldn, issue_creator) =
                        Self::parse_application_issue(self.application_id.clone()).await?;
                    let new_alloc = AllocationRequest::new(
                        issue_creator,
                        uuid,
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
                    let app_file: ApplicationFile = self.file().await?;
                    if !app_file
                        .info
                        .datacap_allocations
                        .is_active(request_id.clone())
                    {
                        return Err(LDNApplicationError::LoadApplicationError(format!(
                            "Request {} is not active",
                            request_id
                        )));
                    }
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id,
                        &app_file.info.core_information.data_owner_name.clone(),
                    );
                    let app_lifecycle = app_file
                        .info
                        .application_lifecycle
                        .set_approval_state(Some(request_id.clone()));

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

    /// Move application from Governance Review to Proposal
    pub async fn complete_new_application_approval(
        &self,
        info: CompleteNewApplicationProposalInfo,
    ) -> Result<ApplicationFile, LDNApplicationError> {
        let CompleteNewApplicationProposalInfo { signer, request_id } = info;
        match self.app_state().await {
            Ok(s) => match s {
                ApplicationFileState::Approval => {
                    let app_file: ApplicationFile = self.file().await?;
                    let app_pull_request = LDNPullRequest::load(
                        &self.application_id.clone(),
                        &app_file.info.core_information.data_owner_name,
                    );
                    let app_lifecycle = app_file
                        .info
                        .application_lifecycle
                        .set_confirmed_state(Some(request_id.clone()));

                    let app_file = app_file.add_signer_to_allocation_and_complete(
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

    async fn parse_application_issue(
        application_id: String,
    ) -> Result<(ParsedLDN, String), LDNApplicationError> {
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
        let issue_body = match issue.body {
            Some(body) => body,
            None => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} is empty",
                    application_id
                )))
            }
        };
        Ok((parse_ldn_app_body(&issue_body), issue.user.login))
    }

    /// Return Application state
    async fn app_state(&self) -> Result<ApplicationFileState, LDNApplicationError> {
        let f = self.file().await?;
        Ok(f.info.application_lifecycle.get_state())
    }

    /// Return Application state
    pub async fn total_dc_reached(id: String) -> Result<bool, LDNApplicationError> {
        let merged = Self::merged().await?;
        let app = match merged.iter().find(|app| app.id == id) {
            Some(app) => app,
            None => {
                return Err(LDNApplicationError::LoadApplicationError(format!(
                    "Application issue {} does not exist",
                    id
                )))
            }
        };
        match app.info.application_lifecycle.get_state() {
            ApplicationFileState::Confirmed => {
                let app = app.reached_total_datacap();
                let pr_handler =
                    LDNPullRequest::load(&app.id, &app.info.core_information.data_owner_name);
                let gh: GithubWrapper<'_> = GithubWrapper::new();

                let ContentItems { items } = gh
                    .get_file(&pr_handler.path, &pr_handler.branch_name)
                    .await
                    .unwrap();

                LDNPullRequest::create_refill_pr(
                    app.id.clone(),
                    app.info.core_information.data_owner_name.clone(),
                    items[0].sha.clone(),
                    serde_json::to_string_pretty(&app).unwrap(),
                )
                .await?;
                // let app_file: ApplicationFile = self.file().await?;
                // let file_content = serde_json::to_string_pretty(&app_file).unwrap();
                Ok(true)
            }
            _ => Ok(false),
        }
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

    async fn file(&self) -> Result<ApplicationFile, LDNApplicationError> {
        let app_path = LDNPullRequest::application_path(&self.application_id);
        let app_branch_name = LDNPullRequest::application_branch_name(&self.application_id);
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

    pub async fn merged() -> Result<Vec<ApplicationFile>, LDNApplicationError> {
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
        let active: Vec<ApplicationFile> = Self::active(None).await?;
        for f in all_files {
            let f = match f.text().await {
                Ok(f) => f,
                Err(_) => {
                    continue;
                }
            };
            match serde_json::from_str::<ApplicationFile>(&f) {
                Ok(app) => {
                    if active.iter().find(|a| a.id == app.id).is_none()
                        && app.info.application_lifecycle.is_active
                    {
                        apps.push(app);
                    }
                }
                Err(_) => {
                    continue;
                }
            };
        }
        Ok(apps)
    }

    pub async fn refill(refill_info: RefillInfo) -> Result<bool, LDNApplicationError> {
        let gh = GithubWrapper::new();
        let apps = LDNApplication::merged().await?;

        if let Some(app) = apps.iter().find(|app| app.id == refill_info.id) {
            let uuid = uuidv4::uuid::v4();
            let app_lifecycle = app
                .info
                .application_lifecycle
                .set_refill_proposal_state(Some(uuid.clone()));
            let new_request: AllocationRequest = AllocationRequest {
                actor: "SSA Bot".to_string(),
                id: uuid.clone(),
                request_type: ApplicationAllocationTypes::Refill,
                client_address: app.info.core_information.data_owner_address.clone(),
                created_at: Utc::now().to_string(),
                is_active: true,
                allocation_amount: format!("{}{}", refill_info.amount, refill_info.amount_type),
            };
            let app_allocations = app
                .clone()
                .info
                .datacap_allocations
                .add_new_request(new_request);
            let app_info = ApplicationInfo::new(
                app.info.core_information.clone(),
                app_lifecycle,
                app_allocations,
            );
            let application_file = ApplicationFile::new(app_info, app.id.clone()).await;
            let pr_handler =
                LDNPullRequest::load(&app.id, &app.info.core_information.data_owner_name);

            let ContentItems { items } = gh
                .get_file(&pr_handler.path, &pr_handler.branch_name)
                .await
                .unwrap();

            LDNPullRequest::create_refill_pr(
                app.id.clone(),
                app.info.core_information.data_owner_name.clone(),
                items[0].sha.clone(),
                serde_json::to_string_pretty(&application_file).unwrap(),
            )
            .await?;
            return Ok(true);
        }
        Err(LDNApplicationError::LoadApplicationError(
            "Failed to get application file".to_string(),
        ))
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
    async fn create_empty_pr(
        application_id: String,
        owner_name: String,
        app_branch_name: String,
        base_hash: Option<String>,
    ) -> Result<String, LDNApplicationError> {
        let initial_commit = Self::application_initial_commit(&owner_name, &application_id);
        let gh: GithubWrapper = GithubWrapper::new();
        let create_ref_request =
            match gh.build_create_ref_request(app_branch_name.clone(), base_hash) {
                Ok(req) => req,
                Err(e) => {
                    return Err(LDNApplicationError::NewApplicationError(format!(
                        "Application issue cannot create branch request object /// {}",
                        e
                    )))
                }
            };

        let (_pr, file_sha) = match gh
            .create_merge_request(CreateMergeRequestData {
                application_id: application_id.clone(),
                owner_name,
                ref_request: create_ref_request,
                file_content: "{}".to_string(),
                commit: initial_commit,
            })
            .await
        {
            Ok((pr, file_sha)) => (pr, file_sha),
            Err(e) => {
                return Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                )));
            }
        };
        Ok(file_sha)
    }

    async fn create_refill_pr(
        application_id: String,
        owner_name: String,
        file_sha: String,
        file_content: String,
    ) -> Result<u64, LDNApplicationError> {
        let initial_commit = Self::application_initial_commit(&owner_name, &application_id);
        let gh: GithubWrapper = GithubWrapper::new();
        let pr = match gh
            .create_refill_merge_request(CreateRefillMergeRequestData {
                application_id: application_id.clone(),
                owner_name,
                file_content,
                commit: initial_commit,
                file_sha,
            })
            .await
        {
            Ok(pr) => pr,
            Err(e) => {
                return Err(LDNApplicationError::NewApplicationError(format!(
                    "Application issue {} cannot create branch /// {}",
                    application_id, e
                )));
            }
        };
        Ok(pr.number)
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
pub fn get_file_sha(content: &ContentItems) -> Option<String> {
    match content.items.get(0) {
        Some(item) => {
            let sha = item.sha.clone();
            Some(sha)
        }
        None => None,
    }
}

// mod tests {
// #[tokio::test]
// async fn refill() {
//     let res: Result<Vec<ApplicationFile>, LDNApplicationError> =
//         LDNApplication::refill_existing_application(vec![RefillInfo {
//             id: "229".to_string(),
//             amount: "10".to_string(),
//             amount_type: "PiB".to_string(),
//         }])
//         .await;
//     dbg!(&res);
//     assert!(false);
// }
// #[ignore]
// #[tokio::test]
// async fn ldnapplication() {
//     let res: Result<Vec<ApplicationFile>, LDNApplicationError> =
//         LDNApplication::get_merged_applications().await;
//     dbg!(&res);
//     assert!(false);
// }
// #[ignore]
// #[tokio::test]
// async fn end_to_end() {
//     // Test Creating an application
//     let gh: GithubWrapper = GithubWrapper::new();

//     // let branches = gh.list_branches().await.unwrap();
//     let issue: Issue = gh.list_issue(63).await.unwrap();
//     let test_issue: Issue = gh
//         .create_issue("from test", &issue.body.unwrap())
//         .await
//         .unwrap();
//     assert!(LDNApplication::new(CreateApplicationInfo {
//         application_id: test_issue.number.to_string(),
//     })
//     .await
//     .is_ok());

//     let application_id = test_issue.number.to_string();

//     // validate file was created
//     assert!(gh
//         .get_file(
//             &LDNPullRequest::application_path(application_id.as_str()),
//             &LDNPullRequest::application_branch_name(application_id.as_str())
//         )
//         .await
//         .is_ok());

//     // validate pull request was created
//     assert!(gh
//         .get_pull_request_by_head(&LDNPullRequest::application_branch_name(
//             application_id.as_str()
//         ))
//         .await
//         .is_ok());

//     // Test Triggering an application
//     let ldn_application_before_trigger =
//         LDNApplication::load(application_id.clone()).await.unwrap();
//     ldn_application_before_trigger
//         .complete_governance_review(CompleteGovernanceReviewInfo {
//             actor: "actor_address".to_string(),
//         })
//         .await
//         .unwrap();
//     let ldn_application_after_trigger =
//         LDNApplication::load(application_id.clone()).await.unwrap();
//     assert_eq!(
//         ldn_application_after_trigger.app_state().await.unwrap(),
//         ApplicationFileState::Proposal
//     );
//     dbg!("waiting for 2 second");
//     sleep(Duration::from_millis(1000)).await;

//     // // Test Proposing an application
//     let ldn_application_after_trigger_success =
//         LDNApplication::load(application_id.clone()).await.unwrap();
//     ldn_application_after_trigger_success
//         .complete_new_application_proposal(CompleteNewApplicationProposalInfo {
//             request_id: "request_id".to_string(),
//             signer: ApplicationAllocationsSigner {
//                 signing_address: "signing_address".to_string(),
//                 time_of_signature: "time_of_signature".to_string(),
//                 message_cid: "message_cid".to_string(),
//             },
//         })
//         .await
//         .unwrap();
//     let ldn_application_after_proposal =
//         LDNApplication::load(application_id.clone()).await.unwrap();
//     assert_eq!(
//         ldn_application_after_proposal.app_state().await.unwrap(),
//         ApplicationFileState::Approval
//     );
//     dbg!("waiting for 2 second");
//     sleep(Duration::from_millis(1000)).await;

//     // Test Approving an application
//     let ldn_application_after_proposal_success =
//         LDNApplication::load(application_id.clone()).await.unwrap();
//     ldn_application_after_proposal_success
//         .complete_new_application_approval(CompleteNewApplicationProposalInfo {
//             request_id: "request_id".to_string(),
//             signer: ApplicationAllocationsSigner {
//                 signing_address: "signing_address".to_string(),
//                 time_of_signature: "time_of_signature".to_string(),
//                 message_cid: "message_cid".to_string(),
//             },
//         })
//         .await
//         .unwrap();
//     let ldn_application_after_approval =
//         LDNApplication::load(application_id.clone()).await.unwrap();
//     assert_eq!(
//         ldn_application_after_approval.app_state().await.unwrap(),
//         ApplicationFileState::Confirmed
//     );
//     dbg!("waiting for 2 second");
//     sleep(Duration::from_millis(1000)).await;

//     // // Cleanup
//     assert!(gh.close_issue(test_issue.number).await.is_ok());
//     assert!(gh
//         .close_pull_request(
//             gh.get_pull_request_by_head(&LDNPullRequest::application_branch_name(
//                 &application_id.clone()
//             ))
//             .await
//             .unwrap()[0]
//                 .number,
//         )
//         .await
//         .is_ok());
//     let remove_branch_request = gh
//         .build_remove_ref_request(LDNPullRequest::application_branch_name(
//             &application_id.clone(),
//         ))
//         .unwrap();
//     assert!(gh.remove_branch(remove_branch_request).await.is_ok());
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use octocrab::models::issues::Issue;
//     use tokio::time::{sleep, Duration};

//     #[ignore]
//     #[tokio::test]
//     async fn ldnapplication() {
//         let res: Result<Vec<ApplicationFile>, LDNApplicationError> =
//             LDNApplication::get_merged_applications().await;
//         dbg!(&res);
//         assert!(false);
//     }
//     #[ignore]
//     #[tokio::test]
//     async fn end_to_end() {
//         // Test Creating an application
//         let gh: GithubWrapper = GithubWrapper::new();

//         // let branches = gh.list_branches().await.unwrap();
//         let issue: Issue = gh.list_issue(63).await.unwrap();
//         let test_issue: Issue = gh
//             .create_issue("from test", &issue.body.unwrap())
//             .await
//             .unwrap();
//         assert!(LDNApplication::new(CreateApplicationInfo {
//             application_id: test_issue.number.to_string(),
//         })
//         .await
//         .is_ok());

//         let application_id = test_issue.number.to_string();

//         // validate file was created
//         assert!(gh
//             .get_file(
//                 &LDNPullRequest::application_path(application_id.as_str()),
//                 &LDNPullRequest::application_branch_name(application_id.as_str())
//             )
//             .await
//             .is_ok());

//         // validate pull request was created
//         assert!(gh
//             .get_pull_request_by_head(&LDNPullRequest::application_branch_name(
//                 application_id.as_str()
//             ))
//             .await
//             .is_ok());

//         // Test Triggering an application
//         let ldn_application_before_trigger =
//             LDNApplication::load(application_id.clone()).await.unwrap();
//         ldn_application_before_trigger
//             .complete_governance_review(CompleteGovernanceReviewInfo {
//                 actor: "actor_address".to_string(),
//             })
//             .await
//             .unwrap();
//         let ldn_application_after_trigger =
//             LDNApplication::load(application_id.clone()).await.unwrap();
//         assert_eq!(
//             ldn_application_after_trigger.app_state().await.unwrap(),
//             ApplicationFileState::Proposal
//         );
//         dbg!("waiting for 2 second");
//         sleep(Duration::from_millis(1000)).await;

//         // // Test Proposing an application
//         let ldn_application_after_trigger_success =
//             LDNApplication::load(application_id.clone()).await.unwrap();
//         ldn_application_after_trigger_success
//             .complete_new_application_proposal(CompleteNewApplicationProposalInfo {
//                 request_id: "request_id".to_string(),
//                 signer: ApplicationAllocationsSigner {
//                     signing_address: "signing_address".to_string(),
//                     time_of_signature: "time_of_signature".to_string(),
//                     message_cid: "message_cid".to_string(),
//                     username: "gh_username".to_string(),
//                 },
//             })
//             .await
//             .unwrap();
//         let ldn_application_after_proposal =
//             LDNApplication::load(application_id.clone()).await.unwrap();
//         assert_eq!(
//             ldn_application_after_proposal.app_state().await.unwrap(),
//             ApplicationFileState::Approval
//         );
//         dbg!("waiting for 2 second");
//         sleep(Duration::from_millis(1000)).await;

//         // Test Approving an application
//         let ldn_application_after_proposal_success =
//             LDNApplication::load(application_id.clone()).await.unwrap();
//         ldn_application_after_proposal_success
//             .complete_new_application_approval(CompleteNewApplicationProposalInfo {
//                 request_id: "request_id".to_string(),
//                 signer: ApplicationAllocationsSigner {
//                     signing_address: "signing_address".to_string(),
//                     time_of_signature: "time_of_signature".to_string(),
//                     message_cid: "message_cid".to_string(),
//                     username: "gh_username".to_string(),
//                 },
//             })
//             .await
//             .unwrap();
//         let ldn_application_after_approval =
//             LDNApplication::load(application_id.clone()).await.unwrap();
//         assert_eq!(
//             ldn_application_after_approval.app_state().await.unwrap(),
//             ApplicationFileState::Confirmed
//         );
//         dbg!("waiting for 2 second");
//         sleep(Duration::from_millis(1000)).await;

//         // // Cleanup
//         assert!(gh.close_issue(test_issue.number).await.is_ok());
//         assert!(gh
//             .close_pull_request(
//                 gh.get_pull_request_by_head(&LDNPullRequest::application_branch_name(
//                     &application_id.clone()
//                 ))
//                 .await
//                 .unwrap()[0]
//                     .number,
//             )
//             .await
//             .is_ok());
//         let remove_branch_request = gh
//             .build_remove_ref_request(LDNPullRequest::application_branch_name(
//                 &application_id.clone(),
//             ))
//             .unwrap();
//         assert!(gh.remove_branch(remove_branch_request).await.is_ok());
