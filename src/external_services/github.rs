use http::header::USER_AGENT;
use http::{Request, Uri};
use hyper_rustls::HttpsConnectorBuilder;

use crate::core::LDNPullRequest;
use octocrab::auth::AppAuth;
use octocrab::models::issues::{Comment, Issue};
use octocrab::models::pulls::PullRequest;
use octocrab::models::repos::{Branch, ContentItems, FileUpdate};
use octocrab::models::{InstallationId, IssueState};
use octocrab::params::{pulls::State as PullState, State};
use octocrab::service::middleware::base_uri::BaseUriLayer;
use octocrab::service::middleware::extra_headers::ExtraHeadersLayer;
use octocrab::{AuthState, Error as OctocrabError, Octocrab, OctocrabBuilder, Page};
use std::sync::Arc;

const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_OWNER: &str = "jbesraa";
const GITHUB_REPO: &str = "light-node";
const APP_ID: u64 = 344928;
const APP_INSTALLATION_ID: u64 = 38402249;
const MAIN_BRANCH_HASH: &str = "a9f99d9fc56cae689a0bf0ee177c266287eb48cd";

#[derive(Debug)]
pub struct CreateMergeRequestData {
    pub application_id: String,
    pub owner_name: String,
    pub ref_request: Request<String>,
    pub file_content: String,
    pub commit: String,
}

#[derive(Debug)]
pub struct GithubWrapper {
    pub inner: Arc<Octocrab>,
}

