pub fn init() {
    dotenv::dotenv().ok();
}

pub fn get_applications_folder() -> String {
    std::env::var("APPLICATIONS_FOLDER").unwrap_or_else(|_| {
        println!("APPLICATIONS_FOLDER not found in environment, using default 'applications'");
        "applications".to_string()
    })
}

pub fn get_github_private_key() -> String {
    std::env::var("GH_PRIVATE_KEY").unwrap_or_else(|_| {
        eprintln!("GH_PRIVATE_KEY not found in environment or .env file.");
        std::process::exit(1);
    })
}