use crate::get_database_connection;
use crate::models::applications::{
    ActiveModel, Column, Entity as Application, Model as ApplicationModel,
};
use chrono::{DateTime, Utc};
use sea_orm::prelude::Expr;
use sea_orm::{entity::*, query::*, DbBackend, DbErr};
use sha1::{Digest, Sha1};

/**
 * Get all applications from the database
 *
 * # Returns
 * @return Result<Vec<ApplicationModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_active_applications() -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;

    //Get all applications from the database.
    //Distinct on is not supported in sea_orm yet, so we have to use raw SQL
    let app_data = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
            SELECT DISTINCT ON (owner, repo, id) 
                a.id, 
                a.owner, 
                a.repo, 
                a.pr_number,
                a.issue_number,
                a.application, 
                a.updated_at, 
                a.sha,
                a.path,
                a.client_contract_address,
                a.issue_reporter_handle
            FROM 
                applications a 
            WHERE 
                (application::json->'Lifecycle'->>'Active')::boolean IS TRUE
            ORDER BY 
                a.owner, 
                a.repo, 
                a.id, 
                a.pr_number DESC
            "#,
        [],
    ))
    .all(&conn)
    .await?;

    let applications = app_data
        .into_iter()
        .map(|app| ApplicationModel {
            id: get_string_field(&app, "id").expect("ID must exist"),
            owner: get_string_field(&app, "owner").expect("Owner must exist"),
            repo: get_string_field(&app, "repo").expect("Repo must exist"),
            pr_number: get_i64_field(&app, "pr_number").expect("PR number must exist"),
            issue_number: get_i64_field(&app, "issue_number").expect("Issue number must exist"),
            application: get_string_field(&app, "application"),
            updated_at: parse_datetime_field(&app, "updated_at")
                .expect("Updated_at must be a valid datetime"),
            sha: get_string_field(&app, "sha"),
            path: get_string_field(&app, "path"),
            client_contract_address: get_string_field(&app, "client_contract_address"),
            issue_reporter_handle: get_string_field(&app, "issue_reporter_handle"),
        })
        .collect();

    Ok(applications)
}

fn get_string_field(json: &JsonValue, field: &str) -> Option<String> {
    json.get(field)?.as_str().map(|s| s.to_string())
}

/// Retrieves an i64 field from a JSON value, if it exists.
fn get_i64_field(json: &JsonValue, field: &str) -> Option<i64> {
    json.get(field)?.as_i64()
}

fn parse_datetime_field(json: &JsonValue, field: &str) -> Option<DateTime<Utc>> {
    json.get(field)?
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
}

/**
 * Get merged applications from the database
 *
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 *
 * # Returns
 * @return Result<Vec<ApplicationModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_merged_applications(
    owner: Option<String>,
    repo: Option<String>,
) -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let mut query = Application::find().filter(Column::PrNumber.eq(0));
    if let Some(owner) = owner.clone() {
        query = query.filter(Column::Owner.contains(owner));
    }
    if let Some(repo) = repo {
        if owner.is_none() {
            return Err(DbErr::Custom(
                "Owner is required to get merged applications".to_string(),
            ));
        }
        query = query.filter(Column::Repo.contains(repo));
    }
    query
        .order_by(Column::Owner, Order::Asc)
        .order_by(Column::Repo, Order::Asc)
        .all(&conn)
        .await
}

/**
 * Get active applications from the database
 *
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 *
 * # Returns
 * @return Result<Vec<ApplicationModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_applications_with_open_pull_request(
    owner: Option<String>,
    repo: Option<String>,
) -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let mut query = Application::find()
        .filter(Column::PrNumber.ne(0))
        .filter(Expr::cust(
            "(application::json->'Lifecycle'->>'Active')::boolean IS TRUE",
        ));

    if let Some(owner) = owner.clone() {
        query = query.filter(Column::Owner.contains(owner));
    }
    if let Some(repo) = repo {
        if owner.is_none() {
            return Err(DbErr::Custom(
                "Owner is required to get merged applications".to_string(),
            ));
        }
        query = query.filter(Column::Repo.contains(repo));
    }
    query
        .order_by(Column::Owner, Order::Asc)
        .order_by(Column::Repo, Order::Asc)
        .all(&conn)
        .await
}

/**
 * Get an application from the database with max pr_number for given id, owner and repo
 *
 * # Arguments
 * @param id: String - The ID of the application
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param pr_number: Option<u64> - Optional PR number to filter by
 *
 * # Returns
 * @return Result<ApplicationModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_application(
    id: String,
    owner: String,
    repo: String,
    pr_number: Option<u64>,
) -> Result<ApplicationModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let mut query = Application::find()
        .filter(Column::Id.eq(id))
        .filter(Column::Owner.contains(owner))
        .filter(Column::Repo.contains(repo));
    if let Some(number) = pr_number {
        query = query.filter(Column::PrNumber.eq(number as i64));
    }

    let application = query
        .order_by(Column::PrNumber, Order::Desc)
        .one(&conn)
        .await?
        .ok_or(DbErr::Custom("Application not found".to_string()))?;

    Ok(application)
}

/**
 * Get an application from the database with given pr_number
 *
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param pr_number: u64 - The PR number
 *
 * # Returns
 * @return Result<ApplicationModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_application_by_pr_number(
    owner: String,
    repo: String,
    pr_number: u64,
) -> Result<ApplicationModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let application = Application::find()
        .filter(Column::Owner.contains(owner))
        .filter(Column::Repo.contains(repo))
        .filter(Column::PrNumber.eq(pr_number as i64))
        .one(&conn)
        .await?
        .ok_or(DbErr::Custom("Application not found".to_string()))?;

    Ok(application)
}

/**
 * Get an application from the database with given issue_number
 *
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param issue_number: i64 - The issue number
 *
 * # Returns
 * @return Result<ApplicationModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_application_by_issue_number(
    owner: String,
    repo: String,
    issue_number: i64,
) -> Result<ApplicationModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;

    Application::find()
        .filter(Column::Owner.eq(owner))
        .filter(Column::Repo.eq(repo))
        .filter(Column::IssueNumber.eq(issue_number))
        .one(&conn)
        .await?
        .ok_or_else(|| DbErr::Custom("Application not found.".to_string()))
}

/**
 * Merge an application in the database
 *
 * # Arguments
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param pr_number: u64 - The PR number
 *
 * # Returns
 * @return Result<ApplicationModel, sea_orm::DbErr> - The result of the operation
 */
