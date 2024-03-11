#![allow(dead_code)]
use fplus_database::database::allocators::get_allocator;
use http::header::USER_AGENT;
use http::{Request, Uri};
use hyper_rustls::HttpsConnectorBuilder;

use octocrab::auth::AppAuth;
use octocrab::models::issues::{Comment, Issue};
use octocrab::models::pulls::PullRequest;
use octocrab::models::repos::{Branch, ContentItems, FileDeletion, FileUpdate, secrets::CreateRepositorySecret};
use octocrab::models::{InstallationId, IssueState, Label};
use octocrab::params::{pulls::State as PullState, State};
use octocrab::service::middleware::base_uri::BaseUriLayer;
use octocrab::service::middleware::extra_headers::ExtraHeadersLayer;
use octocrab::{AuthState, Error as OctocrabError, Octocrab, OctocrabBuilder, Page};
use libsodium_sys as sodium;

use base64::{encode, decode};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::get_env_var_or_default;
use crate::core::application::file::AppState;

const GITHUB_API_URL: &str = "https://api.github.com";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RefObject {
    sha: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RefData {
    #[serde(rename = "ref")]
    _ref: String,
    object: RefObject,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct RefList(pub Vec<RefData>);

struct GithubParams {
    pub owner: String,
    pub repo: String,
    pub app_id: u64,
    pub installation_id: u64,
}

#[derive(Debug)]
pub struct CreateRefillMergeRequestData {
    pub application_id: String,
    pub owner_name: String,
    pub ref_request: Request<String>,
    pub file_content: String,
    pub file_name: String,
    pub branch_name: String,
    pub commit: String,
    pub file_sha: String,
}

#[derive(Debug)]
pub struct CreateMergeRequestData {
    pub application_id: String,
    pub owner_name: String,
    pub ref_request: Request<String>,
    pub file_content: String,
    pub file_name: String,
    pub branch_name: String,
    pub commit: String,
}

#[derive(Debug)]
pub struct GithubWrapper {
    pub inner: Arc<Octocrab>,
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CommitData {
    commit: Commit,
}

#[derive(Debug, Deserialize, Serialize)]
struct Commit {
    author: Author,
}

#[derive(Debug, Deserialize, Serialize)]
struct Author {
    name: String,
}

pub async fn github_async_new(owner: String, repo: String) -> GithubWrapper {
    let allocator_result = get_allocator(owner.as_str(), repo.as_str()).await;
    
    let allocator = match allocator_result {
        Ok(allocator) => allocator,
        Err(e) => {
            log::error!("Failed to get allocator from database: {:?}", e);
            std::process::exit(1);
        }
    };

    let installation_id = allocator.unwrap().installation_id.unwrap_or(0).to_string();

    return GithubWrapper::new(owner, repo, installation_id);
}

impl GithubWrapper {
    pub fn new(owner: String, repo: String, installation_id_str: String) -> Self {
        let app_id_str = get_env_var_or_default("GITHUB_APP_ID");
    
        let app_id = app_id_str.parse::<u64>().unwrap_or_else(|_| {
            log::error!("Failed to parse GITHUB_APP_ID as u64, using default value");
            0
        });
    
        let installation_id = installation_id_str.parse::<u64>().unwrap_or_else(|_| {
            log::error!("Failed to parse GITHUB_INSTALLATION_ID as u64, using default value");
            0
        });
    
        let gh_private_key = std::env::var("GH_PRIVATE_KEY").unwrap_or_else(|_| {
            log::warn!("GH_PRIVATE_KEY not found in .env file, attempting to read from gh-private-key.pem");
            std::fs::read_to_string("gh-private-key.pem").unwrap_or_else(|e| {
                log::error!("Failed to read gh-private-key.pem. Error: {:?}", e);
                std::process::exit(1);
            })
        });

        let connector = HttpsConnectorBuilder::new()
            .with_native_roots() // enabled the `rustls-native-certs` feature in hyper-rustls
            .https_only()
            .enable_http1()
            .build();

        let client = hyper::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(15))
            .build(connector);
        dbg!(gh_private_key.clone());
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(gh_private_key.as_bytes()).unwrap();
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
            owner,
            repo,
            inner: Arc::new(installation),
        }
    }

    pub async fn list_issues(&self) -> Result<Vec<Issue>, OctocrabError> {
        let iid = self
            .inner
            .issues(&self.owner, &self.repo)
            .list()
            .state(State::Open)
            .send()
            .await?;
        Ok(iid.into_iter().map(|i: Issue| i).collect())
    }

    pub async fn list_issue(&self, number: u64) -> Result<Issue, OctocrabError> {
        let iid = self
            .inner
            .issues(&self.owner, &self.repo)
            .get(number)
            .await?;
        println!("{:?}", iid);
        Ok(iid)
    }

    pub async fn add_comment_to_issue(
        &self,
        number: u64,
        body: &str,
    ) -> Result<Comment, OctocrabError> {
        let iid = self
            .inner
            .issues(&self.owner, &self.repo)
            .create_comment(number, body)
            .await?;
        Ok(iid)
    }

    pub async fn replace_issue_labels(
        &self,
        number: u64,
        labels: &[String],
    ) -> Result<Vec<Label>, OctocrabError> {
        let iid = self
            .inner
            .issues(&self.owner, &self.repo)
            .replace_all_labels(number, labels)
            .await?;
        Ok(iid)
    }

    // the comment param is in case we want to add an 'error' comment as well to the issue later on, I can remove it if not necessary
    pub async fn add_error_label(
        &self,
        number: u64,
        _comment: String
    ) -> Result<(), OctocrabError> {
        self
            .inner
            .issues(&self.owner, &self.repo)
            .add_labels(number, &[AppState::Error.as_str().to_string()])
            .await?;
        
        Ok(())
    }

    pub async fn update_issue_labels(
        &self,
        number: u64,
        new_labels: &[&str],
    ) -> Result<(), OctocrabError> {
        let search_labels = vec![
            "waiting for governance review",
            AppState::Submitted.as_str(),
            AppState::ReadyToSign.as_str(),
            AppState::StartSignDatacap.as_str(),
            AppState::Granted.as_str(),
            AppState::TotalDatacapReached.as_str(),
        ];
    
        let issue = self.list_issue(number).await?;
    
        let labels_to_keep: Vec<String> = issue
            .labels
            .iter()
            .filter(|label| !search_labels.contains(&label.name.as_str()))
            .map(|label| label.name.clone())
            .collect();
    
        self.replace_issue_labels(number, &labels_to_keep).await?;
    
        let new_labels: Vec<String> = new_labels.iter().map(|&s| s.to_string()).collect();
        self
            .inner
            .issues(&self.owner, &self.repo)
            .add_labels(number, &new_labels)
            .await?;
    
        Ok(())
    }
    

    pub async fn list_pull_requests(&self) -> Result<Vec<PullRequest>, OctocrabError> {
        let iid = self
            .inner
            .pulls(&self.owner, &self.repo)
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
            .commits(&self.owner, &self.repo)
            .create_comment(branch_name, commit_body)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn get_pull_request_files(
        &self,
        pr_number: u64,
    ) -> Result<(u64, Vec<octocrab::models::pulls::FileDiff>), OctocrabError> {
        let iid: Page<octocrab::models::pulls::FileDiff> = self
            .inner
            .pulls(&self.owner, &self.repo)
            .media_type(octocrab::params::pulls::MediaType::Full)
            .list_files(pr_number)
            .await?;
        Ok((pr_number, iid.items.into_iter().map(|i| i.into()).collect()))
    }

    pub async fn get_last_modification_date(&self, path: &str) -> Result<chrono::DateTime<chrono::Utc>, OctocrabError> {
        let commits = self
            .inner
            .repos(&self.owner, &self.repo)
            .list_commits()
            .path(path)
            .per_page(1)
            .send()
            .await?;

        match commits.items.into_iter().next() {
            Some(commit) => {
                match commit.commit.author.and_then(|author| author.date) {
                    Some(date) => Ok(date),
                    None => Ok(chrono::Utc::now()),
                }
            },
            None => Ok(chrono::Utc::now()),
        }
    }

    pub async fn list_branches(&self) -> Result<Vec<Branch>, OctocrabError> {
        let iid = self
            .inner
            .repos(&self.owner, &self.repo)
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
        let iid = self
            .inner
            .pulls(&self.owner, &self.repo)
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
            .pulls(&self.owner, &self.repo)
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
            .pulls(&self.owner, &self.repo)
            .update(number)
            .body(body)
            .send()
            .await?;
        Ok(iid)
    }

    pub async fn delete_file(
        &self,
        path: &str,
        branch: &str,
        message: &str,
        sha: &str,
    ) -> Result<FileDeletion, OctocrabError> {
        let iid = self
            .inner
            .repos(&self.owner, &self.repo)
            .delete_file(path, message, sha)
            .branch(branch)
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
            .repos(&self.owner, &self.repo)
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
            .pulls(&self.owner, &self.repo)
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
            .repos(&self.owner, &self.repo)
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
            .repos(&self.owner, &self.repo)
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

    pub async fn get_main_branch_sha(&self) -> Result<String, http::Error> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/refs",
            self.owner, self.repo
        );
        let request = http::request::Builder::new()
            .method(http::Method::GET)
            .uri(url);
        let request = self.inner.build_request::<String>(request, None).unwrap();

        let mut response = match self.inner.execute(request).await {
            Ok(r) => r,
            Err(e) => {
                println!("Error getting main branch sha: {:?}", e);
                return Ok("".to_string());
            }
        };
        let response = response.body_mut();
        let body = hyper::body::to_bytes(response).await.unwrap();
        let shas = body.into_iter().map(|b| b as char).collect::<String>();
        let shas: RefList = serde_json::from_str(&shas).unwrap();
        for sha in shas.0 {
            if sha._ref == "refs/heads/main" {
                return Ok(sha.object.sha);
            }
        }
        Ok("".to_string())
    }

    pub fn build_create_ref_request(
        &self,
        name: String,
        head_hash: String,
    ) -> Result<Request<String>, http::Error> {
        let request = Request::builder()
            .method("POST")
            .uri(format!(
                "https://api.github.com/repos/{}/{}/git/refs",
                self.owner, self.repo
            ))
            .body(format!(
                r#"{{"ref": "refs/heads/{}","sha": "{}" }}"#,
                name, head_hash
            ))?;
        Ok(request)
    }

    pub async fn create_issue(&self, title: &str, body: &str) -> Result<Issue, OctocrabError> {
        Ok(self
            .inner
            .issues(&self.owner, &self.repo)
            .create(title)
            .body(body)
            .send()
            .await?)
    }

    pub async fn close_issue(&self, issue_number: u64) -> Result<Issue, OctocrabError> {
        Ok(self
            .inner
            .issues(&self.owner, &self.repo)
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
            .pulls(&self.owner, &self.repo)
            .list()
            .state(State::Open)
            .head(format!("{}:{}", self.owner.clone(), head))
            .per_page(1)
            .send()
            .await?;
        let pull_requests_vec: Vec<PullRequest> = pull_requests.take_items();
        Ok(pull_requests_vec)
    }

    pub async fn close_pull_request(&self, number: u64) -> Result<PullRequest, OctocrabError> {
        Ok(self
            .inner
            .pulls(&self.owner, &self.repo)
            .update(number)
            .state(PullState::Closed)
            .send()
            .await?)
    }

    pub async fn create_refill_merge_request(
        &self,
        data: CreateRefillMergeRequestData,
    ) -> Result<(PullRequest, String), OctocrabError> {
        let CreateRefillMergeRequestData {
            application_id: _,
            ref_request,
            owner_name,
            file_content,
            file_name,
            branch_name,
            commit,
            file_sha,
        } = data;
        let _create_branch_res = self.create_branch(ref_request).await?;
        self.update_file_content(&file_name, &commit, &file_content, &branch_name, &file_sha)
            .await?;
        let pr = self
            .create_pull_request(
                &format!("Datacap for {}", owner_name),
                &branch_name,
                &format!("BODY"),
            )
            .await?;

        Ok((pr, file_sha))
    }

    pub async fn create_merge_request(
        &self,
        data: CreateMergeRequestData,
    ) -> Result<(PullRequest, String), OctocrabError> {
        let CreateMergeRequestData {
            application_id: _,
            ref_request,
            owner_name,
            file_content,
            file_name,
            branch_name,
            commit,
        } = data;
        let _create_branch_res = self.create_branch(ref_request).await?;
        let add_file_res = self
            .add_file(&file_name, &file_content, &commit, &branch_name)
            .await?;
        let file_sha = add_file_res.content.sha;
        let pr = self
            .create_pull_request(
                &format!("Datacap for {}", owner_name),
                &branch_name,
                &format!("BODY"),
            )
            .await?;

        Ok((pr, file_sha))
    }

    pub async fn merge_pull_request(&self, number: u64) -> Result<(), OctocrabError> {
        let _merge_res = self
            .inner
            .pulls(&self.owner, &self.repo)
            .merge(number)
            .send()
            .await?;
        Ok(())
    }

    // If provided with empty string, will take all files from root
    pub async fn get_files(&self, path: &str) -> Result<ContentItems, OctocrabError> {
        let contents_items = self
            .inner
            .repos(&self.owner, &self.repo)
            .get_content()
            .path(path)
            .r#ref("main")
            .send()
            .await?;

        Ok(contents_items)
    }

    pub async fn get_all_files_from_branch(
        &self,
        branch: &str,
    ) -> Result<ContentItems, OctocrabError> {
        let contents_items = self
            .inner
            .repos(&self.owner, &self.repo)
            .get_content()
            .r#ref(branch)
            .send()
            .await?;

        Ok(contents_items)
    }

    pub async fn get_last_commit_author(&self, pr_number: u64) -> Result<String, http::Error> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/commits",
            self.owner, self.repo, pr_number
        );

        let request = http::request::Builder::new()
            .method(http::Method::GET)
            .uri(url);

        let request = self.inner.build_request::<String>(request, None).unwrap();

        let mut response = match self.inner.execute(request).await {
            Ok(r) => r,
            Err(e) => {
                println!("Error fetching last commit author: {:?}", e);
                return Ok("".to_string());
            }
        };

        let response_body = response.body_mut();
        let body = hyper::body::to_bytes(response_body).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let commits: Vec<CommitData> = serde_json::from_str(&body_str).unwrap();

        let last_commit: &CommitData = commits.last().unwrap();
        let author = last_commit.commit.author.name.clone();

        Ok(author)
    }

    pub async fn get_branch_name_from_pr(&self, pr_number: u64) -> Result<String, OctocrabError> {
        let pull_request = self
            .inner
            .pulls(&self.owner, &self.repo)
            .get(pr_number)
            .await?;
        Ok(pull_request.head.ref_field.clone())
    }

    pub async fn get_files_from_public_repo(
        &self,
        owner: &str, 
        repo: &str, 
        path: Option<&str>
    ) -> Result<ContentItems, OctocrabError> {
        let branch = "main";
        let octocrab = Octocrab::builder().build()?;
        let gh = octocrab.repos(owner, repo);

        //if path is not provided, take all files from root
        let contents_items = match path {
            Some(p) => gh.get_content().r#ref(branch).path(p).send().await?,
            None => gh.get_content().r#ref(branch).send().await?,
        };
        
        Ok(contents_items)
    }

    /**
     * Create or update a secret in the repository
     * This function will receive the secret name and value and will create or update the secret in the repository
     * The secret value will be encrypted using the public key of the repository, as required by the GitHub API
     * More information here: https://docs.github.com/en/rest/actions/secrets?apiVersion=2022-11-28#create-or-update-a-repository-secret
     */
    pub async fn create_or_update_secret(
        &self,
        secret_name: &str,
        secret_value: &str,
    ) -> Result<(), OctocrabError> {
        let pk = self
            .inner
            .repos(&self.owner, &self.repo)
            .secrets()
            .get_public_key()
            .await?;
    
        let pk_bytes = match decode(pk.key) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("Failed to decode public key: {:?}", e);
                return Ok(());
            }
        };
        assert_eq!(pk_bytes.len(), sodium::crypto_box_PUBLICKEYBYTES as usize, "Invalid public key length");
        //Create a buffer to store the encrypted secret
        let mut encrypted_secret = vec![0u8; sodium::crypto_box_SEALBYTES as usize + secret_value.len()];
        //Encrypt the secret using the public key
        let _encrypt_res = unsafe {
            sodium::crypto_box_seal(
                encrypted_secret.as_mut_ptr(),
                secret_value.as_ptr(),
                secret_value.len() as u64,
                pk_bytes.as_ptr(),
            )
        };
        //Encode the encrypted secret to base64
        let encrypted_secret_base64 = encode(&encrypted_secret);

        //Encrypt using libsodium
        let _create_secret_res = self
            .inner
            .repos(&self.owner, &self.repo)
            .secrets()
            .create_or_update_secret(secret_name, &CreateRepositorySecret{
                key_id: &pk.key_id,
                encrypted_value: &encrypted_secret_base64,
            })
            .await?;
        Ok(())
    }
}
