use actix_web::web;
use std::sync::Mutex;

pub async fn validate_trigger(github_handle: String, pull_request_number: String) {
    if validate_rkh_holder(&github_handle).await {
        println!(
            "Validated Root Key Holder {} for application {}",
            &github_handle, pull_request_number
        );
    } else {
        println!(
            "No Root Key Holder found with github handle {}",
            github_handle
        );
    }
}

pub async fn validate_proposal(github_handle: String, pull_request_number: String) {
    if validate_notary(&github_handle).await {
        println!(
            "Validated Notary {} Proposal for application {}",
            &github_handle, pull_request_number
        );
    } else {
        println!("No Notary found with github handle {}", github_handle);
    }
}

pub async fn validate_approval(github_handle: String, pull_request_number: String) {
    if validate_notary(&github_handle).await {
        println!(
            "Validated Notary {} Approval for application {}",
            &github_handle, pull_request_number
        );
    } else {
        println!("No Notary found with github handle {}", github_handle);
    }
    // let application_file: ApplicationFile =
    //     fplus_lib::core::LDNApplication::get_by_pr_number(pull_request_number.parse().unwrap()).await.unwrap();
}

async fn validate_rkh_holder(github_handle: &str) -> bool {
    let db_connection: web::Data<Mutex<mongodb::Client>> =
        web::Data::new(Mutex::new(fplus_database::core::setup::setup().await.unwrap()));
    let rkh_holders = fplus_database::core::collections::rkh::find(db_connection)
        .await
        .unwrap();
    let rkh_holders: Option<fplus_database::core::collections::rkh::RootKeyHolder> = rkh_holders
        .into_iter()
        .find(|rkh| &rkh.github_handle == github_handle);
    if rkh_holders.is_none() {
        false
    } else {
        true
    }
}

async fn validate_notary(github_handle: &str) -> bool {
    let db_connection: web::Data<Mutex<mongodb::Client>> =
        web::Data::new(Mutex::new(fplus_database::core::setup::setup().await.unwrap()));
    let notary = fplus_database::core::collections::notary::find(db_connection)
        .await
        .unwrap();
    let notary: Option<fplus_database::core::collections::notary::Notary> = notary
        .into_iter()
        .find(|n| &n.github_handle == github_handle);
    if notary.is_none() {
        false
    } else {
        true
    }
}