pub async fn merge_application_by_pr_number(
    owner: String,
    repo: String,
    pr_number: u64,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let pr_application =
        get_application_by_pr_number(owner.clone(), repo.clone(), pr_number).await?;

    let mut application_active_model: ActiveModel;
    if let Ok(application) = get_application(
        pr_application.id.clone(),
        owner.clone(),
        repo.clone(),
        Some(0),
    )
    .await
    {
        application_active_model = application.into_active_model();
        application_active_model.application = Set(pr_application.application.clone());
        application_active_model.sha = Set(pr_application.sha.clone());
        application_active_model.update(&conn).await?;
    } else {
        application_active_model = pr_application.clone().into_active_model();
        application_active_model.pr_number = Set(0);
        application_active_model.insert(&conn).await?;
    }
    pr_application.delete(&conn).await?;
    Ok(())
}

/**
 * Update an application in the database
 *
 * # Arguments
 * @param id: String - The ID of the application
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param pr_number: u64 - The PR number
 * @param app_file: String - The application file
 * @param path: Option<String> - The path of the application
 * @param sha: Option<String> - The SHA of the application
 *
 * # Returns
 * @return Result<ApplicationModel, sea_orm::DbErr> - The result of the operation
 */
#[allow(clippy::too_many_arguments)]
pub async fn update_application(
    id: String,
    owner: String,
    repo: String,
    pr_number: u64,
    app_file: String,
    path: Option<String>,
    sha: Option<String>,
    client_contract_address: Option<String>,
) -> Result<ApplicationModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;

    let existing_application =
        get_application(id.clone(), owner.clone(), repo.clone(), Some(pr_number)).await?;

    let mut active_application: ActiveModel = existing_application.into_active_model();
    active_application.application = Set(Some(app_file.clone()));
    let file_sha = sha.unwrap_or_else(|| {
        //Calculate SHA
        let mut hasher = Sha1::new();
        let application = format!("blob {}\x00{}", app_file.len(), app_file);
        hasher.update(application.as_bytes());
        format!("{:x}", hasher.finalize())
    });
    active_application.sha = Set(Some(file_sha));

    if let Some(path) = path {
        active_application.path = Set(Some(path));
    };

    if let Some(client_contract_address) = client_contract_address {
        active_application.client_contract_address = Set(Some(client_contract_address));
    } else {
        active_application.client_contract_address = Set(None);
    }

    let updated_application = active_application.update(&conn).await?;
    Ok(updated_application)
}

