#![allow(dead_code)]
use http::header::USER_AGENT;
use http::{Request, Uri};
use hyper_rustls::HttpsConnectorBuilder;
use markdown::{mdast::Node, to_mdast, ParseOptions};

use crate::core::{LDNPullRequest, ParsedApplicationDataFields};
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

struct GithubParams<'a> {
    pub owner: &'a str,
    pub repo: &'a str,
    pub app_id: u64,
    pub installation_id: u64,
    pub main_branch_hash: &'a str,
}

impl GithubParams<'static> {
    fn test_env() -> Self {
        Self {
            owner: "filecoin-project",
            repo: "filplus-tooling-backend-test",
            app_id: 373258,
            installation_id: 40514592,
            main_branch_hash: "650a0aec11dc1cc436a45b316db5bb747e518514",
        }
    }
}

#[derive(Debug)]
pub struct CreateMergeRequestData {
    pub application_id: String,
    pub owner_name: String,
    pub ref_request: Request<String>,
    pub file_content: String,
    pub commit: String,
}

#[derive(Debug)]
pub struct GithubWrapper<'a> {
    pub inner: Arc<Octocrab>,
    pub owner: &'a str,
    pub repo: &'a str,
    pub main_branch_hash: &'a str,
}

impl GithubWrapper<'static> {
    pub fn new() -> Self {
        let GithubParams {
            owner,
            repo,
            app_id,
            installation_id,
            main_branch_hash,
        } = GithubParams::test_env();
        let connector = HttpsConnectorBuilder::new()
            .with_native_roots() // enabled the `rustls-native-certs` feature in hyper-rustls
            .https_only()
            .enable_http1()
            .build();

        let client = hyper::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(15))
            .build(connector);
        let key =
            jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("../../gh-private-key.pem"))
                .unwrap();
        let octocrab = OctocrabBuilder::new_empty()
            .with_service(client)
            .with_layer(&BaseUriLayer::new(Uri::from_static(GITHUB_API_URL)))
            .with_layer(&ExtraHeadersLayer::new(Arc::new(vec![(
                USER_AGENT,
                "octocrab".parse().unwrap(),
            )])))
            .with_auth(AuthState::App(AppAuth {
                app_id: app_id.into(),
                key,
            }))
            .build()
            .expect("Could not create Octocrab instance");
        let iod: InstallationId = installation_id.try_into().expect("Invalid installation id");
        let installation = octocrab.installation(iod);
        Self {
            main_branch_hash,
            owner,
            repo,
            inner: Arc::new(installation),
        }
    }

    pub async fn list_issues(&self) -> Result<Vec<Issue>, OctocrabError> {
        let iid = self
            .inner
            .issues(self.owner, self.repo)
            .list()
            .state(State::Open)
            .send()
            .await?;
        Ok(iid.into_iter().map(|i: Issue| i).collect())
    }

    pub async fn list_issue(&self, number: u64) -> Result<Issue, OctocrabError> {
        let iid = self.inner.issues(self.owner, self.repo).get(number).await?;
        Ok(iid)
    }

    pub async fn add_comment_to_issue(
        &self,
        number: u64,
        body: &str,
    ) -> Result<Comment, OctocrabError> {
        let iid = self
            .inner
            .issues(self.owner, self.repo)
            .create_comment(number, body)
            .await?;
        Ok(iid)
    }

    pub async fn list_pull_requests(&self) -> Result<Vec<PullRequest>, OctocrabError> {
        let iid = self
            .inner
            .pulls(self.owner, self.repo)
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
            .commits(self.owner, self.repo)
            .create_comment(branch_name, commit_body)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn get_pull_request_files(
        &self,
        number: u64,
    ) -> Result<Vec<octocrab::models::pulls::FileDiff>, OctocrabError> {
        let iid: Page<octocrab::models::pulls::FileDiff> = self
            .inner
            .pulls(self.owner, self.repo)
            .media_type(octocrab::params::pulls::MediaType::Full)
            .list_files(number)
            .await?;
        Ok(iid.items.into_iter().map(|i| i.into()).collect())
    }

    pub async fn list_branches(&self) -> Result<Vec<Branch>, OctocrabError> {
        let iid = self
            .inner
            .repos(self.owner, self.repo)
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

    /// remove branch from github
    /// you should use build_remove_ref_request function to construct request
    pub async fn remove_branch(&self, request: Request<String>) -> Result<bool, OctocrabError> {
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
        let iid = self.inner.pulls(self.owner, self.repo).get(number).await?;
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
            .pulls(self.owner, self.repo)
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
            .pulls(self.owner, self.repo)
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
            .repos(self.owner, self.repo)
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
        let iid = self.inner.pulls(self.owner, self.repo).get(number).await?;
        Ok(iid)
    }

    pub async fn get_file(
        &self,
        path: &str,
        branch: &str,
    ) -> Result<ContentItems, octocrab::Error> {
        let iid = self
            .inner
            .repos(self.owner, self.repo)
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
            .repos(self.owner, self.repo)
            .update_file(path, message, content, file_sha)
            .branch(branch)
            .send()
            .await?;
        Ok(iid)
    }

    pub fn build_remove_ref_request(&self, name: String) -> Result<Request<String>, http::Error> {
        let request = Request::builder()
            .method("DELETE")
            .uri(format!(
                "https://api.github.com/repos/{}/{}/git/refs/heads/{}",
                self.owner, self.repo, name
            ))
            .body("".to_string())?;
        Ok(request)
    }

    pub fn build_create_ref_request(
        &self,
        name: String,
        head_hash: Option<String>,
    ) -> Result<Request<String>, http::Error> {
        let hash = match head_hash {
            Some(hash) => hash,
            None => self.main_branch_hash.to_string(),
        };
        let request = Request::builder()
            .method("POST")
            .uri(format!(
                "https://api.github.com/repos/{}/{}/git/refs",
                self.owner, self.repo
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
            .issues(self.owner, self.repo)
            .create(title)
            .body(body)
            .send()
            .await?)
    }

    pub async fn close_issue(&self, issue_number: u64) -> Result<Issue, OctocrabError> {
        Ok(self
            .inner
            .issues(self.owner, self.repo)
            .update(issue_number)
            .state(IssueState::Closed)
            .send()
            .await?)
    }

    pub async fn get_all_pull_requests(&self) -> Result<Vec<u64>, OctocrabError> {
        let mut pull_requests: Page<octocrab::models::pulls::PullRequest> = self
            .inner
            .pulls(self.owner, self.repo)
            .list()
            .state(State::Open)
            .base("main")
            .send()
            .await?;
        let pull_requests_vec: Vec<u64> = pull_requests
            .take_items()
            .into_iter()
            .map(|pr| pr.number)
            .collect();
        Ok(pull_requests_vec)
    }

    pub async fn get_pull_request_by_head(
        &self,
        head: &str,
    ) -> Result<Vec<PullRequest>, OctocrabError> {
        let mut pull_requests: Page<octocrab::models::pulls::PullRequest> = self
            .inner
            .pulls(self.owner, self.repo)
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
            .pulls(self.owner, self.repo)
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
            .pulls(self.owner, self.repo)
            .merge(number)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TestPatch {
    pub hello: String,
    pub second: String,
    pub third: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ParsedTestPatch {
    Hello,
    Second,
    Third,
}

impl From<String> for ParsedTestPatch {
    fn from(s: String) -> Self {
        dbg!(&s);
        match s.as_str() {
            "Hello" => ParsedTestPatch::Hello,
            "Second" => ParsedTestPatch::Second,
            "Third" => ParsedTestPatch::Third,
            _ => panic!("Unknown property"),
        }
    }
}

pub fn parser(body: &str) -> TestPatch {
    let tree: Node = to_mdast(body, &ParseOptions::default()).unwrap();
    let mut hello: Option<String> = None;
    let mut second: Option<String> = None;
    let mut third: Option<String> = None;
    // let mut custom_notary: Option<String> = None;
    for (index, i) in tree.children().unwrap().into_iter().enumerate().step_by(2) {
        let prop: ParsedTestPatch = i.to_string().into();
        let tree = tree.children().unwrap().into_iter();
        let value = match tree.skip(index + 1).next() {
            Some(v) => v.to_string(),
            None => continue,
        };
        match prop {
            ParsedTestPatch::Hello => {
                hello = Some(value);
            }
            ParsedTestPatch::Second => {
                second = Some(value);
            }
            ParsedTestPatch::Third => {
                third = Some(value);
            }
        }
    }
    let parsed_ldn = TestPatch {
        hello: hello.unwrap_or_else(|| "No Name".to_string()),
        second: second.unwrap_or_else(|| "No Region".to_string()),
        third: third.unwrap_or_else(|| "No Region".to_string()),
    };
    parsed_ldn
}

fn remove_invalid_chars(mut s: String) -> String {
    s.retain(|x| !['+', '\n', '\'', '-'].contains(&x));
    s
}

fn http_server() -> reqwest::Client {
    let client = reqwest::Client::builder()
        .user_agent("FP-CORE/0.1.0")
        .connection_verbose(true)
        .build()
        .expect("Failed to build client");
    client
}

#[cfg(test)]
mod tests {
    use crate::{
        core::application::ApplicationFile,
        external_services::github::{http_server, remove_invalid_chars, GithubWrapper, TestPatch},
    };

    #[ignore]
    #[tokio::test]
    async fn test_basic_integration() {
        let gh = GithubWrapper::new();
        let mut pull_requests: Vec<u64> = gh.get_all_pull_requests().await.unwrap();
        dbg!(&pull_requests);
        let mut ret: Vec<ApplicationFile> = vec![];
        while let Some(pr_number) = pull_requests.pop() {
            let files = gh.get_pull_request_files(pr_number).await.unwrap();
            let blolb_url = match files.get(0) {
                Some(file) => file.raw_url.clone(),
                None => continue,
            };
            let scheme = blolb_url.scheme();
            let host = match blolb_url.host_str() {
                Some(host) => host,
                None => continue,
            };
            let path = blolb_url.path();
            let url = format!("{}://{}{}", scheme, host, path);
            let client = http_server();
            let res = match client.get(&url).send().await {
                Ok(res) => res,
                Err(_) => {
                    continue;
                }
            };
            let res = match res.text().await {
                Ok(res) => res,
                Err(_) => {
                    continue;
                }
            };
            let res: ApplicationFile = match serde_json::from_str(&res) {
                Ok(res) => res,
                Err(_) => {
                    continue;
                }
            };
            ret.push(res);
        }
        dbg!(&ret);
        // // dbg!(&files.get(0).unwrap().blob_url);
        // dbg!(&files.get(0).unwrap().patch);
        // reqwest.
        // let url = format!(blolb_url.schema);
        // let mut patch = files.get(0).unwrap().patch.clone().unwrap();
        // let offset = patch.find("{").unwrap();
        // let deleted = patch.drain(..offset).collect::<String>();
        // // dbg!(&deleted);
        // // dbg!(&patch);
        // // dbg!(&patch);
        // let parsed = remove_invalid_chars(patch);
        // dbg!(&parsed);
        // let test_patch: TestPatch = serde_json::from_str(&parsed).unwrap();
        // dbg!(&test_patch);
        // dbg!(&test_patch);
        // assert!(false);
        // assert!(gh.list_issues().await.is_ok());
        // assert!(gh.list_pull_requests().await.is_ok());
        // assert!(gh.list_branches().await.is_ok());
    }
}
