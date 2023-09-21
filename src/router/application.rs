use crate::core::{
    CompleteGovernanceReviewInfo, CompleteNewApplicationProposalInfo, CreateApplicationInfo,
    LDNApplication,
};
use actix_web::{get, post, web, HttpResponse, Responder};

/// Create a new application.
///
/// # Returns
/// Returns the application id.
///
/// # Example
/// ```plaintext
/// curl --header "Content-Type: application/json" 
///      --request POST 
///      --data '{"application_id": "0x1234"}' 
///      http://localhost:8080/application
/// ```
///
/// # Response
/// Created new application for issue: 0x1234
#[post("/application")]
pub async fn create_application(info: web::Json<CreateApplicationInfo>) -> impl Responder {
    match LDNApplication::new(info.into_inner()).await {
        Ok(app) => HttpResponse::Ok().body(format!(
            "Created new application for issue: {}",
            app.application_id.clone()
        )),
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    }
}

/// Trigger an application.
///
/// # Returns
/// Returns the ApplicationFile.
///
/// # Example
/// ```plaintext
/// curl --header "Content-Type: application/json" 
///      --request POST 
///      --data '{"actor": "JohnDoe"}' 
///      http://localhost:8080/application/0x1234/trigger
/// ```
///
/// # Response
/// ```json
/// {
///  "id": "0x1234",
///  "_type": "ldn-v3",
///  ..
/// }
/// ```
#[post("/application/{id}/trigger")]
pub async fn trigger_application(
    id: web::Path<String>,
    info: web::Json<CompleteGovernanceReviewInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_governance_review(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => {
            return HttpResponse::BadRequest().body("Application is not in the correct state");
        }
    }
}

/// Propose an application.
///
/// # Returns
/// Returns the ApplicationFile.
///
/// # Example
/// ```plaintext
/// curl --header "Content-Type: application/json" 
///      --request POST 
///      --data '{
///         "signer": {
///           "signing_address": "0x1234567890abcdef1234567890abcdef12345678",
///           "time_of_signature": "2023-08-07T14:30:00Z",
///           "message_cid": "QmXYZ1234567890abcdef1234567890abcdef12345678"
///         },
///         "request_id": "exampleRequestId123"
///      }' 
///      http://localhost:8080/application/0x1234/propose
/// ```
///
/// # Response
/// ```json
/// {
///  "id": "0x1234",
///  "_type": "ldn-v3",
///  ..
/// }
/// ```
#[post("/application/{id}/propose")]
pub async fn propose_application(
    id: web::Path<String>,
    info: web::Json<CompleteNewApplicationProposalInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_new_application_proposal(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => {
            return HttpResponse::BadRequest().body("Application is not in the correct state");
        }
    }
}
/// Approve an application.
///
/// # Returns
/// Returns the ApplicationFile.
///
/// # Example
/// ```plaintext
/// curl --header "Content-Type: application/json" 
///      --request POST 
///      --data '{
///         "signer": {
///           "signing_address": "0x1234567890abcdef1234567890abcdef12345678",
///           "time_of_signature": "2023-08-07T14:30:00Z",
///           "message_cid": "QmXYZ1234567890abcdef1234567890abcdef12345678"
///         },
///         "request_id": "exampleRequestId123"
///      }' 
///      http://localhost:8080/application/0x1234/approve
/// ```
///
/// # Response
/// ```json
/// {
///  "id": "0x1234",
///  "_type": "ldn-v3",
///  ..
/// }
/// ```
#[post("/application/{id}/approve")]
pub async fn approve_application(
    id: web::Path<String>,
    info: web::Json<CompleteNewApplicationProposalInfo>,
) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application
        .complete_new_application_approval(info.into_inner())
        .await
    {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

/// Merge a previously proposed application.
///
/// # Returns
/// Returns the ApplicationFile.
///
/// # Example
/// ```plaintext
/// curl --header "Content-Type: application/json" 
///      --request POST 
///      http://localhost:8080/application/0x1234/merge
/// ```
///
/// # Response
/// ```json
/// {
///  "id": "0x1234",
///  "_type": "ldn-v3",
///  ..
/// }
/// ```
#[post("/application/{id}/merge")]
pub async fn merge_application(id: web::Path<String>) -> impl Responder {
    let ldn_application = match LDNApplication::load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return HttpResponse::BadRequest().body(e.to_string());
        }
    };
    match ldn_application.merge_new_application_pr().await {
        Ok(app) => HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()),
        Err(_) => HttpResponse::BadRequest().body("Application is not in the correct state"),
    }
}

/// Retrieve an application based on its ID.
///
/// # Example
/// ```plaintext
/// curl -X GET http://localhost:8080/application/0x1234
/// ```
///
/// # Response
/// ```json
/// {
///  "id": "0x1234",
///  "_type": "ldn-v3",
///  ..
/// }
/// ```
#[get("/application/{id}")]
pub async fn get_application(id: web::Path<String>) -> actix_web::Result<impl Responder> {
    let app = match LDNApplication::app_file_without_load(id.into_inner()).await {
        Ok(app) => app,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().body(e.to_string()));
        }
    };
    Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&app).unwrap()))
}

/// Retrieve all active applications.
///
/// # Example
/// ```plaintext
/// curl -X GET http://localhost:8080/application
/// ```
///
/// # Response
/// ```json
/// [
///   {
///     "id": "0x1234",
///     "_type": "ldn-v3",
///     ..
///   },
///   ...
/// ]
/// ```
#[get("/application")]
pub async fn get_all_applications() -> actix_web::Result<impl Responder> {
    let apps = match LDNApplication::get_all_active_applications().await {
        Ok(app) => app,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().body(e.to_string()));
        }
    };
    Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&apps).unwrap()))
}

/// Check the health status.
///
/// # Example
/// ```plaintext
/// curl -X GET http://localhost:8080/health
/// ```
///
/// # Response
/// `OK`
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
