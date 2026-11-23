//! Gap-3 admin routers: domain_limits, extension_settings,
//! sofia_global_settings, call_recordings, call_flows + webrtc hyphen alias.
//! Column parity with `app/*/models.py`.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde_json::Value;

type Claims = pbx_core::auth::Claims;
use crate::remaining_a::{b, envelope, s, tenant_domain, unprocessable};
use pbx_db::entities::{
    call_flows, call_recordings, extension_settings, sofia_global_settings,
};

// ---- domain_limits: counts view (parity app/domain_limits/router.py) ----
async fn get_domain_limits(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    use sea_orm::PaginatorTrait;
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    // explicit per-table counts (SeaORM types differ, so count each directly)
    let ext_cnt = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let dial_cnt = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let gw_cnt = pbx_db::entities::gateways::Entity::find()
        .filter(pbx_db::entities::gateways::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let dev_cnt = pbx_db::entities::devices::Entity::find()
        .filter(pbx_db::entities::devices::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let vm_cnt = pbx_db::entities::voicemails::Entity::find()
        .filter(pbx_db::entities::voicemails::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    // limits from domain_settings category==limit, else hardcoded defaults (parity)
    let mut limits: std::collections::HashMap<String, i64> = Default::default();
    let rows = pbx_db::entities::domain_settings::Entity::find()
        .filter(pbx_db::entities::domain_settings::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    for r in rows {
        let sub = r.subcategory.to_lowercase();
        if r.category == "limit" || sub.contains("limit") {
            if let Ok(v) = r.value.parse::<i64>() {
                // key field holds limit name; fallback to subcategory
                let k = if r.key.is_empty() { r.subcategory.clone() } else { r.key.clone() };
                limits.insert(k, v);
            }
        }
    }
    let lim = |k: &str, d: u64| -> u64 {
        limits.get(k).and_then(|v| u64::try_from(*v).ok()).unwrap_or(d)
    };
    let mk = |used: u64, allowed: u64| {
        serde_json::json!({
            "used": used,
            "allowed": allowed,
            "remaining": allowed.saturating_sub(used),
        })
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "domain_id": domain.to_string(),
            "limits": {
                "extensions": mk(ext_cnt, lim("extensions", 1000)),
                "dialplans": mk(dial_cnt, lim("dialplans", 1000)),
                "gateways": mk(gw_cnt, lim("gateways", 100)),
                "devices": mk(dev_cnt, lim("devices", 1000)),
                "voicemails": mk(vm_cnt, lim("voicemails", 1000)),
            },
            "_links": {"self": {"href": "/api/v1/domain_limits"}},
        })),
    )
        .into_response()
}

// ---- extension_settings CRUD ----
fn ser_ext_setting(r: &extension_settings::Model) -> Value {
    serde_json::json!({
        "extension_setting_uuid": r.extension_setting_uuid,
        "domain_id": r.domain_id.to_string(),
        "extension_uuid": r.extension_uuid.to_string(),
        "extension_setting_type": r.extension_setting_type,
        "extension_setting_name": r.extension_setting_name,
        "extension_setting_value": r.extension_setting_value,
        "extension_setting_enabled": r.extension_setting_enabled,
        "enabled": r.extension_setting_enabled,
        "extension_setting_description": r.extension_setting_description,
    })
}

async fn create_ext_setting(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let (Some(name), Some(value)) = (
        s(&payload, &["extension_setting_name", "name"]),
        s(&payload, &["extension_setting_value", "value"]),
    ) else {
        return unprocessable("extension_setting_name and value required").into_response();
    };
    let ext_uuid = s(&payload, &["extension_uuid", "extension"])
        .and_then(|v| v.parse::<uuid::Uuid>().ok());
    let Some(ext_uuid) = ext_uuid else {
        return unprocessable("extension_uuid required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = extension_settings::ActiveModel {
        extension_setting_uuid: Set(uuid::Uuid::now_v7()),
        domain_id: Set(domain),
        extension_uuid: Set(ext_uuid),
        extension_setting_type: Set(s(&payload, &["extension_setting_type", "type"])),
        extension_setting_name: Set(name),
        extension_setting_value: Set(value),
        extension_setting_enabled: Set(b(&payload, &["extension_setting_enabled", "enabled"], true)),
        extension_setting_description: Set(s(&payload, &["extension_setting_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_ext_setting(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_ext_settings(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["extension_setting_view", "extension_view"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "extension_setting_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = extension_settings::Entity::find();
    if !show_all {
        sel = sel.filter(extension_settings::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_ext_setting).collect()).into_response()
}

async fn get_ext_setting(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match extension_settings::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_ext_setting(&r))).into_response(),
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

async fn patch_ext_setting(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = extension_settings::Entity::find_by_id(id).one(db).await.unwrap_or(None) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response();
    };
    if r.domain_id != domain {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response();
    }
    let mut am: extension_settings::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["extension_setting_name", "name"]) {
        am.extension_setting_name = Set(v);
    }
    if let Some(v) = s(&payload, &["extension_setting_value", "value"]) {
        am.extension_setting_value = Set(v);
    }
    if payload.get("extension_setting_enabled").or_else(|| payload.get("enabled")).is_some() {
        am.extension_setting_enabled = Set(b(&payload, &["extension_setting_enabled", "enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_ext_setting(&r))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("update failed: {e}")}))).into_response(),
    }
}

async fn delete_ext_setting(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match extension_settings::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == domain => {
            let am: extension_settings::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

// ---- sofia_global_settings CRUD (global, no domain) ----
fn ser_sofia(r: &sofia_global_settings::Model) -> Value {
    serde_json::json!({
        "sofia_global_setting_uuid": r.sofia_global_setting_uuid,
        "global_setting_name": r.global_setting_name,
        "name": r.global_setting_name,
        "global_setting_value": r.global_setting_value,
        "value": r.global_setting_value,
        "global_setting_enabled": r.global_setting_enabled,
        "enabled": r.global_setting_enabled,
        "global_setting_description": r.global_setting_description,
    })
}

async fn create_sofia(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sofia_add") {
        return (s, j).into_response();
    }
    let (Some(name), Some(value)) = (
        s(&payload, &["global_setting_name", "name"]),
        s(&payload, &["global_setting_value", "value"]),
    ) else {
        return unprocessable("name and value required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let now = crate::routing_db::now();
    let am = sofia_global_settings::ActiveModel {
        sofia_global_setting_uuid: Set(uuid::Uuid::now_v7()),
        global_setting_name: Set(name),
        global_setting_value: Set(value),
        global_setting_enabled: Set(b(&payload, &["global_setting_enabled", "enabled"], true)),
        global_setting_description: Set(s(&payload, &["global_setting_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_sofia(&r))).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") || msg.contains("unique") || msg.contains("duplicate") {
                return (StatusCode::CONFLICT, Json(serde_json::json!({"detail": "duplicate"}))).into_response();
            }
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response()
        }
    }
}

async fn list_sofia(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sofia_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = sofia_global_settings::Entity::find().all(db).await.unwrap_or_default();
    envelope(rows.iter().map(ser_sofia).collect()).into_response()
}

async fn get_sofia(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sofia_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match sofia_global_settings::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) => (StatusCode::OK, Json(ser_sofia(&r))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

async fn patch_sofia(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sofia_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(r) = sofia_global_settings::Entity::find_by_id(id).one(db).await.unwrap_or(None) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response();
    };
    let mut am: sofia_global_settings::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["global_setting_name", "name"]) {
        am.global_setting_name = Set(v);
    }
    if let Some(v) = s(&payload, &["global_setting_value", "value"]) {
        am.global_setting_value = Set(v);
    }
    if payload.get("global_setting_enabled").or_else(|| payload.get("enabled")).is_some() {
        am.global_setting_enabled = Set(b(&payload, &["global_setting_enabled", "enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_sofia(&r))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("update failed: {e}")}))).into_response(),
    }
}

async fn delete_sofia(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "sofia_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match sofia_global_settings::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) => {
            let am: sofia_global_settings::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

// ---- call_recordings CRUD ----
fn ser_call_rec(r: &call_recordings::Model) -> Value {
    serde_json::json!({
        "call_recording_uuid": r.call_recording_uuid,
        "domain_id": r.domain_id.to_string(),
        "cdr_uuid": r.cdr_uuid,
        "recording_path": r.recording_path,
        "recording_name": r.recording_name,
    })
}

async fn create_call_rec(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_add", "call_recording_add"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let am = call_recordings::ActiveModel {
        call_recording_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        cdr_uuid: Set(s(&payload, &["cdr_uuid"])),
        recording_path: Set(s(&payload, &["recording_path", "path"])),
        recording_name: Set(s(&payload, &["recording_name", "name"])),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_call_rec(&r))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_call_rec(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_view", "call_recording_view"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "call_recording_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = call_recordings::Entity::find();
    if !show_all {
        sel = sel.filter(call_recordings::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_call_rec).collect()).into_response()
}

async fn get_call_rec(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_view", "call_recording_view"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_recordings::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == domain => (StatusCode::OK, Json(ser_call_rec(&r))).into_response(),
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

async fn delete_call_rec(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["recording_add", "call_recording_add"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_recordings::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == domain => {
            let am: call_recordings::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

// ---- call_flows CRUD ----
fn ser_call_flow(r: &call_flows::Model) -> Value {
    serde_json::json!({
        "call_flow_uuid": r.call_flow_uuid,
        "domain_id": r.domain_id.map(|d| d.to_string()),
        "dialplan_uuid": r.dialplan_uuid,
        "call_flow_name": r.call_flow_name,
        "name": r.call_flow_name,
        "call_flow_extension": r.call_flow_extension,
        "extension": r.call_flow_extension,
        "call_flow_feature_code": r.call_flow_feature_code,
        "call_flow_context": r.call_flow_context,
        "call_flow_status": r.call_flow_status,
        "call_flow_pin_number": r.call_flow_pin_number,
        "call_flow_label": r.call_flow_label,
        "call_flow_enabled": r.call_flow_enabled,
        "enabled": r.call_flow_enabled,
        "call_flow_description": r.call_flow_description,
        "description": r.call_flow_description,
        "call_flow_sound": r.call_flow_sound,
        "call_flow_app": r.call_flow_app,
        "call_flow_data": r.call_flow_data,
    })
}

async fn create_call_flow(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_flow_add") {
        return (s, j).into_response();
    }
    let (Some(name), Some(ext)) = (
        s(&payload, &["call_flow_name", "name"]),
        s(&payload, &["call_flow_extension", "extension"]),
    ) else {
        return unprocessable("name and extension required").into_response();
    };
    // parity validator: 2-7 digits optionally prefixed *#+ (app/call_flows/models.py)
    let ok = {
        let t = ext.trim();
        let digits = t.trim_start_matches(|c| c == '*' || c == '#' || c == '+');
        (2..=7).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit())
    };
    if !ok {
        return unprocessable("call_flow_extension must be 2-7 digits").into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = call_flows::ActiveModel {
        call_flow_uuid: Set(uuid::Uuid::now_v7()),
        domain_id: Set(Some(domain)),
        dialplan_uuid: Set(s(&payload, &["dialplan_uuid"]).and_then(|v| v.parse().ok())),
        call_flow_name: Set(name),
        call_flow_extension: Set(ext),
        call_flow_feature_code: Set(s(&payload, &["call_flow_feature_code"])),
        call_flow_context: Set(s(&payload, &["call_flow_context"]).unwrap_or_else(|| "public".to_owned())),
        call_flow_status: Set(s(&payload, &["call_flow_status"])),
        call_flow_pin_number: Set(s(&payload, &["call_flow_pin_number"])),
        call_flow_label: Set(s(&payload, &["call_flow_label"])),
        call_flow_enabled: Set(b(&payload, &["call_flow_enabled", "enabled"], true)),
        call_flow_description: Set(s(&payload, &["call_flow_description", "description"])),
        call_flow_sound: Set(s(&payload, &["call_flow_sound"])),
        call_flow_app: Set(s(&payload, &["call_flow_app"])),
        call_flow_data: Set(s(&payload, &["call_flow_data"])),
        call_flow_alternate_label: Set(s(&payload, &["call_flow_alternate_label"])),
        call_flow_alternate_destination: Set(s(&payload, &["call_flow_alternate_destination"])),
        call_flow_alternate_sound: Set(s(&payload, &["call_flow_alternate_sound"])),
        call_flow_alternate_app: Set(s(&payload, &["call_flow_alternate_app"])),
        call_flow_alternate_data: Set(s(&payload, &["call_flow_alternate_data"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_call_flow(&r))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("insert failed: {e}")}))).into_response(),
    }
}

async fn list_call_flows(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_flow_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "call_flow_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = call_flows::Entity::find();
    if !show_all {
        sel = sel.filter(
            Condition::any()
                .add(call_flows::Column::DomainId.eq(domain))
                .add(call_flows::Column::DomainId.is_null()),
        );
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_call_flow).collect()).into_response()
}

async fn get_call_flow(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_flow_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_flows::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == Some(domain) || r.domain_id.is_none() => (StatusCode::OK, Json(ser_call_flow(&r))).into_response(),
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

async fn patch_call_flow(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_flow_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = call_flows::Entity::find_by_id(id).one(db).await.unwrap_or(None) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response();
    };
    if r.domain_id != Some(domain) && r.domain_id.is_some() {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response();
    }
    let mut am: call_flows::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["call_flow_name", "name"]) {
        am.call_flow_name = Set(v);
    }
    if payload.get("call_flow_enabled").or_else(|| payload.get("enabled")).is_some() {
        am.call_flow_enabled = Set(b(&payload, &["call_flow_enabled", "enabled"], true));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_call_flow(&r))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"detail": format!("update failed: {e}")}))).into_response(),
    }
}

async fn delete_call_flow(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "call_flow_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match call_flows::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == Some(domain) || r.domain_id.is_none() => {
            let am: call_flows::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!({"detail": "not found"}))).into_response(),
    }
}

// ---- webrtc hyphen alias: same shape as underscore handlers ----
async fn list_webrtc_hyphen(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = pbx_db::entities::webrtc_settings::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None);
    let item = match row {
        Some(r) => serde_json::json!({
            "domain_id": r.domain_id.to_string(),
            "enabled": r.enabled,
            "auth_mode": r.auth_mode,
            "wss_url": r.wss_url,
            "sip_domain": r.sip_domain,
        }),
        None => serde_json::json!({
            "domain_id": domain.to_string(),
            "enabled": false,
            "auth_mode": "extension",
            "wss_url": serde_json::Value::Null,
            "sip_domain": serde_json::Value::Null,
        }),
    };
    envelope(vec![item]).into_response()
}

pub fn admin_router() -> Router {
    Router::new()
        .route("/api/v1/domain_limits", axum::routing::get(get_domain_limits))
        .route("/api/v1/domain_limits/", axum::routing::get(get_domain_limits))
        .route("/api/v1/extension_settings", axum::routing::post(create_ext_setting).get(list_ext_settings))
        .route("/api/v1/extension_settings/", axum::routing::post(create_ext_setting).get(list_ext_settings))
        .route("/api/v1/extension_settings/{id}", axum::routing::get(get_ext_setting).patch(patch_ext_setting).delete(delete_ext_setting))
        .route("/api/v1/sofia_global_settings", axum::routing::post(create_sofia).get(list_sofia))
        .route("/api/v1/sofia_global_settings/", axum::routing::post(create_sofia).get(list_sofia))
        .route("/api/v1/sofia_global_settings/{id}", axum::routing::get(get_sofia).patch(patch_sofia).delete(delete_sofia))
        .route("/api/v1/call_recordings", axum::routing::post(create_call_rec).get(list_call_rec))
        .route("/api/v1/call_recordings/", axum::routing::post(create_call_rec).get(list_call_rec))
        .route("/api/v1/call_recordings/{id}", axum::routing::get(get_call_rec).delete(delete_call_rec))
        .route("/api/v1/call_flows", axum::routing::post(create_call_flow).get(list_call_flows))
        .route("/api/v1/call_flows/", axum::routing::post(create_call_flow).get(list_call_flows))
        .route("/api/v1/call_flows/{id}", axum::routing::get(get_call_flow).patch(patch_call_flow).delete(delete_call_flow))
        .route("/api/v1/webrtc-settings", axum::routing::get(list_webrtc_hyphen))
        .route("/api/v1/webrtc-settings/", axum::routing::get(list_webrtc_hyphen))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
