//! Fax router — mirrors `app/fax/router.py` (accounts/files/logs/users/send)
//! + `app/fax_queue` (status/retry).
//!
//! Order: models → schemas → handlers → xml (no FS XML for fax; S3/file stub note).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{fax_files, fax_logs, fax_queue, fax_users, faxes};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct FaxQueueSubmit {
    fax_extension: String,
    destination: String,
}

#[derive(Debug, Serialize)]
struct FaxQueueOut {
    fax_queue_uuid: String,
    fax_extension: String,
    destination: String,
    status: String,
}

// ---- handlers ----
fn valid_ext(s: &str) -> bool {
    let t = s.trim();
    (2..=7).contains(&t.len()) && t.chars().all(|c| c.is_ascii_digit())
}

fn ser_fax(r: &faxes::Model) -> serde_json::Value {
    serde_json::json!({
        "fax_uuid": r.fax_uuid,
        "fax_extension": r.fax_extension,
        "fax_name": r.fax_name,
        "description": r.description,
        "enabled": r.enabled,
    })
}

fn ser_queue(r: &fax_queue::Model) -> serde_json::Value {
    serde_json::json!({
        "fax_queue_uuid": r.fax_queue_uuid,
        "fax_extension": r.fax_extension,
        "destination": r.destination,
        "status": r.status,
    })
}

async fn fax_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<faxes::Model> {
    faxes::Entity::find_by_id(id.to_owned())
        .filter(faxes::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn health_fax() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "fax"}))
}

async fn create_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ext = payload
        .get("fax_extension")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    let name = payload
        .get("fax_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if !valid_ext(ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_extension must be 2-7 digits"})),
        )
            .into_response();
    }
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = faxes::ActiveModel {
        fax_uuid: Set(id.clone()),
        domain_id: Set(domain),
        fax_extension: Set(ext.to_owned()),
        fax_name: Set(name.to_owned()),
        description: Set(payload
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "fax_uuid": id,
                "fax_extension": ext,
                "fax_name": name,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = faxes::Entity::find()
        .filter(faxes::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_fax).collect();
    (
        crate::paged_headers("/api/v1/fax", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn get_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match fax_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_fax(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response(),
    }
}

async fn patch_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = fax_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    };
    let mut am: faxes::ActiveModel = r.into();
    if let Some(v) = payload.get("fax_name").and_then(|v| v.as_str()) {
        am.fax_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_fax(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // Set deletes: 3 statements, no per-row fetch-then-delete round-trips.
    let _ = fax_files::Entity::delete_many()
        .filter(fax_files::Column::FaxUuid.eq(id.clone()))
        .filter(fax_files::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let _ = fax_logs::Entity::delete_many()
        .filter(fax_logs::Column::FaxUuid.eq(id.clone()))
        .filter(fax_logs::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let _ = fax_users::Entity::delete_many()
        .filter(fax_users::Column::FaxUuid.eq(id.clone()))
        .filter(fax_users::Column::DomainId.eq(domain))
        .exec(db)
        .await;
    let Some(r) = fax_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    };
    let am: faxes::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_files(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = fax_files::Entity::find()
        .filter(fax_files::Column::FaxUuid.eq(id.clone()))
        .filter(fax_files::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "fax_file_uuid": r.fax_file_uuid,
                "fax_uuid": r.fax_uuid,
                "file_name": r.file_name,
            })
        })
        .collect();
    (
        crate::paged_headers(&format!("/api/v1/fax/{id}/files"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn upload_file(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let fid = crate::routing_db::new_uuid();
    let am = fax_files::ActiveModel {
        fax_file_uuid: Set(fid.clone()),
        domain_id: Set(domain),
        fax_uuid: Set(id.clone()),
        file_name: Set(payload
            .get("file_name")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"fax_file_uuid": fid, "fax_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_logs(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = fax_logs::Entity::find()
        .filter(fax_logs::Column::FaxUuid.eq(id.clone()))
        .filter(fax_logs::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "fax_log_uuid": r.fax_log_uuid,
                "fax_uuid": r.fax_uuid,
                "fax_result_text": r.fax_result_text,
            })
        })
        .collect();
    (
        crate::paged_headers(&format!("/api/v1/fax/{id}/logs"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn list_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if fax_or_404(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let paginator = fax_users::Entity::find()
        .filter(fax_users::Column::FaxUuid.eq(id.clone()))
        .filter(fax_users::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "fax_user_uuid": r.fax_user_uuid,
                "fax_uuid": r.fax_uuid,
                "user_uuid": r.user_uuid,
            })
        })
        .collect();
    (
        crate::paged_headers(&format!("/api/v1/fax/{id}/users"), page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn send_fax(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(f) = fax_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax not found"})),
        )
            .into_response();
    };
    let dest = payload
        .get("destination")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("");
    if dest.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "destination required"})),
        )
            .into_response();
    }
    // Enqueue a fax_queue job (mirrors Python send → queue insert).
    let qid = crate::routing_db::new_uuid();
    let am = fax_queue::ActiveModel {
        fax_queue_uuid: Set(qid.clone()),
        domain_id: Set(domain),
        fax_extension: Set(f.fax_extension),
        destination: Set(dest.to_owned()),
        status: Set("waiting".to_owned()),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "fax_queue_uuid": qid,
            "fax_uuid": id,
            "destination": dest,
            "status": "waiting",
        })),
    )
        .into_response()
}

