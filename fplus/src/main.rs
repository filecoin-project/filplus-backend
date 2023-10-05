use std::sync::Mutex;

use actix_web::web;
use clap::{arg, Command};

async fn validate_rkh_holder(github_handle: String, app_id: String) {
    let db_connection: web::Data<Mutex<mongodb::Client>> =
        web::Data::new(Mutex::new(database::core::setup::setup().await.unwrap()));
    let rkh_holders = database::core::collections::rkh::find(db_connection)
        .await
        .unwrap();
    let rkh_holders: Option<database::core::collections::rkh::RootKeyHolder> = rkh_holders
        .into_iter()
        .find(|rkh| &rkh.github_handle == &github_handle);
    if rkh_holders.is_none() {
        println!(
            "No Root Key Holder found with github handle {}",
            github_handle
        );
    } else {
        println!(
            "Validated Root Key Holder {} for application {}",
            github_handle, app_id
        );
    }
}

fn cli() -> Command {
    Command::new("filplus")
        .about("Fil+ CLI - Management tool for fil+ applications")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("validate-trigger")
                .about("Validates triggering an application")
                .arg(arg!(<APP_ID> "Application ID"))
                .arg(arg!(<RKH_GITHUB_HANDLE> "Github handle of Root Key Holder"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("validate-proposal")
                .about("Validates proposing an application")
                .arg(arg!(<APP_ID> "Application ID"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("validate-approval")
                .about("Validates approving an application")
                .arg(arg!(<APP_ID> "Application ID"))
                .arg_required_else_help(true),
        )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    Ok(match matches.subcommand() {
        Some(("validate-trigger", sub_matches)) => {
            let app_id = sub_matches.get_one::<String>("APP_ID").expect("required");
            let rkh_gh_handle = sub_matches
                .get_one::<String>("RKH_GITHUB_HANDLE")
                .expect("required");
            validate_rkh_holder(rkh_gh_handle.to_string(), app_id.to_string()).await;
        }
        Some(("validate-proposal", sub_matches)) => {
            let app_id = sub_matches.get_one::<String>("APP_ID").expect("required");
            println!("Validated proposal {}", app_id);
        }
        Some(("validate-approval", sub_matches)) => {
            let app_id = sub_matches.get_one::<String>("APP_ID").expect("required");
            println!("Validated approval {}", app_id);
        }
        _ => {
            println!("No subcommand was used");
        }
    })
}