/**
 * Create an application in the database
 *
 * # Arguments
 * @param id: String - The ID of the application
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param pr_number: u64 - The PR number
 * @param app_file: String - The application file
 * @param sha: String - The SHA of the application
 * @param path: String - The path of the application
 *
 * # Returns
 * @return Result<ApplicationModel, sea_orm::DbErr> - The result of the operation
 */
#[allow(clippy::too_many_arguments)]
pub async fn create_application(
    id: String,
    owner: String,
    repo: String,
    pr_number: u64,
    issue_number: i64,
    app_file: String,
    path: String,
    issue_reporter_handle: Option<String>,
) -> Result<ApplicationModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    //Calculate SHA
    let mut hasher = Sha1::new();
    let application = format!("blob {}\x00{}", app_file.len(), app_file);
    hasher.update(application.as_bytes());
    let file_sha = format!("{:x}", hasher.finalize());

    let new_application = ActiveModel {
        id: Set(id),
        owner: Set(owner),
        repo: Set(repo),
        pr_number: Set(pr_number as i64),
        issue_number: Set(issue_number),
        application: Set(Some(app_file)),
        sha: Set(Some(file_sha)),
        path: Set(Some(path)),
        issue_reporter_handle: Set(issue_reporter_handle),
        ..Default::default()
    };

    let application = new_application.insert(&conn).await?;
    Ok(application)
}

/**
 * Delete an application from the database
 *
 * # Arguments
 * @param id: String - The ID of the application
 * @param owner: String - The owner of the repository
 * @param repo: String - The repository name
 * @param pr_number: u64 - The PR number
 *
 * # Returns
 * @return Result<(), sea_orm::DbErr> - The result of the operation
 */
pub async fn delete_application(
    id: String,
    owner: String,
    repo: String,
    pr_number: u64,
) -> Result<(), sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let application =
        get_application(id.clone(), owner.clone(), repo.clone(), Some(pr_number)).await?;
    application.delete(&conn).await?;
    Ok(())
}

pub async fn get_applications_by_client_id(
    id: &String,
) -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let result = Application::find()
        .filter(Column::Id.eq(id))
        .all(&conn)
        .await?;
    Ok(result)
}

pub async fn get_distinct_applications_by_clients_addresses(
    clients_addresses: Vec<String>,
) -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let result = Application::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT DISTINCT ON (id) * FROM applications 
         WHERE id = ANY($1)",
            [clients_addresses.into()],
        ))
        .all(&conn)
        .await?;

    Ok(result)
}

pub async fn get_closed_applications() -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let result = Application::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT * 
            FROM applications 
            WHERE (application::json->'Lifecycle'->>'Active')::boolean IS NOT TRUE",
            [],
        ))
        .all(&conn)
        .await?;
    Ok(result)
}

pub async fn get_allocator_closed_applications(
    owner: &str,
    repo: &str,
) -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let result = Application::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT DISTINCT ON (id) * 
            FROM applications 
            WHERE (application::json->'Lifecycle'->>'Active')::boolean IS NOT TRUE
            AND owner = $1
            AND repo = $2
            ORDER BY id, pr_number DESC",
            [owner.into(), repo.into()],
        ))
        .all(&conn)
        .await?;
    Ok(result)
}
