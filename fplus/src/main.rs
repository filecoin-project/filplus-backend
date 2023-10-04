use clap::{arg, Command};

fn cli() -> Command {
    Command::new("filplus")
        .about("Fil+ CLI - Management tool for fil+ applications")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("validate-trigger")
                .about("Validates triggering an application")
                .arg(arg!(<APP_ID> "Application ID"))
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

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("validate-trigger", sub_matches)) => {
            let app_id = sub_matches.get_one::<String>("APP_ID").expect("required");
            println!("Validated Trigger {}", app_id);
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
    }
}
