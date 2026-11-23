//! Recordings router — mirrors `app/recordings` + `app/call_recordings`.
//!
//! Order: models → schemas → handlers → xml (S3 stub note).
//!
//! Storage backend is S3 (`recordings/{domain_id}/...` per models.py).
//! `GET /presign` returns a documented stub URL (no S3 calls in tests).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

// ---- models ----
use pbx_db::entities::recordings;

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct RecordingCreate {
    #[serde(default)]
    name: String,
    #[serde(default, alias = "recording_name")]
    recording_name: Option<String>,
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default, alias = "recording_filename")]
    recording_filename: Option<String>,
    #[serde(default)] recording_voice: Option<String>,
    #[serde(default)] recording_description: Option<String>,
    #[serde(default, alias = "description")]
    description: Option<String>,
    #[serde(default)] recording_speed: Option<String>,
    #[serde(default)] recording_message: Option<String>,
    #[serde(default)] recording_base64: Option<String>,
}

// ---- handlers ----
async fn create_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<RecordingCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_add", "call_recording_add"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let path = payload
        .file_path
        .or(payload.recording_filename.clone())
        .unwrap_or_else(|| format!("recordings/{domain}/{id}.wav"));
    let name = if payload.name.trim().is_empty() {
        payload.recording_name.clone().unwrap_or_default()
    } else {
        payload.name.trim().to_owned()
    };
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        )
            .into_response();
    }
    let am = recordings::ActiveModel {
        recording_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(name.clone()),
        file_path: Set(path.clone()),
        storage_backend: Set("s3".to_owned()),
        recording_voice: Set(payload.recording_voice.clone()),
        recording_description: Set(payload.recording_description.or(payload.description.clone())),
        recording_speed: Set(payload.recording_speed.clone()),
        recording_message: Set(payload.recording_message.clone()),
        recording_base64: Set(payload.recording_base64.clone()),
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
    match rec_or_404(db, domain, &id).await {
        Some(r) => (StatusCode::CREATED, Json(ser_rec(&r))).into_response(),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response(),
    }
}

async fn list_rec(Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_view", "call_recording_view"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all_any(
        &claims,
        show_q.show.as_deref(),
        &["recording_all", "call_recording_all"],
    );
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = recordings::Entity::find();
    if !show_all {
        sel = sel.filter(recordings::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows.iter().map(ser_rec).collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn presign_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_view", "call_recording_view"]) {
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
    let mut v = serde_json::json!({
        "recording_uuid": r.recording_uuid,
        "name": r.name,
        "recording_name": r.name,
        "file_path": r.file_path,
        "recording_filename": r.file_path,
        "storage_backend": r.storage_backend,
        "recording_voice": r.recording_voice,
        "recording_description": r.recording_description,
        "description": r.recording_description,
        "recording_speed": r.recording_speed,
        "recording_message": r.recording_message,
        "recording_base64": r.recording_base64,
    });
    crate::alias::emit_aliases(&mut v);
    v
}

async fn get_rec(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_view", "call_recording_view"]) {
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
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_view", "call_recording_view"]) {
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
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_edit", "call_recording_edit"]) {
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
    if let Some(v) = payload.get("name").or_else(|| payload.get("recording_name")).and_then(|v| v.as_str()) {
        if !v.trim().is_empty() {
            am.name = Set(v.to_owned());
        }
    }
    if let Some(v) = payload.get("file_path").or_else(|| payload.get("recording_filename")).and_then(|v| v.as_str()) {
        am.file_path = Set(v.to_owned());
    }
    for key in ["recording_voice", "voice", "recording_speed", "recording_message", "recording_base64", "recording_description", "description"] {
        if let Some(v) = payload.get(key).and_then(|v| v.as_str()) {
            match key {
                "recording_voice" | "voice" => am.recording_voice = Set(Some(v.to_owned())),
                "recording_speed" => am.recording_speed = Set(Some(v.to_owned())),
                "recording_message" => am.recording_message = Set(Some(v.to_owned())),
                "recording_base64" => am.recording_base64 = Set(Some(v.to_owned())),
                _ => am.recording_description = Set(Some(v.to_owned())),
            }
        }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_delete", "call_recording_delete"]) {
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
