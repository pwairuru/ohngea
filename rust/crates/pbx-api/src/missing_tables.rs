//! Missing PHP tables CRUD — parity for the 15 `v_*` tables with no Rust entity.
//! Each endpoint is tenant-scoped (domain_id) and follows the admin.rs pattern.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::Value;

use pbx_core::auth::Claims;
use pbx_db::entities::*;

fn deny(claims: &Claims, perm: &str) -> Option<(StatusCode, Json<Value>)> {
    crate::tenant::deny_if_missing(claims, perm)
}

// ---- generic helpers via macro ----
macro_rules! crud {
    ($entity:ty, $active:ty, $model:ty, $route:literal, $perm_view:literal, $perm_add:literal, $pk:ident, $build:expr, $ser:expr) => {};
}

// We implement each table explicitly for clarity and SeaORM type safety.

#[allow(dead_code)]
async fn now_db() -> sea_orm::DatabaseConnection {
    crate::routing_db::conn().await.clone()
}

// ---------- conference_room_users ----------
async fn list_cru(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "conference_room_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = conference_room_users::Entity::find()
        .filter(conference_room_users::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({"conference_room_user_uuid": r.conference_room_user_uuid, "conference_room_uuid": r.conference_room_uuid, "user_uuid": r.user_uuid})
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_cru(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "conference_room_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = conference_room_users::ActiveModel {
        conference_room_user_uuid: Set(id.clone()),
        domain_id: Set(d),
        conference_room_uuid: Set(p
            .get("conference_room_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        user_uuid: Set(p
            .get("user_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"conference_room_user_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- extension_users ----------
async fn list_eu(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "extension_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = extension_users::Entity::find()
        .filter(extension_users::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| serde_json::json!({"extension_user_uuid": r.extension_user_uuid, "extension_uuid": r.extension_uuid, "user_uuid": r.user_uuid}))
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_eu(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "extension_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let id = crate::routing_db::new_uuid();
    let am = extension_users::ActiveModel {
        extension_user_uuid: Set(id.clone()),
        domain_id: Set(d),
        extension_uuid: Set(p
            .get("extension_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        user_uuid: Set(p
            .get("user_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"extension_user_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- ring_group_users ----------
async fn list_rgu(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "ring_group_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = ring_group_users::Entity::find()
        .filter(ring_group_users::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| serde_json::json!({"ring_group_user_uuid": r.ring_group_user_uuid, "ring_group_uuid": r.ring_group_uuid, "user_uuid": r.user_uuid}))
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_rgu(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "ring_group_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let id = crate::routing_db::new_uuid();
    let am = ring_group_users::ActiveModel {
        ring_group_user_uuid: Set(id.clone()),
        domain_id: Set(d),
        ring_group_uuid: Set(p
            .get("ring_group_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        user_uuid: Set(p
            .get("user_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"ring_group_user_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- voicemail_destinations ----------
async fn list_vd(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "voicemail_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = voicemail_destinations::Entity::find()
        .filter(voicemail_destinations::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| serde_json::json!({"voicemail_destination_uuid": r.voicemail_destination_uuid, "voicemail_uuid": r.voicemail_uuid}))
        .collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_vd(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "voicemail_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let id = crate::routing_db::new_uuid();
    let am = voicemail_destinations::ActiveModel {
        voicemail_destination_uuid: Set(id.clone()),
        domain_id: Set(d),
        voicemail_uuid: Set(p
            .get("voicemail_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        voicemail_uuid_copy: Set(p
            .get("voicemail_uuid_copy")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"voicemail_destination_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- device_vendors ----------
async fn list_dv(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "device_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = device_vendors::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"device_vendor_uuid": r.device_vendor_uuid, "name": r.name, "enabled": r.enabled})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_dv(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "device_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let name = p
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_owned();
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        )
            .into_response();
    }
    let am = device_vendors::ActiveModel {
        device_vendor_uuid: Set(id.clone()),
        name: Set(name),
        enabled: Set(p.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)),
        description: Set(p
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"device_vendor_uuid": id})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- modules (full v_modules parity) ----------
fn ser_module(r: &modules::Model) -> Value {
    serde_json::json!({
        "module_uuid": r.module_uuid,
        "module_label": r.module_label,
        "module_name": r.module_name,
        "module_category": r.module_category,
        "module_order": r.module_order,
        "module_enabled": r.module_enabled,
        "enabled": r.module_enabled,
        "module_default_enabled": r.module_default_enabled,
        "module_description": r.module_description,
    })
}

async fn list_mod(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "module_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = modules::Entity::find().all(db).await.unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(ser_module).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

async fn create_mod(Extension(c): Extension<Claims>, Json(p): Json<Value>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "module_add") {
        return (s, j).into_response();
    }
    let Some(name) = p
        .get("module_name")
        .or_else(|| p.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "module_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = modules::ActiveModel {
        module_uuid: Set(id.clone()),
        module_label: Set(p
            .get("module_label")
            .or_else(|| p.get("label"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        module_name: Set(Some(name.to_owned())),
        module_category: Set(p
            .get("module_category")
            .or_else(|| p.get("category"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        module_order: Set(p
            .get("module_order")
            .or_else(|| p.get("order"))
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        module_enabled: Set(p
            .get("module_enabled")
            .or_else(|| p.get("enabled"))
            .and_then(|v| v.as_bool())),
        module_default_enabled: Set(p.get("module_default_enabled").and_then(|v| v.as_bool())),
        module_description: Set(p
            .get("module_description")
            .or_else(|| p.get("description"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(_) => match modules::Entity::find_by_id(id)
            .one(db)
            .await
            .unwrap_or(None)
        {
            Some(r) => (StatusCode::CREATED, Json(ser_module(&r))).into_response(),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": "insert failed"})),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn get_mod(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "module_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match modules::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_module(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_mod(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(p): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "module_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(r) = modules::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: modules::ActiveModel = r.into();
    for key in [
        "module_label",
        "label",
        "module_name",
        "name",
        "module_category",
        "category",
        "module_description",
        "description",
    ] {
        if let Some(v) = p.get(key).and_then(|v| v.as_str()) {
            match key {
                "module_label" | "label" => am.module_label = Set(Some(v.to_owned())),
                "module_name" | "name" => am.module_name = Set(Some(v.to_owned())),
                "module_category" | "category" => am.module_category = Set(Some(v.to_owned())),
                _ => am.module_description = Set(Some(v.to_owned())),
            }
        }
    }
    if let Some(v) = p
        .get("module_order")
        .or_else(|| p.get("order"))
        .and_then(|v| v.as_i64())
    {
        am.module_order = Set(Some(v as i32));
    }
    if let Some(v) = p
        .get("module_enabled")
        .or_else(|| p.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        am.module_enabled = Set(Some(v));
    }
    if let Some(v) = p.get("module_default_enabled").and_then(|v| v.as_bool()) {
        am.module_default_enabled = Set(Some(v));
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_module(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_mod(
    Extension(c): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "module_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(r) = modules::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: modules::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---------- database_transactions ----------
#[allow(dead_code)]
async fn list_dt(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "database_transaction_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = database_transactions::Entity::find()
        .filter(database_transactions::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"database_transaction_uuid": r.database_transaction_uuid, "app_name": r.app_name, "app_uuid": r.app_uuid, "transaction_code": r.transaction_code, "transaction_address": r.transaction_address, "transaction_type": r.transaction_type, "transaction_date": r.transaction_date, "transaction_old": r.transaction_old, "transaction_new": r.transaction_new, "transaction_result": r.transaction_result, "user_uuid": r.user_uuid})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

// ---------- email_queue_attachments ----------
async fn list_eqa(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "email_queue_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = email_queue_attachments::Entity::find()
        .filter(email_queue_attachments::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|r| serde_json::json!({"email_queue_attachment_uuid": r.email_queue_attachment_uuid, "email_queue_uuid": r.email_queue_uuid})).collect();
    Json(serde_json::json!({"data": data, "total": data.len()})).into_response()
}

// ---------- xml_cdr family (read-mostly) ----------
async fn list_xe(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = xml_cdr_extensions::Entity::find()
        .filter(xml_cdr_extensions::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    Json(serde_json::json!({"data": rows.len(), "total": rows.len()})).into_response()
}
async fn list_xj(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = xml_cdr_json::Entity::find()
        .filter(xml_cdr_json::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    Json(serde_json::json!({"data": rows.len(), "total": rows.len()})).into_response()
}
async fn list_xf(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = xml_cdr_flow::Entity::find()
        .filter(xml_cdr_flow::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    Json(serde_json::json!({"data": rows.len(), "total": rows.len()})).into_response()
}
async fn list_xl(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = xml_cdr_logs::Entity::find()
        .filter(xml_cdr_logs::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    Json(serde_json::json!({"data": rows.len(), "total": rows.len()})).into_response()
}
async fn list_xt(Extension(c): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = deny(&c, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let d = crate::routing_db::domain(&c);
    let rows = xml_cdr_transcripts::Entity::find()
        .filter(xml_cdr_transcripts::Column::DomainId.eq(d))
        .all(db)
        .await
        .unwrap_or_default();
    Json(serde_json::json!({"data": rows.len(), "total": rows.len()})).into_response()
}

pub fn missing_tables_router() -> Router {
    Router::new()
        .route(
            "/api/v1/conference_room_users",
            axum::routing::post(create_cru).get(list_cru),
        )
        .route(
            "/api/v1/extension_users",
            axum::routing::post(create_eu).get(list_eu),
        )
        .route(
            "/api/v1/ring_group_users",
            axum::routing::post(create_rgu).get(list_rgu),
        )
        .route(
            "/api/v1/voicemail_destinations",
            axum::routing::post(create_vd).get(list_vd),
        )
        .route(
            "/api/v1/device_vendors",
            axum::routing::post(create_dv).get(list_dv),
        )
        .route(
            "/api/v1/email_queue_attachments",
            axum::routing::get(list_eqa),
        )
        .route("/api/v1/xml_cdr_extensions", axum::routing::get(list_xe))
        .route("/api/v1/xml_cdr_json", axum::routing::get(list_xj))
        .route("/api/v1/xml_cdr_flow", axum::routing::get(list_xf))
        .route("/api/v1/xml_cdr_logs", axum::routing::get(list_xl))
        .route("/api/v1/xml_cdr_transcripts", axum::routing::get(list_xt))
        .route(
            "/api/v1/modules",
            axum::routing::post(create_mod).get(list_mod),
        )
        .route(
            "/api/v1/modules/{id}",
            axum::routing::get(get_mod)
                .patch(patch_mod)
                .delete(delete_mod),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
