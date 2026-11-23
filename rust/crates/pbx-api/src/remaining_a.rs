//! Spillover replacement (part A): per-table CRUD for call_block,
//! call_broadcast(s), bridges, destinations, follow_me, call_forward and
//! time_conditions — SeaORM entities with Python-model columns instead of
//! `remaining_records` JSON spillover.
//!
//! Order: models → schemas → handlers → registry (per-file section).
//! Shared `s/b/i` payload helpers live here as `pub(crate)`.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde_json::Value;

// ---- models ----
type Claims = pbx_core::auth::Claims;
use pbx_db::entities::{
    call_block, call_broadcasts, call_forwards, destinations, follow_me, follow_me_destinations,
    time_conditions,
};

// ---- schemas (payload helpers; flexible legacy aliases) ----
pub(crate) fn s(v: &Value, keys: &[&str]) -> Option<String> {    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(b) = x.as_str() {
                let t = b.trim();
                if !t.is_empty() {
                    return Some(t.to_owned());
                }
            } else if x.is_number() {
                return Some(x.to_string());
            }
        }
    }
    None
}

pub(crate) fn b(v: &Value, keys: &[&str], dflt: bool) -> bool {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(bb) = x.as_bool() {
                return bb;
            }
            if let Some(n) = x.as_i64() {
                return n != 0;
            }
            if let Some(st) = x.as_str() {
                match st.trim().to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => return true,
                    "false" | "0" | "no" | "off" => return false,
                    _ => {}
                }
            }
        }
    }
    dflt
}

pub(crate) fn b_opt(v: &Value, keys: &[&str]) -> Option<bool> {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(bb) = x.as_bool() {
                return Some(bb);
            }
            if let Some(n) = x.as_i64() {
                return Some(n != 0);
            }
        }
    }
    None
}

pub(crate) fn i(v: &Value, keys: &[&str], dflt: i32) -> i32 {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(n) = x.as_i64() {
                return n as i32;
            }
            if let Some(st) = x.as_str() {
                if let Ok(n) = st.trim().parse::<i32>() {
                    return n;
                }
            }
        }
    }
    dflt
}

pub(crate) fn unprocessable(msg: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({"detail": msg})),
    )
}

pub(crate) fn envelope(items: Vec<Value>) -> Json<Value> {
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total}))
}

pub(crate) async fn tenant_domain(db: &sea_orm::DatabaseConnection, claims: &Claims) -> uuid::Uuid {
    crate::routing_db::tenant_domain(db, claims).await
}

// ---- handlers: call_block ----
fn ser_call_block(r: &call_block::Model) -> Value {
    serde_json::json!({
        "call_block_uuid": r.call_block_uuid,
        "domain_id": r.domain_id.map(|d| d.to_string()),
        "call_block_direction": r.call_block_direction,
        "extension_uuid": r.extension_uuid,
        "call_block_name": r.call_block_name,
        "name": r.call_block_name,
        "call_block_country_code": r.call_block_country_code,
        "call_block_number": r.call_block_number,
        "number": r.call_block_number,
        "call_block_enabled": r.call_block_enabled,
        "enabled": r.call_block_enabled,
        "call_block_description": r.call_block_description,
        "call_block_action": r.call_block_action,
        "call_block_count": r.call_block_count,
        "call_block_app": r.call_block_app,
        "call_block_data": r.call_block_data,
    })
}

