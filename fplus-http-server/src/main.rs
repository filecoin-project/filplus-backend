use env_logger;
use fplus_lib::core::allocator::update_installation_ids_logic;
use log::info;
mod middleware;
use middleware::verifier_auth::VerifierAuth;
pub(crate) mod router;
use std::env;

use actix_web::{
    App,
    HttpServer,
    web,
    middleware::Logger,
};

use cron::Schedule;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time::sleep_until;
use chrono::Utc;

pub async fn run_cron<F>(expression: &str, mut task: F)
where
    F: FnMut() -> tokio::task::JoinHandle<()> + Send + 'static,
{
    let schedule = match Schedule::from_str(expression) {
        Ok(schedule) => schedule,
        Err(e) => {
            log::error!("Failed to parse CRON expression: {}", e);
            return; // Exit the function or handle the error without panicking
        }
    };
    loop {
        let now = Utc::now();
        let next = match schedule.upcoming(Utc).next() {
            Some(next_time) => next_time,
            None => continue,
        };

        let sleep_duration = (next - now).to_std().unwrap_or_else(|_| Duration::from_secs(1));
        let sleep_until_time = Instant::now() + sleep_duration;
        sleep_until(sleep_until_time.into()).await;

        let _ = task().await;
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

    tokio::spawn(async {
        run_cron("0 0 0,4,8,12,16,20 * * * *", || tokio::spawn(update_installation_ids_logic())).await;
    });

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
                    .wrap(VerifierAuth)
                    .service(router::application::trigger)
                    .service(router::application::approve_changes)
                    .service(router::application::propose)
                    .service(router::application::approve)
                    .service(router::application::decline)
                    .service(router::application::additional_info_required)
                    .service(router::application::trigger_ssa)
            )
            .service(router::application::merged)
            .service(router::application::active)
            .service(router::application::all_applications)
            .service(router::application::refill)
            .service(router::application::total_dc_reached)
            .service(router::application::single)
            .service(router::application::application_with_allocation_amount_handler)
            .service(router::application::validate_application_flow)
            .service(router::application::check_for_changes)
            .service(router::application::validate_application_trigger)
            .service(router::application::validate_application_proposal)
            .service(router::application::validate_application_approval)
            .service(router::application::validate_application_merge)
            .service(router::application::delete_branch)
            .service(router::application::cache_renewal)
            .service(router::application::update_from_issue)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
            .service(router::verifier::verifiers)
            .service(router::allocator::allocators)
            .service(router::allocator::allocator)
            .service(router::allocator::delete)
            .service(router::allocator::create_from_json)
            .service(router::allocator::update_single_installation_id)
            .service(router::allocator::update_allocator_force)
            // .service(router::allocator::get_installation_ids)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
