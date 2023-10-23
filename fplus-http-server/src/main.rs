use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger;
use std::sync::Mutex;

pub(crate) mod router;

/// Http Server Setup
/// Exposes Application and Blockchain endpoints
/// Application endpoints:
///    - Create Application
///    - Trigger Application
///    - Propose Application
///    - Approve Application
///    - Merge Application
///    - Get All Merged Applications
///    - Get Application
///    - Get All Applications
/// Blockchain endpoints:
///   - Address Allowance
///   - Verified Clients
#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let client = match fplus_database::core::setup::setup().await {
        Ok(client) => client,
        Err(e) => panic!("Error setting up database: {}", e),
    };

    let db_connection = web::Data::new(Mutex::new(client));
    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .app_data(db_connection.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .service(router::health)
            .service(router::application::create_application)
            .service(router::application::trigger_application)
            .service(router::application::propose_application)
            .service(router::application::approve_application)
            .service(router::application::get_merged_applications)
            .service(router::application::get_application)
            .service(router::application::get_all_applications)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
