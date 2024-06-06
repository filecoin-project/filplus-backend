use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use reqwest::Client;
use serde::Deserialize;

// Import any other modules that you reference in this file
use fplus_database::database::allocators::get_allocator;
#[derive(Deserialize, Debug)]
struct RepoQuery {
    owner: String,
    repo: String,
    github_username: String,
}

pub struct VerifierAuth;

impl<S, B> Transform<S, ServiceRequest> for VerifierAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = VerifierAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(VerifierAuthMiddleware { service }))
    }
}

pub struct VerifierAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for VerifierAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let query_string = req.query_string();
        let query: Result<web::Query<RepoQuery>, _> = web::Query::from_query(query_string);
        let RepoQuery {
            owner,
            repo,
            github_username,
        } = match query {
            Ok(q) => q.into_inner(),
            Err(e) => {
                println!("{}", e);
                return Box::pin(async {
                    return Err(actix_web::error::ErrorBadRequest(
                        "Wrong query string format",
                    ));
                });
            }
        };

        let auth_header_value = req
            .headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .filter(|hv| hv.starts_with("Bearer "))
            .map(|hv| hv["Bearer ".len()..].to_string());
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut user_handle = String::new();

            if let Some(token) = auth_header_value {
                // Make the asynchronous HTTP request here
                let client = Client::new();
                let user_info_result = client
                    .get("https://api.github.com/user")
                    .header("Authorization", format!("Bearer {}", token))
                    .header("User-Agent", "Actix-web")
                    .send()
                    .await;

                match user_info_result {
                    Ok(response) => {
                        //Raise an actix test error
                        if response.status().is_success() {
                            let user_info = response
                                .json::<serde_json::Value>()
                                .await
                                .expect("Failed to parse JSON");

                            if let Some(login) = user_info.get("login").and_then(|v| v.as_str()) {
                                user_handle = login.to_string();
                            } else {
                                println!("GitHub handle information not found.");
                                return Err(actix_web::error::ErrorInternalServerError(
                                    "GitHub handle information not found.",
                                ));
                            }
                        } else {
                            println!("Failed to get GitHub user info");
                            return Err(actix_web::error::ErrorUnauthorized(
                                "Failed to get GitHub user info.",
                            ));
                        }
                    }
                    Err(e) => {
                        println!("Request error: {:?}", e);
                        return Err(actix_web::error::ErrorBadRequest(e));
                    }
                }
            }

            if github_username != user_handle {
                // comment this for testing
                println!("Sent GitHub handle different than auth token owner.");
                return Err(actix_web::error::ErrorBadRequest(
                    "Sent GitHub handle different than auth token owner.",
                ));
            }

            match get_allocator(&owner, &repo).await {
                Ok(allocator) => {
                    if let Some(allocator) = &allocator {
                        if let Some(verifiers) = &allocator.verifiers_gh_handles {
                            let verifier_handles: Vec<String> = verifiers
                                .split(',')
                                .map(|s| s.trim().to_lowercase())
                                .collect();
                            if verifier_handles.contains(&user_handle.to_lowercase()) {
                                println!("{} is a verifier.", user_handle);
                            } else {
                                println!("The user is not a verifier.");
                                return Err(actix_web::error::ErrorUnauthorized(
                                    "The user is not a verifier.",
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to get allocator: {:?}", e);
                }
            }

            let res = fut.await?;
            return Ok(res);
        })
    }
}
