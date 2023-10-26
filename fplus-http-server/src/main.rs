use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger;
use std::sync::Mutex;

pub(crate) mod router;

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
            .service(router::application::create)
            .service(router::application::trigger)
            .service(router::application::propose)
            .service(router::application::approve)
            .service(router::application::merged)
            .service(router::application::active)
            .service(router::application::refill)
            .service(router::application::total_dc_reached)
            .service(router::application::single)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
            .service(router::rkh::receive_pr)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