// ---- fax_queue (legacy submit path + full CRUD) ----
async fn submit_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<FaxQueueSubmit>,
) -> impl IntoResponse {
    if payload.fax_extension.trim().is_empty() || payload.destination.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "fax_extension/destination required"})),
        )
            .into_response();
    }
    insert_queue_job(&claims, &payload.fax_extension, &payload.destination).await
}

async fn insert_queue_job(
    claims: &pbx_core::auth::Claims,
    ext: &str,
    dest: &str,
) -> axum::response::Response {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(claims);
    let id = crate::routing_db::new_uuid();
    let am = fax_queue::ActiveModel {
        fax_queue_uuid: Set(id.clone()),
        domain_id: Set(domain),
        fax_extension: Set(ext.trim().to_owned()),
        destination: Set(dest.trim().to_owned()),
        status: Set("waiting".to_owned()),
    };
    if am.insert(db).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(FaxQueueOut {
            fax_queue_uuid: id,
            fax_extension: ext.to_owned(),
            destination: dest.to_owned(),
            status: "waiting".to_owned(),
        })),
    )
        .into_response()
}

async fn list_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let paginator = fax_queue::Entity::find()
        .filter(fax_queue::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_queue).collect();
    (
        crate::paged_headers("/api/v1/fax_queue", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
}

async fn queue_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<fax_queue::Model> {
    fax_queue::Entity::find_by_id(id.to_owned())
        .filter(fax_queue::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn get_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match queue_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_queue(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response(),
    }
}

async fn patch_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response();
    };
    let mut am: fax_queue::ActiveModel = r.into();
    if let Some(v) = payload.get("status").and_then(|v| v.as_str()) {
        am.status = Set(v.to_owned());
    }
    if let Some(v) = payload.get("destination").and_then(|v| v.as_str()) {
        am.destination = Set(v.to_owned());
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_queue(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response();
    };
    let am: fax_queue::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

async fn status_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match queue_or_404(db, domain, &id).await {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "fax_queue_uuid": r.fax_queue_uuid,
                "status": r.status,
                "fax_extension": r.fax_extension,
                "destination": r.destination,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response(),
    }
}

async fn retry_queue(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = queue_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Fax queue not found"})),
        )
            .into_response();
    };
    let mut am: fax_queue::ActiveModel = r.into();
    am.status = Set("waiting".to_owned());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "fax_queue_uuid": r.fax_queue_uuid,
                "status": r.status,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn fax_router() -> Router {
    Router::new()
        .route("/api/v1/fax/health", axum::routing::get(health_fax))
        .route("/api/v1/fax", post(create_fax).get(list_fax))
        .route("/api/v1/fax/", post(create_fax).get(list_fax))
        .route(
            "/api/v1/fax/{id}",
            axum::routing::get(get_fax)
                .patch(patch_fax)
                .delete(delete_fax),
        )
        .route(
            "/api/v1/fax/{id}/files",
            axum::routing::get(list_files).post(upload_file),
        )
        .route("/api/v1/fax/{id}/logs", axum::routing::get(list_logs))
        .route("/api/v1/fax/{id}/users", axum::routing::get(list_users))
        .route("/api/v1/fax/{id}/send", post(send_fax))
        .route("/api/v1/fax/queue", post(submit_queue).get(list_queue))
        .route("/api/v1/fax/queue/", post(submit_queue).get(list_queue))
        .route("/api/v1/fax_queue", post(submit_queue).get(list_queue))
        .route("/api/v1/fax_queue/", post(submit_queue).get(list_queue))
        .route(
            "/api/v1/fax_queue/{id}",
            axum::routing::get(get_queue)
                .patch(patch_queue)
                .delete(delete_queue),
        )
        .route(
            "/api/v1/fax_queue/{id}/status",
            axum::routing::get(status_queue),
        )
        .route("/api/v1/fax_queue/{id}/retry", post(retry_queue))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
