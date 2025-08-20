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
        let method = req.method();
        let path = req.path();
        let request_info = format!("{method} {path}?{query_string}");
        let query: Result<web::Query<RepoQuery>, _> = web::Query::from_query(query_string);
        let RepoQuery {
            owner,
            repo,
            github_username,
        } = match query {
            Ok(q) => q.into_inner(),
            Err(e) => {
                let err = actix_web::error::ErrorBadRequest(format!(
                    "Wrong query string format error: {e}"
                ));
                log::info!(
                    "{} {}",
                    request_info,
                    err.as_response_error().status_code().as_u16()
                );
                return Box::pin(async { Err(err) });
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
                    .header("Authorization", format!("Bearer {token}"))
                    .header("User-Agent", "Actix-web")
                    .send()
                    .await
                    .map_err(|e| {
                        let err = actix_web::error::ErrorBadRequest(e);
                        log::info!(
                            "{} {}",
                            request_info,
                            err.as_response_error().status_code().as_u16()
                        );
                        err
                    })?;

                if user_info_result.status().is_success() {
                    let user_info =
                        user_info_result
                            .json::<serde_json::Value>()
                            .await
                            .map_err(|_| {
                                let err = actix_web::error::ErrorInternalServerError(
                                    "GitHub handle information not found.",
                                );
                                log::info!(
                                    "{} {}",
                                    request_info,
                                    err.as_response_error().status_code().as_u16()
                                );
                                log::error!("{err}");
                                err
                            })?;

                    if let Some(login) = user_info.get("login").and_then(|v| v.as_str()) {
                        user_handle = login.to_string();
                    } else {
                        let err = actix_web::error::ErrorInternalServerError(
                            "GitHub handle information not found.",
                        );
                        log::info!(
                            "{} {}",
                            request_info,
                            err.as_response_error().status_code().as_u16()
                        );
                        log::error!("{err}");
                        return Err(err);
                    }
                } else {
                    let err = actix_web::error::ErrorUnauthorized("Failed to get GitHub user info");
                    log::info!(
                        "{} {}",
                        request_info,
                        err.as_response_error().status_code().as_u16()
                    );
                    log::error!("{err}");
                    return Err(err);
                }
            }

            if github_username != user_handle {
                let err = actix_web::error::ErrorUnauthorized(
                    "Sent GitHub handle different than auth token owner.",
                );
                log::info!(
                    "{} {}",
                    request_info,
                    err.as_response_error().status_code().as_u16()
                );
                log::error!("{err}");
                return Err(err);
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
                                log::info!("{user_handle} is a verifier.");
                            } else {
                                let err = actix_web::error::ErrorUnauthorized(
                                    "The user is not a verifier.",
                                );
                                log::info!(
                                    "{} {}",
                                    request_info,
                                    err.as_response_error().status_code().as_u16()
                                );
                                log::error!("{err}");
                                return Err(err);
                            }
                        }
                    }
                }
                Err(e) => {
                    let err = actix_web::error::ErrorInternalServerError(e);
                    log::info!(
                        "{} {}",
                        request_info,
                        err.as_response_error().status_code().as_u16()
                    );
                    log::error!("{err}");
                    return Err(err);
                }
            }

            let res = fut.await?;
            Ok(res)
        })
    }
}
