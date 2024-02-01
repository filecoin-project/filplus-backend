pub async fn validate_trigger(github_handle: String, pull_request_number: String) -> bool {
    true
    // if validate_gov_team_member(&github_handle).await {
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

async fn validate_gov_team_member(github_handle: &str) -> bool {
    true
    // let db_connection: web::Data<Mutex<mongodb::Client>> = web::Data::new(Mutex::new(
    //     fplus_database::core::setup::setup().await.unwrap(),
    // ));
    // let gov_team_members = fplus_database::core::collections::govteam::find(db_connection)
    //     .await
    //     .unwrap();
    // let gov_team_members: Option<fplus_database::core::collections::govteam::GovTeamMember> = gov_team_members
    //     .into_iter()
    //     .find(|gov| &gov.github_handle == github_handle);
    // if gov_team_members.is_none() {
    //     false
    // } else {
    //     true
    // }
}

async fn validate_notary(github_handle: &str) -> bool {
    true
    // let db_connection: web::Data<Mutex<mongodb::Client>> = web::Data::new(Mutex::new(
    //     fplus_database::core::setup::setup().await.unwrap(),
    // ));
    // let notary = fplus_database::core::collections::notary::find(db_connection)
    //     .await
    //     .unwrap();
    // let notary: Option<fplus_database::core::collections::notary::Notary> = notary
    //     .into_iter()
    //     .find(|n| &n.github_handle == github_handle);
    // if notary.is_none() {
    //     false
    // } else {
    //     true
    // }
}
