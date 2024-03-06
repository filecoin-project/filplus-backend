use octocrab::models::repos::ContentItems;

use crate::config::get_env_var_or_default;
use crate::{base64::decode_allocator_model, error::LDNError, external_services::github::GithubWrapper};

use self::file::AllocatorModel;

pub mod file;

