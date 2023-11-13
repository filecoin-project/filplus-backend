use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use env_logger;

pub(crate) mod router;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
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
            .service(router::application::validate_application_trigger)
            .service(router::application::validate_application_proposal)
            .service(router::application::validate_application_approval)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
