use actix_web::{
    error::{ErrorInternalServerError, ErrorUnauthorized},
    Error,
};
use fplus_lib::{config::get_env_var_or_default, external_services::github::github_async_new};

pub async fn check_if_pull_request_opened_by_bot(
    owner: &str,
    repo: &str,
    pr_number: &u64,
) -> Result<(), Error> {
    let gh_bot = get_env_var_or_default("BOT_USER");
    let gh = github_async_new(owner.to_string(), repo.to_string())
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to get GitHub client: {e}")))?;
    let pr = gh
        .get_pull_request_by_number(*pr_number)
        .await
        .map_err(|e| ErrorInternalServerError(format!("Failed to get pull request: {e}")))?;

    let login = pr
        .user
        .as_ref()
        .map(|user| user.login.as_str())
        .ok_or_else(|| {
            ErrorInternalServerError("User handle not found in pull request".to_string())
        })?;

    if login.is_empty() {
        return Err(ErrorInternalServerError("User handle is empty".to_string()));
    }
    if login != gh_bot {
        return Err(ErrorUnauthorized(format!("Unauthorized user: {login}")));
    }
    Ok(())
}
