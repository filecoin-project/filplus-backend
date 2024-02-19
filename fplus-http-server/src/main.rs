use std::env;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use env_logger;
use log::info;
use fplus_database;
pub(crate) mod router;

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
            .service(router::application::trigger)
            .service(router::application::propose)
            .service(router::application::approve)
            .service(router::application::merged)
            .service(router::application::active)
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
