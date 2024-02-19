pub async fn validate_trigger(github_handle: String, pull_request_number: String) -> bool {
    true
    // if validate_verifier(&github_handle).await {
    //     println!(
    //         "Validated Root Key Holder {} for application {}",
    //         &github_handle, pull_request_number
    //     );
    // } else {
    //     println!(
    //         "No Root Key Holder found with github handle {}",
    //         github_handle
    //     );
    // }
}

pub async fn validate_proposal(github_handle: String, pull_request_number: String) -> bool {
    true
    // if validate_notary(&github_handle).await {
    //     println!(
    //         "Validated Notary {} Proposal for application {}",
    //         &github_handle, pull_request_number
    //     );
    // } else {
    //     println!("No Notary found with github handle {}", github_handle);
    // }
}

pub async fn validate_approval(github_handle: String, pull_request_number: String) -> bool {
    true
    // if validate_notary(&github_handle).await {
    //     println!(
    //         "Validated Notary {} Approval for application {}",
    //         &github_handle, pull_request_number
    //     );
    // } else {
    //     println!("No Notary found with github handle {}", github_handle);
    // }
}

async fn validate_verifier(github_handle: &str) -> bool {
    true
    // let db_connection: web::Data<Mutex<mongodb::Client>> = web::Data::new(Mutex::new(
    //     fplus_database::core::setup::setup().await.unwrap(),
    // ));
    // let verifiers_list = fplus_database::core::collections::verifier::find(db_connection)
    //     .await
    //     .unwrap();
    // let verifiers_list: Option<fplus_database::core::collections::verifier::Verifier> = verifiers
    //     .into_iter()
    //     .find(|gov| &gov.github_handle == github_handle);
    // if verifiers_list.is_none() {
    //     false
    // } else {
    //     true
    // }
}