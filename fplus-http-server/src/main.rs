use env_logger;
use log::info;
mod middleware;
use middleware::{verifier_auth::VerifierAuth, rkh_auth::RKHAuth};
pub(crate) mod router;
use std::env;

use actix_web::{
    App,
    HttpServer,
    web,
    middleware::Logger,
};

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
                web::scope("/verifier")
                    .wrap(VerifierAuth) // Apply GitHubAuth to all routes under "/api"
                    .service(router::application::trigger)
                    .service(router::application::propose)
                    .service(router::application::approve)
            )
            .service(
                web::scope("/rkh")
                .wrap(RKHAuth)
                .service(router::application::merged)
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
            .service(router::application::validate_application_merge)
            .service(router::application::cache_renewal)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
            .service(router::verifier::verifiers)
            .service(router::allocator::allocators)
            .service(router::allocator::allocator)
            .service(router::allocator::delete)
            .service(router::allocator::create_from_json)
            .service(router::rkh::fetch_nonce)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
