use clap::{arg, Command};
use validators::{validate_proposal, validate_trigger};

use crate::validators::validate_approval;

pub mod validators;

fn cli() -> Command {
    Command::new("filplus")
        .about("Fil+ CLI - Management tool for fil+ applications")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("validate-trigger")
                .about("Validates triggering an application")
                .arg(arg!(<PR_NUMBER> "Pull Request Number"))
                .arg(arg!(<VERIFIER_GITHUB_HANDLE> "Github handle of Verifier"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("validate-proposal")
                .about("Validates proposing an application")
                .arg(arg!(<PR_NUMBER> "Pull Request Number"))
                .arg(arg!(<VERIFIER_GITHUB_HANDLE> "Github handle of Verifier"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("validate-approval")
                .about("Validates approving an application")
                .arg(arg!(<PR_NUMBER> "Pull Request Number"))
                .arg_required_else_help(true),
        )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    Ok(match matches.subcommand() {
        Some(("validate-trigger", sub_matches)) => {
            let pull_request_number = sub_matches
                .get_one::<String>("PR_NUMBER")
                .expect("required");
            let verifier_gh_handle = sub_matches
                .get_one::<String>("VERIFIER_GITHUB_HANDLE")
                .expect("required");
            validate_trigger(verifier_gh_handle.to_string(), pull_request_number.to_string()).await;
        }
        Some(("validate-proposal", sub_matches)) => {
            let pull_request_number = sub_matches
                .get_one::<String>("PR_NUMBER")
                .expect("required");
            let verifier_gh_handle = sub_matches
                .get_one::<String>("VERIFIER_GITHUB_HANDLE")
                .expect("required");
            validate_proposal(
                verifier_gh_handle.to_string(),
                pull_request_number.to_string(),
            )
            .await;
        }
        Some(("validate-approval", sub_matches)) => {
            let pull_request_number = sub_matches
                .get_one::<String>("PR_NUMBER")
                .expect("required");
            let verifier_gh_handle = sub_matches
                .get_one::<String>("VERIFIER_GITHUB_HANDLE")
                .expect("required");
            validate_approval(
                verifier_gh_handle.to_string(),
                pull_request_number.to_string(),
            )
            .await;
            println!("Validated approval {}", pull_request_number);
        }
        _ => {
            println!("No subcommand was used");
        }
    })
}
