//! Recordings router — mirrors `app/recordings` + `app/call_recordings`.
//!
//! Order: models → schemas → handlers → xml (S3 stub note).
//!
//! Storage backend is S3 (`recordings/{domain_id}/...` per models.py).
//! `GET /presign` returns a documented stub URL (no S3 calls in tests).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::recordings;

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct RecordingCreate {
    name: String,
    #[serde(default)]
    file_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct RecordingOut {
    recording_uuid: String,
    name: String,
    file_path: String,
}

// ---- handlers ----
async fn create_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<RecordingCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_add") {
        return (s, j).into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let path = payload
        .file_path
        .unwrap_or_else(|| format!("recordings/{domain}/{id}.wav"));
    let am = recordings::ActiveModel {
        recording_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        file_path: Set(path.clone()),
        storage_backend: Set("s3".to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
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
        Json(serde_json::json!(RecordingOut {
            recording_uuid: id,
            name: payload.name,
            file_path: path,
        })),
    )
        .into_response()
}

async fn list_rec(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = recordings::Entity::find()
        .filter(recordings::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "recording_uuid": r.recording_uuid,
                "name": r.name,
                "file_path": r.file_path,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn presign_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = recordings::Entity::find_by_id(id.clone())
        .filter(recordings::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "recording_uuid": r.recording_uuid,
                "url": format!("s3://stub-bucket/{}", r.file_path),
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn rec_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<recordings::Model> {
    recordings::Entity::find_by_id(id.to_owned())
        .filter(recordings::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

fn ser_rec(r: &recordings::Model) -> serde_json::Value {
    serde_json::json!({
        "recording_uuid": r.recording_uuid,
        "name": r.name,
        "file_path": r.file_path,
        "storage_backend": r.storage_backend,
    })
}

async fn get_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match rec_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_rec(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn playback_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match rec_or_404(db, domain, &id).await {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "recording_uuid": r.recording_uuid,
                "file_path": r.file_path,
                "playback_url": format!("/api/v1/recordings/{}/playback", r.recording_uuid),
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = rec_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: recordings::ActiveModel = r.into();
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_rec(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "recording_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = rec_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: recordings::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn recordings_router() -> Router {
    Router::new()
        .route("/api/v1/recordings", post(create_rec).get(list_rec))
        .route("/api/v1/recordings/", post(create_rec).get(list_rec))
        .route(
            "/api/v1/recordings/{id}",
            axum::routing::get(get_rec)
                .patch(patch_rec)
                .delete(delete_rec),
        )
        .route(
            "/api/v1/recordings/{id}/playback",
            axum::routing::get(playback_rec),
        )
        .route(
            "/api/v1/recordings/{id}/presign",
            axum::routing::get(presign_rec),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
