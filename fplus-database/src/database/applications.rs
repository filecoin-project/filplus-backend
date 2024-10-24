use crate::get_database_connection;
use crate::models::applications::{
    ActiveModel, Column, Entity as Application, Model as ApplicationModel,
};
use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{entity::*, query::*, DbBackend, DbErr};
use sha1::{Digest, Sha1};

/**
 * Get all applications from the database
 *
 * # Returns
 * @return Result<Vec<ApplicationModel>, sea_orm::DbErr> - The result of the operation
 */
pub async fn get_applications() -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
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
                a.client_contract_address
            FROM 
                applications a 
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

    //Iterate over the results and convert them to ApplicationModel
    let mut applications: Vec<ApplicationModel> = Vec::new();
    for app in app_data {
        applications.push(ApplicationModel {
            id: app.get("id").unwrap().as_str().unwrap().to_string(),
            owner: app.get("owner").unwrap().as_str().unwrap().to_string(),
            repo: app.get("repo").unwrap().as_str().unwrap().to_string(),
            pr_number: app.get("pr_number").unwrap().as_i64().unwrap(),
            issue_number: app
                .get("issue_number")
                .expect("Issue number should be in the application data")
                .as_i64()
                .expect("Issue number must exist"),
            application: Some(
                app.get("application")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            updated_at: Utc.from_utc_datetime(
                &app.get("updated_at")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .naive_utc(),
            ),
            sha: Some(app.get("sha").unwrap().as_str().unwrap().to_string()),
            path: Some(app.get("path").unwrap().as_str().unwrap().to_string()),
            client_contract_address: app
                .get("client_contract_address")
                .map(|client_contract_address| client_contract_address.to_string()),
        });
    }
    Ok(applications)
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
pub async fn get_active_applications(
    owner: Option<String>,
    repo: Option<String>,
) -> Result<Vec<ApplicationModel>, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let mut query = Application::find().filter(Column::PrNumber.ne(0));
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

    let result = query
        .order_by(Column::PrNumber, Order::Desc)
        .one(&conn)
        .await?;

    match result {
        Some(application) => Ok(application),
        None => Err(DbErr::Custom("Application not found".to_string())),
    }
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
    let result = Application::find()
        .filter(Column::Owner.contains(owner))
        .filter(Column::Repo.contains(repo))
        .filter(Column::PrNumber.eq(pr_number as i64))
        .one(&conn)
        .await?;

    match result {
        Some(application) => Ok(application),
        None => Err(DbErr::Custom("Application not found".to_string())),
    }
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
) -> Result<ApplicationModel, sea_orm::DbErr> {
    let conn = get_database_connection().await?;
    let pr_application =
        get_application_by_pr_number(owner.clone(), repo.clone(), pr_number).await?;
    let mut exists_merged = true;

    let mut merged_application = match get_application(
        pr_application.id.clone(),
        owner.clone(),
        repo.clone(),
        Some(0),
    )
    .await
    {
        Ok(application) => application.into_active_model(),
        Err(_) => {
            exists_merged = false;
            ActiveModel {
                id: Set(pr_application.id.clone()),
                owner: Set(owner),
                repo: Set(repo),
                pr_number: Set(0),
                application: Set(pr_application.application.clone()),
                path: Set(pr_application.path.clone()),
                ..Default::default()
            }
        }
    };

    let mut hasher = Sha1::new();
    let application = match pr_application.application.clone() {
        Some(app) => format!("blob {}\x00{}", app.len(), app),
        None => "".to_string(),
    };
    hasher.update(application.as_bytes());
    let file_sha = format!("{:x}", hasher.finalize());
    merged_application.sha = Set(Some(file_sha));
    merged_application.application = Set(pr_application.application.clone());

    pr_application.delete(&conn).await?;

    if exists_merged {
        let result = merged_application.update(&conn).await;
        match result {
            Ok(application) => Ok(application),
            Err(e) => Err(sea_orm::DbErr::Custom(format!(
                "Failed to merge application: {}",
                e
            ))),
        }
    } else {
        let result = merged_application.insert(&conn).await;
        match result {
            Ok(application) => Ok(application),
            Err(e) => Err(sea_orm::DbErr::Custom(format!(
                "Failed to merge application: {}",
                e
            ))),
        }
    }
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

    match get_application(id.clone(), owner.clone(), repo.clone(), Some(pr_number)).await {
        Ok(existing_application) => {
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
        Err(_) => Err(sea_orm::DbErr::Custom(
            "Failed to find the application to update.".into(),
        )),
    }
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
pub async fn create_application(
    id: String,
    owner: String,
    repo: String,
    pr_number: u64,
    issue_number: i64,
    app_file: String,
    path: String,
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
        ..Default::default()
    };

    match new_application.insert(&conn).await {
        Ok(application) => Ok(application),
        Err(e) => Err(sea_orm::DbErr::Custom(format!(
            "Failed to insert new application: {}",
            e
        ))),
    }
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