impl GithubWrapper {
    pub fn new() -> Self {
        let connector = HttpsConnectorBuilder::new()
            .with_native_roots() // enabled the `rustls-native-certs` feature in hyper-rustls
            .https_only()
            .enable_http1()
            .build();

        let client = hyper::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(15))
            .build(connector);
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("../../gh-private-key.pem"))
            .unwrap();
        let octocrab = OctocrabBuilder::new_empty()
            .with_service(client)
            .with_layer(&BaseUriLayer::new(Uri::from_static(GITHUB_API_URL)))
            .with_layer(&ExtraHeadersLayer::new(Arc::new(vec![(
                USER_AGENT,
                "octocrab".parse().unwrap(),
            )])))
            .with_auth(AuthState::App(AppAuth {
                app_id: APP_ID.into(),
                key,
            }))
            .build()
            .expect("Could not create Octocrab instance");
        let iod: InstallationId = APP_INSTALLATION_ID
            .try_into()
            .expect("Invalid installation id");
        let installation = octocrab.installation(iod);
        Self {
            inner: Arc::new(installation),
        }
    }

    pub async fn list_issues(&self) -> Result<Vec<Issue>, OctocrabError> {
        let iid = self
            .inner
            .issues(GITHUB_OWNER, GITHUB_REPO)
            .list()
            .state(State::Open)
            .send()
            .await?;
        Ok(iid.into_iter().map(|i: Issue| i).collect())
    }

    pub async fn list_issue(&self, number: u64) -> Result<Issue, OctocrabError> {
        let iid = self
            .inner
            .issues(GITHUB_OWNER, GITHUB_REPO)
            .get(number)
            .await?;
        Ok(iid)
    }

    pub async fn add_comment_to_issue(
        &self,
        number: u64,
        body: &str,
    ) -> Result<Comment, OctocrabError> {
        let iid = self
            .inner
            .issues(GITHUB_OWNER, GITHUB_REPO)
            .create_comment(number, body)
            .await?;
        Ok(iid)
    }

    pub async fn list_pull_requests(&self) -> Result<Vec<PullRequest>, OctocrabError> {
        let iid = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .list()
            .state(State::Open)
            .send()
            .await?;
        Ok(iid.into_iter().collect())
    }

    pub async fn create_commit_in_branch(
        &self,
        branch_name: String,
        commit_body: String,
    ) -> Result<octocrab::models::commits::Comment, OctocrabError> {
        let iid = self
            .inner
            .commits(GITHUB_OWNER, GITHUB_REPO)
            .create_comment(branch_name, commit_body)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn list_branches(&self) -> Result<Vec<Branch>, OctocrabError> {
        let iid = self
            .inner
            .repos(GITHUB_OWNER, GITHUB_REPO)
            .list_branches()
            .send()
            .await?;
        Ok(iid.items)
    }

    /// creates new branch under head on github
    /// you should use build_create_ref_request function to construct request
    pub async fn create_branch(&self, request: Request<String>) -> Result<bool, OctocrabError> {
        match self.inner.execute(request).await {
            Ok(_) => {}
            Err(e) => {
                println!("Error creating branch: {:?}", e);
                return Ok(false);
            }
        };
        Ok(true)
    }

    pub async fn list_pull_request(&self, number: u64) -> Result<PullRequest, OctocrabError> {
        let iid = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .get(number)
            .await?;
        Ok(iid)
    }

    pub async fn create_pull_request(
        &self,
        title: &str,
        head: &str,
        body: impl Into<String>,
    ) -> Result<PullRequest, OctocrabError> {
        let iid = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .create(title, head, "main")
            .body(body)
            .maintainer_can_modify(true)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn update_pull_request(
        &self,
        body: &str,
        number: u64,
    ) -> Result<PullRequest, OctocrabError> {
        let iid = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .update(number)
            .body(body)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn add_file(
        &self,
        path: &str,
        content: &str,
        message: &str,
        branch: &str,
    ) -> Result<FileUpdate, OctocrabError> {
        let iid = self
            .inner
            .repos(GITHUB_OWNER, GITHUB_REPO)
            .create_file(path, message, content)
            .branch(branch)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn get_pull_request_by_number(
        &self,
        number: u64,
    ) -> Result<octocrab::models::pulls::PullRequest, OctocrabError> {
        let iid = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .get(number)
            .await?;
        Ok(iid)
    }

    pub async fn get_file(
        &self,
        path: &str,
        branch: &str,
    ) -> Result<ContentItems, octocrab::Error> {
        let iid = self
            .inner
            .repos(GITHUB_OWNER, GITHUB_REPO)
            .get_content()
            .r#ref(branch)
            .path(path)
            .send()
            .await;
        iid
    }

    pub async fn update_file_content(
        &self,
        path: &str,
        message: &str,
        content: &str,
        branch: &str,
        file_sha: &str,
    ) -> Result<FileUpdate, octocrab::Error> {
        let iid = self
            .inner
            .repos(GITHUB_OWNER, GITHUB_REPO)
            .update_file(path, message, content, file_sha)
            .branch(branch)
            .send()
            .await?;
        Ok(iid)
    }

    pub fn build_governance_review_branch(name: String) -> Result<Request<String>, http::Error> {
        Ok(Request::builder()
            .method("POST")
            .uri(format!(
                "https://api.github.com/repos/{}/{}/git/refs",
                GITHUB_OWNER, GITHUB_REPO
            ))
            .body(format!(
                r#"{{"ref": "refs/heads/{}/allocation","sha": "a9f99d9fc56cae689a0bf0ee177c266287eb48cd"}}"#,
                name
            ))?)
    }

    pub fn build_create_ref_request(
        name: String,
        head_hash: Option<String>,
    ) -> Result<Request<String>, http::Error> {
        let hash = match head_hash {
            Some(hash) => hash,
            None => MAIN_BRANCH_HASH.to_string(),
        };
        let request = Request::builder()
            .method("POST")
            .uri(format!(
                "https://api.github.com/repos/{}/{}/git/refs",
                GITHUB_OWNER, GITHUB_REPO
            ))
            .body(format!(
                r#"{{"ref": "refs/heads/{}","sha": "{}" }}"#,
                name, hash
            ))?;
        Ok(request)
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

    pub async fn create_issue(&self, title: &str, body: &str) -> Result<Issue, OctocrabError> {
        Ok(self
            .inner
            .issues(GITHUB_OWNER, GITHUB_REPO)
            .create(title)
            .body(body)
            .send()
            .await?)
    }

    pub async fn close_issue(&self, issue_number: u64) -> Result<Issue, OctocrabError> {
        Ok(self
            .inner
            .issues(GITHUB_OWNER, GITHUB_REPO)
            .update(issue_number)
            .state(IssueState::Closed)
            .send()
            .await?)
    }

    pub async fn get_pull_request_by_head(
        &self,
        head: &str,
    ) -> Result<Vec<PullRequest>, OctocrabError> {
        let mut pull_requests: Page<octocrab::models::pulls::PullRequest> = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .list()
            .state(State::Open)
            .head(head)
            .per_page(1)
            .send()
            .await?;
        let pull_requests_vec: Vec<PullRequest> = pull_requests.take_items();
        Ok(pull_requests_vec)
    }

    pub async fn close_pull_request(&self, number: u64) -> Result<PullRequest, OctocrabError> {
        Ok(self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .update(number)
            .state(PullState::Closed)
            .send()
            .await?)
    }

    pub async fn create_merge_request(
        &self,
        data: CreateMergeRequestData,
    ) -> Result<(PullRequest, String), OctocrabError> {
        let CreateMergeRequestData {
            application_id,
            ref_request,
            owner_name,
            file_content,
            commit,
        } = data;
        let pull_request_data = LDNPullRequest::load(&*application_id, &owner_name);
        let _create_branch_res = self.create_branch(ref_request).await?;
        let add_file_res = self
            .add_file(
                &pull_request_data.path,
                &file_content,
                &commit,
                &pull_request_data.branch_name,
            )
            .await?;
        let file_sha = add_file_res.content.sha;
        let pr = self
            .create_pull_request(
                &pull_request_data.title,
                &pull_request_data.branch_name,
                &pull_request_data.body,
            )
            .await?;

        Ok((pr, file_sha))
    }

    pub async fn merge_pull_request(&self, number: u64) -> Result<(), OctocrabError> {
        let _merge_res = self
            .inner
            .pulls(GITHUB_OWNER, GITHUB_REPO)
            .merge(number)
            .send()
            .await?;
        Ok(())
    }
}

// pub async fn create_merge_request_for_existing_branch(
//     &self,
//     issue_number: u64,
//     owner_name: String,
//     file_content: String,
//     file_sha: String,
// ) -> Result<(PullRequest, String), OctocrabError> {
//     let pull_request_data = LDNPullRequest::load(issue_number, owner_name);
//     let add_file_res = self
//         .update_file_content(
//             &pull_request_data.path,
//             "Start Signing Process",
//             &file_content,
//             &pull_request_data.branch_name,
//             &file_sha,
//         )
//         .await?;
//     let file_sha = add_file_res.content.sha;
//     let pr = self
//         .create_pull_request(
//             &pull_request_data.title,
//             &pull_request_data.branch_name,
//             &pull_request_data.body,
//         )
//         .await?;

//     Ok((pr, file_sha))
// }
