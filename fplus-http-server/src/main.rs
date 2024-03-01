use env_logger;
use log::info;
use fplus_database::database;
pub(crate) mod router;
use reqwest::Client;
use serde::Deserialize;

use std::{env, future::{ready, Ready}};

use actix_web::{
    App,
    HttpServer,
    web,
    middleware::Logger,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
    HttpResponse,
    http::StatusCode,
};
use futures_util::future::LocalBoxFuture;

#[derive(Deserialize, Debug)]
struct RepoQuery {
    owner: String,
    repo: String,
}

pub struct GHAuth;

impl<S, B> Transform<S, ServiceRequest> for GHAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = GHAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GHAuthMiddleware { service }))
    }
}

pub struct GHAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for GHAuthMiddleware<S>
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
        let RepoQuery { owner, repo } = match query {
            Ok(q) => q.into_inner(),
            Err(_) => {
                return Box::pin(async {
                    return Err(actix_web::error::ErrorBadRequest("Wrong query string format"));
                });
            }
        };

        let auth_header_value = req.headers().get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .filter(|hv| hv.starts_with("Bearer "))
            .map(|hv| hv["Bearer ".len()..].to_string());
        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(token) = auth_header_value {
                // Make the asynchronous HTTP request here
                let client = Client::new();
                let user_info_result = client.get("https://api.github.com/user")
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
                                println!("Login: {}", login);
                            } else {
                                println!("Login information not found.");
                            }
                        } else {
                            println!("Failed to get GitHub user info");
                        }
                    },
                    Err(e) => println!("Request error: {:?}", e),
                }
            }

            match database::get_allocator(&owner, &repo).await {
                Ok(allocator) => {
                    println!("Allocator: {:?}", allocator);
                },
                Err(e) => {
                    println!("Failed to get allocator: {:?}", e);
                }
            }
    
            let res = fut.await?;
            println!("Hi from response");
            Ok(res)
        })
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("Logger initialized at log level: {}", log_level);

    if let Err(e) = fplus_database::setup().await {
        panic!("Failed to setup database connection: {}", e);
    }
    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .service(router::health)
            .service(router::application::create)
            .service(
                web::scope("/api")
                    .wrap(GHAuth) // Apply GitHubAuth to all routes under "/api"
                    .service(router::application::testz)
                    .service(router::application::trigger)
                    .service(router::application::propose)
                    .service(router::application::approve)
            )
            .service(router::application::merged)
            .service(router::application::active)
            .service(router::application::all_applications)
            .service(router::application::refill)
            .service(router::application::total_dc_reached)
            .service(router::application::single)
            .service(router::application::validate_application_flow)
            .service(router::application::validate_application_trigger)
            .service(router::application::validate_application_proposal)
            .service(router::application::validate_application_approval)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
            .service(router::verifier::verifiers)
            .service(router::allocator::allocators)
            .service(router::allocator::allocator)
            .service(router::allocator::delete)
            .service(router::allocator::create_from_json)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