async fn create_call_block(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_block_add") {
        return (s, j).into_response();
    }
    let Some(number) = s(&payload, &["call_block_number", "number"]) else {
        return unprocessable("number required").into_response();
    };
    let name = s(&payload, &["call_block_name", "name"]).unwrap_or_else(|| number.clone());
    let db = crate::routing_db::conn().await;
    let tenant = tenant_domain(db, &claims).await;
    let posted_domain = payload
        .get("domain_uuid")
        .or_else(|| payload.get("domain_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .map(str::to_owned);
    // PHP parity: `call_block_domain` holders post any domain (empty = global).
    // NOTE: raw access, not s() — s() drops empty strings.
    let domain: Option<uuid::Uuid> = match posted_domain.as_deref() {
        Some(v) if !v.is_empty() && crate::tenant::has_permission(&claims, "call_block_domain") => {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => Some(id),
                Err(_) => return unprocessable("domain_uuid must be a UUID or empty").into_response(),
            }
        }
        Some(v) if v.is_empty() && crate::tenant::has_permission(&claims, "call_block_domain") => None,
        _ => Some(tenant),
    };
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = call_block::ActiveModel {
        call_block_uuid: Set(id.clone()),
        domain_id: Set(domain),
        call_block_direction: Set(s(&payload, &["call_block_direction"])),
        call_block_name: Set(name),
        call_block_country_code: Set(s(&payload, &["call_block_country_code"])),
        call_block_number: Set(number),
        call_block_enabled: Set(b(&payload, &["call_block_enabled", "enabled"], true)),
        call_block_description: Set(s(&payload, &["call_block_description", "description"])),
        call_block_action: Set(s(&payload, &["call_block_action", "action"])),
        call_block_app: Set(s(&payload, &["call_block_app", "app"])),
        call_block_data: Set(s(&payload, &["call_block_data", "data"])),
        call_block_count: Set(Some(i(&payload, &["call_block_count"], 0))),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_call_block(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_call_block(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_block_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "call_block_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = call_block::Entity::find();
    if !show_all {
        // PHP `call_block_domain` parity: holders also see global rows.
        if crate::tenant::sees_domain_nulls(&claims, "call_block_domain") {
            sel = sel.filter(
                Condition::any()
                    .add(call_block::Column::DomainId.eq(domain))
                    .add(call_block::Column::DomainId.is_null()),
            );
        } else {
            sel = sel.filter(call_block::Column::DomainId.eq(domain));
        }
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_call_block).collect()).into_response()
}

async fn get_call_block(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_block_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_block::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r)
        if r.domain_id == Some(domain)
            || (r.domain_id.is_none()
                && crate::tenant::sees_domain_nulls(&claims, "call_block_domain")) => {
            (StatusCode::OK, Json(ser_call_block(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_call_block(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_block_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = call_block::Entity::find_by_id(id)
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
    if r.domain_id != Some(domain)
        && (r.domain_id.is_some()
            || !crate::tenant::sees_domain_nulls(&claims, "call_block_domain"))
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: call_block::ActiveModel = r.into();
    // PHP `call_block_domain` parity: perm holders may re-home the row (UUID string)
    // or globalize it (empty string); otherwise the domain is untouched.
    if payload.get("domain_uuid").or_else(|| payload.get("domain_id")).is_some()
        && crate::tenant::sees_domain_nulls(&claims, "call_block_domain")
    {
        let raw = payload
            .get("domain_uuid")
            .or_else(|| payload.get("domain_id"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if raw.is_empty() {
            am.domain_id = Set(None);
        } else {
            match raw.parse::<uuid::Uuid>() {
                Ok(id) => am.domain_id = Set(Some(id)),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID or empty"})),
                    )
                        .into_response();
                }
            }
        }
    }
    if let Some(v) = s(&payload, &["call_block_name", "name"]) {
        am.call_block_name = Set(v);
    }
    if let Some(v) = s(&payload, &["call_block_number", "number"]) {
        am.call_block_number = Set(v);
    }
    if payload
        .get("call_block_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.call_block_enabled = Set(b(&payload, &["call_block_enabled", "enabled"], true));
    }
    if let Some(v) = s(&payload, &["call_block_description", "description"]) {
        am.call_block_description = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["call_block_action", "action"]) {
        am.call_block_action = Set(Some(v));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_call_block(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_call_block(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_block_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_block::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r)
        if r.domain_id == Some(domain)
            || (r.domain_id.is_none()
                && crate::tenant::sees_domain_nulls(&claims, "call_block_domain")) => {
            let am: call_block::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn call_block_router() -> Router {
    let item = "/api/v1/call_block/{id}";
    let r = Router::new()
        .route(
            "/api/v1/call_block",
            axum::routing::post(create_call_block).get(list_call_block),
        )
        .route(
            item,
            axum::routing::get(get_call_block)
                .patch(patch_call_block)
                .delete(delete_call_block),
        );
    // Legacy trailing-slash alias (existing codebase style).
    r.route(
        "/api/v1/call_block/",
        axum::routing::post(create_call_block).get(list_call_block),
    )
}

// ---- handlers: call_broadcast(s) ----
fn ser_broadcast(r: &call_broadcasts::Model) -> Value {
    serde_json::json!({
        "call_broadcast_uuid": r.call_broadcast_uuid,
        "domain_id": r.domain_id.to_string(),
        "broadcast_name": r.broadcast_name,
        "name": r.broadcast_name,
        "broadcast_description": r.broadcast_description,
        "broadcast_timeout": r.broadcast_timeout,
        "broadcast_concurrent_limit": r.broadcast_concurrent_limit,
        "broadcast_caller_id_name": r.broadcast_caller_id_name,
        "broadcast_caller_id_number": r.broadcast_caller_id_number,
        "broadcast_destination_type": r.broadcast_destination_type,
        "broadcast_destination_data": r.broadcast_destination_data,
        "broadcast_phone_numbers": r.broadcast_phone_numbers,
        "broadcast_avmd": r.broadcast_avmd,
        "broadcast_accountcode": r.broadcast_accountcode,
        "broadcast_toll_allow": r.broadcast_toll_allow,
        "recording_uuid": r.recording_uuid,
        "broadcast_start_time": r.broadcast_start_time,
        "broadcast_enabled": r.broadcast_enabled,
        "enabled": r.broadcast_enabled,
    })
}

async fn create_broadcast(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["broadcast_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = call_broadcasts::ActiveModel {
        call_broadcast_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        broadcast_name: Set(name),
        broadcast_description: Set(s(&payload, &["broadcast_description", "description"])),
        broadcast_timeout: Set(Some(i(&payload, &["broadcast_timeout"], 30))),
        broadcast_concurrent_limit: Set(Some(i(&payload, &["broadcast_concurrent_limit"], 1))),
        broadcast_caller_id_name: Set(s(&payload, &["broadcast_caller_id_name"])),
        broadcast_caller_id_number: Set(s(&payload, &["broadcast_caller_id_number"])),
        broadcast_destination_type: Set(s(&payload, &["broadcast_destination_type"])),
        broadcast_destination_data: Set(s(&payload, &["broadcast_destination_data"])),
        broadcast_phone_numbers: Set(s(&payload, &["broadcast_phone_numbers"])),
        broadcast_avmd: Set(b_opt(&payload, &["broadcast_avmd", "avmd"])),
        broadcast_accountcode: Set(s(&payload, &["broadcast_accountcode", "accountcode"])),
        broadcast_toll_allow: Set(s(&payload, &["broadcast_toll_allow", "toll_allow"])),
        recording_uuid: Set(s(&payload, &["recording_uuid"])),
        broadcast_start_time: Set(payload.get("broadcast_start_time").or_else(|| payload.get("start_time")).and_then(|v| v.as_i64())),
        broadcast_enabled: Set(b(&payload, &["broadcast_enabled", "enabled"], true)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_broadcast(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_broadcast(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "call_broadcast_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = call_broadcasts::Entity::find();
    if !show_all {
        sel = sel.filter(call_broadcasts::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_broadcast).collect()).into_response()
}

async fn get_broadcast(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_broadcasts::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            (StatusCode::OK, Json(ser_broadcast(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_broadcast(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = call_broadcasts::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: call_broadcasts::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["broadcast_name", "name"]) {
        am.broadcast_name = Set(v);
    }
    if payload
        .get("broadcast_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.broadcast_enabled = Set(b(&payload, &["broadcast_enabled", "enabled"], true));
    }
    if let Some(v) = s(&payload, &["broadcast_description", "description"]) {
        am.broadcast_description = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["broadcast_destination_type"]) {
        am.broadcast_destination_type = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["broadcast_destination_data"]) {
        am.broadcast_destination_data = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["broadcast_phone_numbers"]) {
        am.broadcast_phone_numbers = Set(Some(v));
    }
    if let Some(v) = b_opt(&payload, &["broadcast_avmd", "avmd"]) {
        am.broadcast_avmd = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["broadcast_accountcode", "accountcode"]) {
        am.broadcast_accountcode = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["broadcast_toll_allow", "toll_allow"]) {
        am.broadcast_toll_allow = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["recording_uuid"]) {
        am.recording_uuid = Set(Some(v));
    }
    if let Some(v) = payload.get("broadcast_start_time").or_else(|| payload.get("start_time")).and_then(|v| v.as_i64()) {
        am.broadcast_start_time = Set(Some(v));
    }
    if let Some(v) = payload.get("broadcast_timeout").and_then(|v| v.as_i64()) {
        am.broadcast_timeout = Set(Some(v as i32));
    }
    if let Some(v) = payload.get("broadcast_concurrent_limit").and_then(|v| v.as_i64()) {
        am.broadcast_concurrent_limit = Set(Some(v as i32));
    }
    if let Some(v) = s(&payload, &["broadcast_caller_id_name"]) {
        am.broadcast_caller_id_name = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["broadcast_caller_id_number"]) {
        am.broadcast_caller_id_number = Set(Some(v));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_broadcast(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_broadcast(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_broadcasts::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: call_broadcasts::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

/// POST `/{id}/send` — start a broadcast campaign (parity:
/// `app/call_broadcast/router.py::send_ep`). Enables the row, then originates
/// to each listed number via ESL `bgapi` (best effort, mock-fallback offline).
async fn send_broadcast(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    use pbx_fs::esl::EsClient;
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = call_broadcasts::Entity::find_by_id(id.clone())
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call broadcast not found"})),
        )
            .into_response();
    };
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call broadcast not found"})),
        )
            .into_response();
    }
    let (Some(dest_type), Some(dest_data)) = (
        r.broadcast_destination_type.clone(),
        r.broadcast_destination_data.clone(),
    ) else {
        return unprocessable(
            "broadcast_destination_type and broadcast_destination_data required to send",
        )
        .into_response();
    };
    if dest_type.trim().is_empty() || dest_data.trim().is_empty() {
        return unprocessable(
            "broadcast_destination_type and broadcast_destination_data required to send",
        )
        .into_response();
    }
    let mut am: call_broadcasts::ActiveModel = r.into();
    am.broadcast_enabled = Set(true);
    am.updated_at = Set(crate::routing_db::now());
    let Ok(r) = am.update(db).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "update failed"})),
        )
            .into_response();
    };
    let esl = pbx_fs::esl::RealEs::from_env();
    let caller = r.broadcast_caller_id_number.clone().unwrap_or_default();
    let numbers: Vec<String> = r
        .broadcast_phone_numbers
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .take(100)
        .map(str::to_owned)
        .collect();
    let results: Vec<Value> = numbers
        .iter()
        .map(|n| {
            let cmd = format!(
                "originate {{origination_caller_id_number={caller}}}user/{n} &transfer({dest_data} {dest_type})"
            );
            let reply: String = esl.bgapi(&cmd);
            let short: String = reply.chars().take(300).collect();
            serde_json::json!({"number": n, "result": short})
        })
        .collect();
    let attempted = results.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "call_broadcast_uuid": r.call_broadcast_uuid,
            "broadcast_enabled": r.broadcast_enabled,
            "send": {"attempted": attempted, "results": results},
        })),
    )
        .into_response()
}

/// POST `/{id}/stop` — stop a broadcast campaign (parity:
/// `app/call_broadcast/router.py::stop_ep`). Disables the row so no further
/// legs originate.
async fn stop_broadcast(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_broadcast_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = call_broadcasts::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call broadcast not found"})),
        )
            .into_response();
    };
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Call broadcast not found"})),
        )
            .into_response();
    }
    let mut am: call_broadcasts::ActiveModel = r.into();
    am.broadcast_enabled = Set(false);
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "call_broadcast_uuid": r.call_broadcast_uuid,
                "broadcast_enabled": r.broadcast_enabled,
                "send": {"attempted": 0, "stopped": true},
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

fn broadcast_router() -> Router {
    // Legacy `/api/v1/call_broadcast` kept; Python-canonical
    // `/api/v1/call_broadcasts` mounted alongside.
    Router::new()
        .route(
            "/api/v1/call_broadcast",
            axum::routing::post(create_broadcast).get(list_broadcast),
        )
        .route(
            "/api/v1/call_broadcast/{id}",
            axum::routing::get(get_broadcast)
                .patch(patch_broadcast)
                .delete(delete_broadcast),
        )
        .route(
            "/api/v1/call_broadcast/{id}/send",
            axum::routing::post(send_broadcast),
        )
        .route(
            "/api/v1/call_broadcast/{id}/stop",
            axum::routing::post(stop_broadcast),
        )
        .route(
            "/api/v1/call_broadcasts",
            axum::routing::post(create_broadcast).get(list_broadcast),
        )
        .route(
            "/api/v1/call_broadcasts/{id}",
            axum::routing::get(get_broadcast)
                .patch(patch_broadcast)
                .delete(delete_broadcast),
        )
        .route(
            "/api/v1/call_broadcasts/{id}/send",
            axum::routing::post(send_broadcast),
        )
        .route(
            "/api/v1/call_broadcasts/{id}/stop",
            axum::routing::post(stop_broadcast),
        )
}

// ---- handlers: bridges ----
fn ser_bridge(r: &pbx_db::entities::bridges::Model) -> Value {
    serde_json::json!({
        "bridge_uuid": r.bridge_uuid,
        "domain_id": r.domain_id.to_string(),
        "bridge_name": r.bridge_name,
        "name": r.bridge_name,
        "bridge_destination": r.bridge_destination,
        "destination": r.bridge_destination,
        "bridge_enabled": r.bridge_enabled,
        "enabled": r.bridge_enabled,
        "bridge_description": r.bridge_description,
    })
}

fn compose_bridge_destination(payload: &Value) -> Option<String> {
    if let Some(d) = s(payload, &["bridge_destination", "destination"]) {
        return Some(d);
    }
    let num = s(payload, &["destination_number", "number"])?;
    match s(payload, &["bridge_action", "action"]).as_deref() {
        Some("gateway") => {
            let gw = s(payload, &["bridge_gateways", "gateway"]).unwrap_or_default();
            Some(format!("sofia/gateway/{gw}/{num}"))
        }
        Some("profile") => {
            let prof = s(payload, &["bridge_profile", "profile"]).unwrap_or_default();
            Some(format!("sofia/{prof}/{num}"))
        }
        Some(a) => Some(format!("{a}/{num}")),
        None => Some(num),
    }
}

async fn create_bridge(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "bridge_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["bridge_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = pbx_db::entities::bridges::ActiveModel {
        bridge_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        bridge_name: Set(name),
        bridge_destination: Set(compose_bridge_destination(&payload)),
        bridge_enabled: Set(b(&payload, &["bridge_enabled", "enabled"], true)),
        bridge_description: Set(s(&payload, &["bridge_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_bridge(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_bridges(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "bridge_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "bridge_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = pbx_db::entities::bridges::Entity::find();
    if !show_all {
        sel = sel.filter(pbx_db::entities::bridges::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_bridge).collect()).into_response()
}

async fn get_bridge(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "bridge_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match pbx_db::entities::bridges::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_bridge(&r))).into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_bridge(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "bridge_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = pbx_db::entities::bridges::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: pbx_db::entities::bridges::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["bridge_name", "name"]) {
        am.bridge_name = Set(v);
    }
    if let Some(v) = compose_bridge_destination(&payload) {
        am.bridge_destination = Set(Some(v));
    }
    if payload
        .get("bridge_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.bridge_enabled = Set(b(&payload, &["bridge_enabled", "enabled"], true));
    }
    if let Some(v) = s(&payload, &["bridge_description", "description"]) {
        am.bridge_description = Set(Some(v));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_bridge(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_bridge(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "bridge_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match pbx_db::entities::bridges::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: pbx_db::entities::bridges::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn bridges_router() -> Router {
    Router::new()
        .route(
            "/api/v1/bridges",
            axum::routing::post(create_bridge).get(list_bridges),
        )
        .route(
            "/api/v1/bridges/",
            axum::routing::post(create_bridge).get(list_bridges),
        )
        .route(
            "/api/v1/bridges/{id}",
            axum::routing::get(get_bridge)
                .patch(patch_bridge)
                .delete(delete_bridge),
        )
}

// ---- handlers: destinations ----
fn ser_destination(r: &destinations::Model) -> Value {
    let mut o = serde_json::Map::new();
    let mut put = |k: &str, v: Value| {
        o.insert(k.to_owned(), v);
    };
    let s = |x: &Option<String>| x.clone().map(Value::String).unwrap_or(Value::Null);
    put("destination_uuid", Value::String(r.destination_uuid.clone()));
    put("domain_id", r.domain_id.map(|d| Value::String(d.to_string())).unwrap_or(Value::Null));
    put("destination_type", Value::String(r.destination_type.clone()));
    put("type", Value::String(r.destination_type.clone()));
    put("destination_number", Value::String(r.destination_number.clone()));
    put("destination", Value::String(r.destination_number.clone()));
    put("number", Value::String(r.destination_number.clone()));
    put("destination_prefix", s(&r.destination_prefix));
    put("destination_enabled", Value::Bool(r.destination_enabled));
    put("enabled", Value::Bool(r.destination_enabled));
    put("destination_description", s(&r.destination_description));
    put("destination_context", s(&r.destination_context));
    put("destination_app", s(&r.destination_app));
    put("action", s(&r.destination_app));
    put("destination_data", s(&r.destination_data));
    put("data", s(&r.destination_data));
    put("destination_caller_id_name", s(&r.destination_caller_id_name));
    put("destination_caller_id_number", s(&r.destination_caller_id_number));
    put("destination_caller_id", s(&r.destination_caller_id));
    put("destination_hold_music", s(&r.destination_hold_music));
    put("destination_record", r.destination_record.map(Value::Bool).unwrap_or(Value::Null));
    put("destination_cid_name_prefix", s(&r.destination_cid_name_prefix));
    put("destination_order", r.destination_order.map(Value::from).unwrap_or(Value::Null));
    put("destination_email", s(&r.destination_email));
    put("dialplan_uuid", s(&r.dialplan_uuid));
    put("fax_uuid", s(&r.fax_uuid));
    put("user_uuid", s(&r.user_uuid));
    put("group_uuid", s(&r.group_uuid));
    put("provider_uuid", s(&r.provider_uuid));
    put("destination_trunk_prefix", s(&r.destination_trunk_prefix));
    put("destination_area_code", s(&r.destination_area_code));
    put("destination_condition_field", s(&r.destination_condition_field));
    put("destination_number_regex", s(&r.destination_number_regex));
    put("destination_distinctive_ring", s(&r.destination_distinctive_ring));
    put("destination_ringback", s(&r.destination_ringback));
    put("destination_accountcode", s(&r.destination_accountcode));
    put("destination_type_voice", r.destination_type_voice.map(Value::Bool).unwrap_or(Value::Null));
    put("destination_type_fax", r.destination_type_fax.map(Value::Bool).unwrap_or(Value::Null));
    put("destination_type_emergency", r.destination_type_emergency.map(Value::Bool).unwrap_or(Value::Null));
    put("destination_type_text", r.destination_type_text.map(Value::Bool).unwrap_or(Value::Null));
    put("destination_conditions", s(&r.destination_conditions));
    put("destination_actions", s(&r.destination_actions));
    put("destination_alternate_app", s(&r.destination_alternate_app));
    put("destination_alternate_data", s(&r.destination_alternate_data));
    Value::Object(o)
}

async fn create_destination(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "destination_add") {
        return (s, j).into_response();
    }
    let Some(number) = s(&payload, &["destination_number", "destination", "number"]) else {
        return unprocessable("destination required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let tenant = tenant_domain(db, &claims).await;
    // PHP `destination_domain` parity: holders may assign another tenant
    // (valid UUID only — empty falls back, never global via this path).
    let domain: Option<uuid::Uuid> = match s(&payload, &["domain_uuid", "domain_id"])
        .map(|v| v.trim().to_owned())
    {
        Some(v) if !v.is_empty() && crate::tenant::has_permission(&claims, "destination_domain") => {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => Some(id),
                Err(_) => return unprocessable("domain_uuid must be a UUID").into_response(),
            }
        }
        _ => Some(tenant),
    };
    let now = crate::routing_db::now();
    let am = destinations::ActiveModel {
        destination_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        destination_type: Set(
            s(&payload, &["destination_type", "type"]).unwrap_or_else(|| "inbound".to_owned())
        ),
        destination_number: Set(number),
        destination_prefix: Set(s(&payload, &["destination_prefix"])),
        destination_enabled: Set(b(&payload, &["destination_enabled", "enabled"], true)),
        destination_description: Set(s(&payload, &["destination_description", "description"])),
        destination_context: Set(
            s(&payload, &["destination_context"]).or(Some("public".to_owned()))
        ),
        destination_app: Set(s(&payload, &["destination_app", "action", "app"])),
        destination_data: Set(s(&payload, &["destination_data", "data"])),
        destination_caller_id_name: Set(s(&payload, &["destination_caller_id_name"])),
        destination_caller_id_number: Set(s(&payload, &["destination_caller_id_number"])),
        destination_caller_id: Set(s(&payload, &["destination_caller_id"])),
        destination_hold_music: Set(s(&payload, &["destination_hold_music"])),
        destination_record: Set(b_opt(&payload, &["destination_record", "record"])),
        destination_cid_name_prefix: Set(s(&payload, &["destination_cid_name_prefix"])),
        destination_order: Set(payload.get("destination_order").and_then(|v| v.as_i64()).map(|v| v as i32)),
        destination_email: Set(s(&payload, &["destination_email"])),
        dialplan_uuid: Set(s(&payload, &["dialplan_uuid"])),
        fax_uuid: Set(s(&payload, &["fax_uuid"])),
        user_uuid: Set(s(&payload, &["user_uuid"])),
        group_uuid: Set(s(&payload, &["group_uuid"])),
        provider_uuid: Set(s(&payload, &["provider_uuid"])),
        destination_trunk_prefix: Set(s(&payload, &["destination_trunk_prefix", "trunk_prefix"])),
        destination_area_code: Set(s(&payload, &["destination_area_code", "area_code"])),
        destination_condition_field: Set(s(&payload, &["destination_condition_field", "condition_field"])),
        destination_number_regex: Set(s(&payload, &["destination_number_regex", "number_regex"])),
        destination_distinctive_ring: Set(s(&payload, &["destination_distinctive_ring", "distinctive_ring"])),
        destination_ringback: Set(s(&payload, &["destination_ringback", "ringback"])),
        destination_accountcode: Set(s(&payload, &["destination_accountcode", "accountcode"])),
        destination_type_voice: Set(b_opt(&payload, &["destination_type_voice", "type_voice"])),
        destination_type_fax: Set(b_opt(&payload, &["destination_type_fax", "type_fax"])),
        destination_type_emergency: Set(b_opt(&payload, &["destination_type_emergency", "type_emergency"])),
        destination_type_text: Set(b_opt(&payload, &["destination_type_text", "type_text"])),
        destination_conditions: Set(s(&payload, &["destination_conditions", "conditions"])),
        destination_actions: Set(s(&payload, &["destination_actions", "actions"])),
        destination_alternate_app: Set(s(&payload, &["destination_alternate_app", "alternate_app"])),
        destination_alternate_data: Set(s(&payload, &["destination_alternate_data", "alternate_data"])),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_destination(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_destinations(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "destination_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "destination_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = destinations::Entity::find();
    if !show_all {
        // PHP parity: global (NULL-domain) destinations show for everyone.
        sel = sel.filter(
            Condition::any()
                .add(destinations::Column::DomainId.eq(domain))
                .add(destinations::Column::DomainId.is_null()),
        );
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_destination).collect()).into_response()
}

async fn get_destination(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "destination_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match destinations::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == Some(domain) || r.domain_id.is_none() => {
            (StatusCode::OK, Json(ser_destination(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_destination(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "destination_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = destinations::Entity::find_by_id(id)
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
    if r.domain_id != Some(domain) && r.domain_id.is_some() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: destinations::ActiveModel = r.into();
    // PHP `destination_domain` parity: holders may re-home to another
    // tenant (valid UUID only — never global via this path).
    if let Some(v) = payload
        .get("domain_uuid")
        .or_else(|| payload.get("domain_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if crate::tenant::sees_domain_nulls(&claims, "destination_domain") {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => am.domain_id = Set(Some(id)),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID"})),
                    )
                        .into_response();
                }
            }
        }
    }
    if let Some(v) = s(&payload, &["destination_number", "destination", "number"]) {
        am.destination_number = Set(v);
    }
    if let Some(v) = s(&payload, &["destination_type", "type"]) {
        am.destination_type = Set(v);
    }
    if let Some(v) = s(&payload, &["destination_app", "action", "app"]) {
        am.destination_app = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["destination_data", "data"]) {
        am.destination_data = Set(Some(v));
    }
    if payload
        .get("destination_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.destination_enabled = Set(b(&payload, &["destination_enabled", "enabled"], true));
    }
    if let Some(v) = s(&payload, &["destination_description", "description"]) {
        am.destination_description = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["destination_cid_name_prefix"]) {
        am.destination_cid_name_prefix = Set(Some(v));
    }
    if payload.get("destination_order").is_some() {
        am.destination_order = Set(payload.get("destination_order").and_then(|v| v.as_i64()).map(|v| v as i32));
    }
    if let Some(v) = s(&payload, &["destination_email"]) {
        am.destination_email = Set(Some(v));
    }
    for key in [
        "dialplan_uuid", "fax_uuid", "user_uuid", "group_uuid", "provider_uuid",
        "destination_trunk_prefix", "destination_area_code", "destination_condition_field",
        "destination_number_regex", "destination_distinctive_ring", "destination_ringback",
        "destination_accountcode", "destination_caller_id", "destination_conditions",
        "destination_actions", "destination_alternate_app", "destination_alternate_data",
    ] {
        if let Some(v) = s(&payload, &[key]) {
            match key {
                "dialplan_uuid" => am.dialplan_uuid = Set(Some(v)),
                "fax_uuid" => am.fax_uuid = Set(Some(v)),
                "user_uuid" => am.user_uuid = Set(Some(v)),
                "group_uuid" => am.group_uuid = Set(Some(v)),
                "provider_uuid" => am.provider_uuid = Set(Some(v)),
                "destination_trunk_prefix" => am.destination_trunk_prefix = Set(Some(v)),
                "destination_area_code" => am.destination_area_code = Set(Some(v)),
                "destination_condition_field" => am.destination_condition_field = Set(Some(v)),
                "destination_number_regex" => am.destination_number_regex = Set(Some(v)),
                "destination_distinctive_ring" => am.destination_distinctive_ring = Set(Some(v)),
                "destination_ringback" => am.destination_ringback = Set(Some(v)),
                "destination_accountcode" => am.destination_accountcode = Set(Some(v)),
                "destination_caller_id" => am.destination_caller_id = Set(Some(v)),
                "destination_conditions" => am.destination_conditions = Set(Some(v)),
                "destination_actions" => am.destination_actions = Set(Some(v)),
                _ => am.destination_alternate_app = Set(Some(v)),
            }
        }
    }
    // destination_alternate_data shares the prefix loop shape; handle explicitly.
    if let Some(v) = s(&payload, &["destination_alternate_data"]) {
        am.destination_alternate_data = Set(Some(v));
    }
    for (key, slot) in [
        ("destination_record", &mut am.destination_record),
        ("destination_type_voice", &mut am.destination_type_voice),
        ("destination_type_fax", &mut am.destination_type_fax),
        ("destination_type_emergency", &mut am.destination_type_emergency),
        ("destination_type_text", &mut am.destination_type_text),
    ] {
        if let Some(v) = b_opt(&payload, &[key]) {
            *slot = Set(Some(v));
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_destination(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_destination(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "destination_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match destinations::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == Some(domain) || r.domain_id.is_none() => {
            let am: destinations::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn destinations_router() -> Router {
    Router::new()
        .route(
            "/api/v1/destinations",
            axum::routing::post(create_destination).get(list_destinations),
        )
        .route(
            "/api/v1/destinations/",
            axum::routing::post(create_destination).get(list_destinations),
        )
        .route(
            "/api/v1/destinations/{id}",
            axum::routing::get(get_destination)
                .patch(patch_destination)
                .delete(delete_destination),
        )
}

// ---- handlers: follow_me (+ destinations child) ----
fn ser_dest(d: &follow_me_destinations::Model) -> Value {
    serde_json::json!({
        "follow_me_destination_uuid": d.follow_me_destination_uuid,
        "follow_me_destination": d.follow_me_destination,
        "destination": d.follow_me_destination,
        "follow_me_delay": d.follow_me_delay,
        "follow_me_timeout": d.follow_me_timeout,
    })
}

async fn dests_of(db: &sea_orm::DatabaseConnection, fm_id: &str) -> Vec<Value> {
    follow_me_destinations::Entity::find()
        .filter(follow_me_destinations::Column::FollowMeUuid.eq(fm_id.to_owned()))
        .all(db)
        .await
        .unwrap_or_default()
        .iter()
        .map(ser_dest)
        .collect()
}

fn ser_follow_me(r: &follow_me::Model, dests: Vec<Value>) -> Value {
    serde_json::json!({
        "follow_me_uuid": r.follow_me_uuid,
        "domain_id": r.domain_id.to_string(),
        "follow_me_enabled": r.follow_me_enabled,
        "enabled": r.follow_me_enabled,
        "cid_name_prefix": r.cid_name_prefix,
        "cid_number_prefix": r.cid_number_prefix,
        "call_prompt": r.call_prompt,
        "dial_string": r.dial_string,
        "description": r.description,
        "follow_me_ignore_busy": r.follow_me_ignore_busy,
        "destinations": dests,
    })
}

async fn insert_dests(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    fm_id: &str,
    dests: &[Value],
) {
    for (ord, d) in dests.iter().enumerate() {
        let num = if let Some(x) = d.as_str() {
            Some(x.to_owned())
        } else {
            s(d, &["follow_me_destination", "destination", "number"])
        };
        let Some(num) = num else { continue };
        let now = crate::routing_db::now();
        let am = follow_me_destinations::ActiveModel {
            follow_me_destination_uuid: Set(crate::routing_db::new_uuid()),
            domain_id: Set(domain),
            follow_me_uuid: Set(fm_id.to_owned()),
            follow_me_destination: Set(num),
            follow_me_delay: Set(i(d, &["follow_me_delay", "delay"], 0)),
            follow_me_timeout: Set(i(d, &["follow_me_timeout", "timeout"], 30)),
            follow_me_order: Set(ord as i32),
            created_at: Set(now),
            ..Default::default()
        };
        let _ = am.insert(db).await;
    }
}

async fn create_follow_me(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "follow_me_add") {
        return (s, j).into_response();
    }
    // Legacy `extension` key accepted (compat with batch-3c contract) but
    // optional: Python stores no extension column — routing targets come
    // from `destinations`. (The generic UI form has no extension input.)
    let _legacy_extension = s(&payload, &["extension", "follow_me_extension"]);
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = follow_me::ActiveModel {
        follow_me_uuid: Set(id.clone()),
        domain_id: Set(domain),
        follow_me_enabled: Set(b(&payload, &["follow_me_enabled", "enabled"], true)),
        cid_name_prefix: Set(s(&payload, &["cid_name_prefix"])),
        cid_number_prefix: Set(s(&payload, &["cid_number_prefix"])),
        call_prompt: Set(s(&payload, &["call_prompt"])),
        dial_string: Set(s(&payload, &["dial_string"])),
        description: Set(s(&payload, &["description"])),
        follow_me_ignore_busy: Set(b(&payload, &["follow_me_ignore_busy"], false)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let r = match am.insert(db).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
    };
    if let Some(arr) = payload.get("destinations").and_then(|x| x.as_array()) {
        insert_dests(db, domain, &id, arr).await;
    } else if let Some(single) = s(&payload, &["follow_me_destination", "destination"]) {
        insert_dests(db, domain, &id, &[serde_json::Value::String(single)]).await;
    }
    let dests = dests_of(db, &id).await;
    (StatusCode::CREATED, Json(ser_follow_me(&r, dests))).into_response()
}

async fn list_follow_me(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "follow_me_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = follow_me::Entity::find()
        .filter(follow_me::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    // One batched child query (was: N+1 `dests_of` per parent).
    let ids: Vec<String> = rows.iter().map(|r| r.follow_me_uuid.clone()).collect();
    let mut by_fm: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
    if !ids.is_empty() {
        let all_dests = follow_me_destinations::Entity::find()
            .filter(follow_me_destinations::Column::FollowMeUuid.is_in(ids))
            .all(db)
            .await
            .unwrap_or_default();
        for d in &all_dests {
            by_fm
                .entry(d.follow_me_uuid.clone())
                .or_default()
                .push(ser_dest(d));
        }
    }
    let mut items = Vec::new();
    for r in &rows {
        let dests = by_fm.remove(&r.follow_me_uuid).unwrap_or_default();
        items.push(ser_follow_me(r, dests));
    }
    envelope(items).into_response()
}

async fn get_follow_me(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "follow_me_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match follow_me::Entity::find_by_id(id.clone())
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let dests = dests_of(db, &id).await;
            (StatusCode::OK, Json(ser_follow_me(&r, dests))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_follow_me(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "follow_me_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = follow_me::Entity::find_by_id(id.clone())
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: follow_me::ActiveModel = r.into();
    if payload
        .get("follow_me_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.follow_me_enabled = Set(b(&payload, &["follow_me_enabled", "enabled"], true));
    }
    if let Some(v) = s(&payload, &["dial_string"]) {
        am.dial_string = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["description"]) {
        am.description = Set(Some(v));
    }
    am.updated_at = Set(crate::routing_db::now());
    let r = match am.update(db).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("update failed: {e}")})),
            )
                .into_response();
        }
    };
    if let Some(arr) = payload.get("destinations").and_then(|x| x.as_array()) {
        let old = follow_me_destinations::Entity::find()
            .filter(follow_me_destinations::Column::FollowMeUuid.eq(id.clone()))
            .all(db)
            .await
            .unwrap_or_default();
        for o in old {
            let am: follow_me_destinations::ActiveModel = o.into();
            let _ = am.delete(db).await;
        }
        insert_dests(db, domain, &id, arr).await;
    } else if let Some(single) = s(&payload, &["follow_me_destination", "destination"]) {
        let old = follow_me_destinations::Entity::find()
            .filter(follow_me_destinations::Column::FollowMeUuid.eq(id.clone()))
            .all(db)
            .await
            .unwrap_or_default();
        for o in old {
            let am: follow_me_destinations::ActiveModel = o.into();
            let _ = am.delete(db).await;
        }
        insert_dests(db, domain, &id, &[serde_json::Value::String(single)]).await;
    }
    let dests = dests_of(db, &id).await;
    (StatusCode::OK, Json(ser_follow_me(&r, dests))).into_response()
}

async fn delete_follow_me(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "follow_me_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match follow_me::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: follow_me::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn follow_me_router() -> Router {
    Router::new()
        .route(
            "/api/v1/follow_me",
            axum::routing::post(create_follow_me).get(list_follow_me),
        )
        .route(
            "/api/v1/follow_me/",
            axum::routing::post(create_follow_me).get(list_follow_me),
        )
        .route(
            "/api/v1/follow_me/{id}",
            axum::routing::get(get_follow_me)
                .patch(patch_follow_me)
                .delete(delete_follow_me),
        )
}

// ---- handlers: call_forward ----
fn ser_call_forward(r: &call_forwards::Model) -> Value {
    serde_json::json!({
        "call_forward_uuid": r.call_forward_uuid,
        "domain_id": r.domain_id.to_string(),
        "extension_uuid": r.extension_uuid,
        "forward_all_destination": r.forward_all_destination,
        "forward_all_enabled": r.forward_all_enabled,
        "forward_busy_destination": r.forward_busy_destination,
        "forward_busy_enabled": r.forward_busy_enabled,
        "forward_no_answer_destination": r.forward_no_answer_destination,
        "forward_no_answer_enabled": r.forward_no_answer_enabled,
        "forward_user_not_registered_destination": r.forward_user_not_registered_destination,
        "forward_user_not_registered_enabled": r.forward_user_not_registered_enabled,
        "enabled": r.enabled,
        "description": r.description,
    })
}

fn opt_uuid(v: &Value, keys: &[&str]) -> Option<String> {
    let raw = s(v, keys)?;
    raw.parse::<uuid::Uuid>().ok().map(|_| raw)
}

async fn create_call_forward(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_forward_add") {
        return (s, j).into_response();
    }
    if s(
        &payload,
        &["extension", "extension_uuid", "forward_all_destination"],
    )
    .is_none()
    {
        return unprocessable("extension required").into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = call_forwards::ActiveModel {
        call_forward_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        extension_uuid: Set(opt_uuid(&payload, &["extension_uuid", "extension"])),
        forward_all_destination: Set(s(&payload, &["forward_all_destination", "destination"])),
        forward_all_enabled: Set(b(&payload, &["forward_all_enabled"], false)),
        forward_busy_destination: Set(s(&payload, &["forward_busy_destination"])),
        forward_busy_enabled: Set(b(&payload, &["forward_busy_enabled"], false)),
        forward_no_answer_destination: Set(s(&payload, &["forward_no_answer_destination"])),
        forward_no_answer_enabled: Set(b(&payload, &["forward_no_answer_enabled"], false)),
        forward_user_not_registered_destination: Set(s(
            &payload,
            &["forward_user_not_registered_destination"],
        )),
        forward_user_not_registered_enabled: Set(b(
            &payload,
            &["forward_user_not_registered_enabled"],
            false,
        )),
        enabled: Set(b(&payload, &["enabled"], true)),
        description: Set(s(&payload, &["description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_call_forward(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_call_forward(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_forward_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "call_forward_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = call_forwards::Entity::find();
    if !show_all {
        sel = sel.filter(call_forwards::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_call_forward).collect()).into_response()
}

async fn get_call_forward(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_forward_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_forwards::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            (StatusCode::OK, Json(ser_call_forward(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_call_forward(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_forward_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = call_forwards::Entity::find_by_id(id)
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
    if r.domain_id != domain {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mut am: call_forwards::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["forward_all_destination", "destination"]) {
        am.forward_all_destination = Set(Some(v));
    }
    if payload.get("forward_all_enabled").is_some() {
        am.forward_all_enabled = Set(b(&payload, &["forward_all_enabled"], false));
    }
    if let Some(v) = s(&payload, &["description"]) {
        am.description = Set(Some(v));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_call_forward(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_call_forward(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_forward_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_forwards::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: call_forwards::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn call_forward_router() -> Router {
    Router::new()
        .route(
            "/api/v1/call_forward",
            axum::routing::post(create_call_forward).get(list_call_forward),
        )
        .route(
            "/api/v1/call_forward/{id}",
            axum::routing::get(get_call_forward)
                .patch(patch_call_forward)
                .delete(delete_call_forward),
        )
}

// ---- handlers: time_conditions ----
fn valid_tc_extension(ext: &str) -> bool {
    let t = ext.trim();
    (2..=7).contains(&t.len()) && t.chars().all(|c| c.is_ascii_digit())
}

fn ser_time_condition(r: &time_conditions::Model) -> Value {
    serde_json::json!({
        "time_condition_uuid": r.time_condition_uuid,
        "domain_id": r.domain_id.map(|d| d.to_string()),
        "name": r.name,
        "time_condition_name": r.name,
        "extension": r.extension,
        "time_condition_extension": r.extension,
        "dialplan_number": r.extension,
        "context": r.context,
        "enabled": r.enabled,
        "time_condition_enabled": r.enabled,
        "description": r.description,
        "time_condition_description": r.description,
        "dialplan_uuid": r.dialplan_uuid,
        "time_hour": r.time_hour,
        "time_minute": r.time_minute,
        "time_wday": r.time_wday,
        "time_mday": r.time_mday,
        "time_mon": r.time_mon,
        "time_year": r.time_year,
        "condition": r.condition,
        "true_destination": r.true_destination,
        "false_destination": r.false_destination,
        "timezone": r.timezone,
        "details": r.details,
    })
}

async fn create_time_condition(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "time_condition_add") {
        return (s, j).into_response();
    }
    let name = match s(&payload, &["name", "time_condition_name"]) {
        Some(n) if !n.trim().is_empty() => n,
        _ => return unprocessable("name required").into_response(),
    };
    let ext = match s(&payload, &["extension", "time_condition_extension", "dialplan_number", "destination_number"]) {
        Some(e) if valid_tc_extension(&e) => e,
        _ => return unprocessable("extension must be 2-7 digits").into_response(),
    };
    let order = payload.get("dialplan_order").or_else(|| payload.get("order")).and_then(|v| v.as_i64()).map(|v| v as i32);
    let context = s(&payload, &["context", "dialplan_context"]);
    let enabled = b(&payload, &["enabled", "time_condition_enabled", "dialplan_enabled"], true);
    let description = s(&payload, &["description", "time_condition_description", "dialplan_description"]);
    let db = crate::routing_db::conn().await;
    let tenant = tenant_domain(db, &claims).await;
    let posted_domain = payload
        .get("domain_uuid")
        .or_else(|| payload.get("domain_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .map(str::to_owned);
    // PHP `time_condition_domain` parity: holders post any domain
    // (empty = global); everyone else resolves to their own tenant.
    // NOTE: raw access, not s() — s() drops empty strings.
    let domain: Option<uuid::Uuid> = match posted_domain.as_deref() {
        Some(v) if !v.is_empty() && crate::tenant::has_permission(&claims, "time_condition_domain") => {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => Some(id),
                Err(_) => return unprocessable("domain_uuid must be a UUID or empty").into_response(),
            }
        }
        Some(v) if v.is_empty() && crate::tenant::has_permission(&claims, "time_condition_domain") => None,
        _ => Some(tenant),
    };
    let now = crate::routing_db::now();
    let id = crate::routing_db::new_uuid();
    let am = time_conditions::ActiveModel {
        time_condition_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(name),
        extension: Set(ext.clone()),
        context: Set(context.clone()),
        enabled: Set(enabled),
        description: Set(description.clone()),
        condition: Set(s(&payload, &["condition"])),
        true_destination: Set(s(&payload, &["true_destination", "true_dest"])),
        false_destination: Set(s(&payload, &["false_destination", "false_dest"])),
        timezone: Set(s(&payload, &["timezone", "dialplan_time_zone", "time_zone"])),
        details: Set(payload.get("details").cloned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let row = match am.insert(db).await {
        Ok(_) => time_conditions::Entity::find_by_id(id.clone()).one(db).await.unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
    };
    let Some(r) = row else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response();
    };
    // PHP parity: maintain the backing v_dialplans row + details.
    let dp_id = crate::dialplans::sync_tc_dialplan(
        db, domain, &r.name, &ext, order, context, enabled, description,
        r.timezone.clone(), r.true_destination.clone(), r.false_destination.clone(),
        r.details.as_ref(), None,
    )
    .await;
    {
        let mut uam: time_conditions::ActiveModel = r.into();
        uam.dialplan_uuid = Set(Some(dp_id.to_string()));
        let _ = uam.update(db).await;
    }
    crate::fs_xml::invalidate_cache();
    match time_conditions::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) => {
            let mut v = ser_time_condition(&r);
            v["dialplan_order"] = serde_json::json!(order);
            (StatusCode::CREATED, Json(v)).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "insert failed"})),
        )
            .into_response(),
    }
}

async fn list_time_conditions(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "time_condition_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "time_condition_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = time_conditions::Entity::find();
    if !show_all {
        // PHP parity: global time_conditions show for everyone.
        sel = sel.filter(
            Condition::any()
                .add(time_conditions::Column::DomainId.eq(domain))
                .add(time_conditions::Column::DomainId.is_null()),
        );
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    // Backing-dialplan orders (PHP order lives on v_dialplans).
    let mut orders = std::collections::HashMap::new();
    for r in &rows {
        if let Some(dp) = r.dialplan_uuid.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok()) {
            if let Ok(Some(d)) = pbx_db::entities::dialplans::Entity::find_by_id(dp).one(db).await {
                orders.insert(r.time_condition_uuid.clone(), d.dialplan_order);
            }
        }
    }
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            let mut v = ser_time_condition(r);
            v["dialplan_order"] = orders.get(&r.time_condition_uuid).map(|o| serde_json::json!(o)).unwrap_or(serde_json::Value::Null);
            v
        })
        .collect();
    envelope(data).into_response()
}

async fn get_time_condition(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "time_condition_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match time_conditions::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == Some(domain) || r.domain_id.is_none() => {
            let mut v = ser_time_condition(&r);
            if let Some(dp) = r.dialplan_uuid.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok()) {
                if let Ok(Some(d)) = pbx_db::entities::dialplans::Entity::find_by_id(dp).one(db).await {
                    v["dialplan_order"] = serde_json::json!(d.dialplan_order);
                }
            }
            (StatusCode::OK, Json(v)).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_time_condition(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "time_condition_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = time_conditions::Entity::find_by_id(id)
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
    if r.domain_id != Some(domain) && r.domain_id.is_some() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    if let Some(e) = s(&payload, &["extension", "time_condition_extension"]) {
        if !valid_tc_extension(&e) {
            return unprocessable("extension must be 2-7 digits").into_response();
        }
    }
    let mut am: time_conditions::ActiveModel = r.into();
    // PHP `time_condition_domain` parity: perm holders may re-home the row (UUID string)
    // or globalize it (empty string); otherwise the domain is untouched.
    if payload.get("domain_uuid").or_else(|| payload.get("domain_id")).is_some()
        && crate::tenant::sees_domain_nulls(&claims, "time_condition_domain")
    {
        let raw = payload
            .get("domain_uuid")
            .or_else(|| payload.get("domain_id"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if raw.is_empty() {
            am.domain_id = Set(None);
        } else {
            match raw.parse::<uuid::Uuid>() {
                Ok(id) => am.domain_id = Set(Some(id)),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID or empty"})),
                    )
                        .into_response();
                }
            }
        }
    }
    if let Some(v) = s(&payload, &["name", "time_condition_name", "dialplan_name"]) {
        am.name = Set(v);
    }
    if let Some(v) = s(&payload, &["extension", "time_condition_extension", "dialplan_number", "destination_number"]) {
        am.extension = Set(v);
    }
    if payload
        .get("enabled")
        .or_else(|| payload.get("time_condition_enabled"))
        .or_else(|| payload.get("dialplan_enabled"))
        .is_some()
    {
        am.enabled = Set(b(&payload, &["enabled", "time_condition_enabled", "dialplan_enabled"], true));
    }
    if let Some(v) = s(&payload, &["description", "time_condition_description", "dialplan_description"]) {
        am.description = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["context", "dialplan_context"]) {
        am.context = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["condition"]) {
        am.condition = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["true_destination", "true_dest"]) {
        am.true_destination = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["false_destination", "false_dest"]) {
        am.false_destination = Set(Some(v));
    }
    if let Some(v) = s(&payload, &["timezone", "dialplan_time_zone", "time_zone"]) {
        am.timezone = Set(Some(v));
    }
    if payload.get("details").is_some() {
        am.details = Set(payload.get("details").cloned());
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => {
            let order = payload.get("dialplan_order").or_else(|| payload.get("order")).and_then(|v| v.as_i64()).map(|v| v as i32);
            let existing = r.dialplan_uuid.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok());
            let dp_id = crate::dialplans::sync_tc_dialplan(
                db, r.domain_id, &r.name, &r.extension, order, r.context.clone(), r.enabled,
                r.description.clone(), r.timezone.clone(), r.true_destination.clone(),
                r.false_destination.clone(), r.details.as_ref(), existing,
            )
            .await;
            if r.dialplan_uuid.as_deref() != Some(&dp_id.to_string()) {
                let mut uam: time_conditions::ActiveModel = r.clone().into();
                uam.dialplan_uuid = Set(Some(dp_id.to_string()));
                let _ = uam.update(db).await;
            }
            crate::fs_xml::invalidate_cache();
            match time_conditions::Entity::find_by_id(r.time_condition_uuid.clone()).one(db).await.unwrap_or(None) {
                Some(row) => {
                    let mut v = ser_time_condition(&row);
                    v["dialplan_order"] = serde_json::json!(order);
                    (StatusCode::OK, Json(v)).into_response()
                }
                None => (StatusCode::OK, Json(ser_time_condition(&r))).into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_time_condition(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "time_condition_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match time_conditions::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == Some(domain) || r.domain_id.is_none() => {
            if let Some(dp) = r.dialplan_uuid.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok()) {
                crate::dialplans::delete_tc_dialplan(db, dp).await;
            }
            let am: time_conditions::ActiveModel = r.into();
            let _ = am.delete(db).await;
            crate::fs_xml::invalidate_cache();
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn time_conditions_router() -> Router {
    Router::new()
        .route(
            "/api/v1/time_conditions",
            axum::routing::post(create_time_condition).get(list_time_conditions),
        )
        .route(
            "/api/v1/time_conditions/{id}",
            axum::routing::get(get_time_condition)
                .patch(patch_time_condition)
                .delete(delete_time_condition),
        )
}

// ---- registry ----
pub fn spillover_a_router() -> Router {
    call_block_router()
        .merge(broadcast_router())
        .merge(bridges_router())
        .merge(destinations_router())
        .merge(follow_me_router())
        .merge(call_forward_router())
        .merge(time_conditions_router())
}
