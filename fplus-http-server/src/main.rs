use fplus_lib::core::allocator::update_installation_ids_logic;
mod middleware;
use middleware::verifier_auth::VerifierAuth;
pub(crate) mod router;

use actix_web::{
    middleware::{Compress, Logger},
    web, App, HttpServer,
};

use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time::sleep_until;

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

        let sleep_duration = (next - now)
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(1));
        let sleep_until_time = Instant::now() + sleep_duration;
        sleep_until(sleep_until_time.into()).await;

        let _ = task().await;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    if let Err(e) = fplus_database::setup().await {
        panic!("Failed to setup database connection: {}", e);
    }

    tokio::spawn(async {
        run_cron("0 0 0,4,8,12,16,20 * * * *", || {
            tokio::spawn(async {
                if let Err(e) = update_installation_ids_logic().await {
                    eprintln!("Error: {:?}", e);
                }
            })
        })
        .await;
    });

    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(Compress::default())
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
                    .service(router::application::additional_info_required)
                    .service(router::application::trigger_ssa)
                    .service(router::application::request_kyc)
                    .service(router::application::remove_pending_allocation)
                    .service(router::application::propose_storage_providers)
                    .service(router::application::approve_storage_providers)
                    .service(router::application::allocation_failed)
                    .service(router::application::decline)
                    .service(router::application::reopen_declined_application),
            )
            .service(router::application::merged)
            .service(router::application::active)
            .service(router::application::all_applications)
            .service(router::application::notify_refill)
            .service(router::application::closed_applications)
            .service(router::application::closed_allocator_applications)
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
            .service(router::application::trigger_ssa)
            .service(router::application::submit_kyc)
            .service(router::blockchain::address_allowance)
            .service(router::blockchain::verified_clients)
            .service(router::verifier::verifiers)
            .service(router::allocator::allocators)
            .service(router::allocator::allocator)
            .service(router::allocator::delete)
            .service(router::allocator::create_allocator_from_json)
            .service(router::allocator::update_allocator_force)
            .service(router::autoallocator::last_client_allocation)
            .service(router::autoallocator::trigger_autoallocation)
            .service(router::autoallocator::check_if_allowance_is_sufficient)
        // .service(router::allocator::get_installation_ids)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
