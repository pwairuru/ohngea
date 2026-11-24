//! Settings admin — mirrors `app/admin_settings/router.py::_crud` for
//! `default_settings` (global, no domain_id) + `domain_settings` read parity.
//!
//! Order: models → schemas → handlers.
//!
//! models: `pbx_db::entities::{default_settings}`.
//! Global table: no tenant filter (parity with Python `domain_scoped=False`).
//! Every handler gates on `default_setting_{view,add,edit,delete}`
//! via `crate::tenant::deny_if_missing` (Python `require_permission` parity).
//!
//! Auth hardening (MFA/OTP, password-reset flow, LDAP) is intentionally
//! deferred — see TODO in `auth.rs`. This module covers the runtime settings
//! editor + per-user settings (user_settings GET/PUT live in `users.rs`).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{default_settings, domain_settings};

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct DefaultSettingCreate {
    #[serde(default)]
    default_setting_category: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    default_setting_subcategory: Option<String>,
    #[serde(default)]
    subcategory: Option<String>,
    #[serde(default)]
    default_setting_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    default_setting_value: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    default_setting_order: Option<i32>,
    #[serde(default)]
    order: Option<i32>,
    #[serde(default)]
    default_setting_enabled: Option<bool>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    default_setting_description: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SettingsListQuery {
    #[serde(default)]
    show: Option<String>,
    #[serde(default = "d_page")]
    page: u64,
    #[serde(default = "d_per_page")]
    per_page: u64,
    #[serde(default)]
    q: Option<String>,
}

fn d_page() -> u64 {
    1
}
fn d_per_page() -> u64 {
    20
}

fn pick(primary: &Option<String>, alias: &Option<String>) -> Option<String> {
    primary
        .as_deref()
        .or(alias.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

// ---- handlers ----
fn ser(r: &default_settings::Model) -> serde_json::Value {
    serde_json::json!({
        "default_setting_uuid": r.default_setting_uuid,
        "default_setting_category": r.default_setting_category,
        "default_setting_subcategory": r.default_setting_subcategory,
        "default_setting_name": r.default_setting_name,
        "default_setting_value": r.default_setting_value,
        "default_setting_order": r.default_setting_order,
        "order": r.default_setting_order,
        "default_setting_enabled": r.default_setting_enabled,
        "enabled": r.default_setting_enabled,
        "default_setting_description": r.default_setting_description,
    })
}

async fn create_default_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<DefaultSettingCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "default_setting_add") {
        return (s, j).into_response();
    }
    let Some(category) = pick(&payload.default_setting_category, &payload.category) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "default_setting_category required"})),
        )
            .into_response();
    };
    let Some(name) = pick(&payload.default_setting_name, &payload.name) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "default_setting_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let now = crate::routing_db::now();
    let enabled = payload
        .default_setting_enabled
        .or(payload.enabled)
        .unwrap_or(true);
    let am = default_settings::ActiveModel {
        default_setting_uuid: Set(uuid::Uuid::now_v7()),
        default_setting_category: Set(category),
        default_setting_subcategory: Set(pick(
            &payload.default_setting_subcategory,
            &payload.subcategory,
        )
        .unwrap_or_default()),
        default_setting_name: Set(name),
        default_setting_value: Set(pick(&payload.default_setting_value, &payload.value)),
        default_setting_order: Set(payload.default_setting_order.or(payload.order)),
        default_setting_enabled: Set(enabled),
        default_setting_description: Set(pick(
            &payload.default_setting_description,
            &payload.description,
        )),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_default_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<SettingsListQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "default_setting_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = crate::PageQuery {
        show: None,
        page: pq.page,
        per_page: pq.per_page,
    }
    .normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let mut sel = default_settings::Entity::find();
    if let Some(q) = pq.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pat = format!("%{q}%");
        sel = sel.filter(
            sea_orm::Condition::any()
                .add(default_settings::Column::DefaultSettingCategory.like(pat.clone()))
                .add(default_settings::Column::DefaultSettingName.like(pat.clone()))
                .add(default_settings::Column::DefaultSettingValue.like(pat)),
        );
    }
    let paginator = sel.paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(ser).collect();
    (
        crate::paged_headers("/api/v1/default_settings", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_default_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "default_setting_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match default_settings::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_default_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "default_setting_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(r) = default_settings::Entity::find_by_id(id)
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
    let mut am: default_settings::ActiveModel = r.into();
    if let Some(v) = payload
        .get("default_setting_category")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        am.default_setting_category = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("default_setting_subcategory")
        .and_then(|v| v.as_str())
    {
        am.default_setting_subcategory = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("default_setting_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        am.default_setting_name = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("default_setting_value")
        .or_else(|| payload.get("value"))
    {
        if v.is_null() {
            am.default_setting_value = Set(None);
        } else if let Some(s) = v.as_str() {
            am.default_setting_value = Set(Some(s.to_owned()));
        }
    }
    if let Some(v) = payload
        .get("default_setting_order")
        .or_else(|| payload.get("order"))
        .and_then(|v| v.as_i64())
    {
        am.default_setting_order = Set(Some(v as i32));
    }
    if let Some(b) = payload
        .get("default_setting_enabled")
        .or_else(|| payload.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        am.default_setting_enabled = Set(b);
    }
    if let Some(v) = payload
        .get("default_setting_description")
        .or_else(|| payload.get("description"))
    {
        if v.is_null() {
            am.default_setting_description = Set(None);
        } else if let Some(s) = v.as_str() {
            am.default_setting_description = Set(Some(s.to_owned()));
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_default_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "default_setting_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(r) = default_settings::Entity::find_by_id(id)
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
    let am: default_settings::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

// ---- domain_settings (tenant-scoped, parity with
// fusionpbx/core/domain_settings v_domain_settings CRUD) ----
#[derive(Debug, serde::Deserialize)]
struct DomainSettingCreate {
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    domain_setting_category: Option<String>,
    #[serde(default)]
    subcategory: Option<String>,
    #[serde(default)]
    domain_setting_subcategory: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    domain_setting_name: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    domain_setting_value: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    domain_setting_enabled: Option<bool>,
}

fn dser(r: &domain_settings::Model) -> serde_json::Value {
    serde_json::json!({
        "id": r.id,
        "domain_setting_uuid": r.id,
        "domain_id": r.domain_id,
        "domain_uuid": r.domain_id,
        "category": r.category,
        "domain_setting_category": r.category,
        "subcategory": r.subcategory,
        "domain_setting_subcategory": r.subcategory,
        "key": r.key,
        "name": r.key,
        "domain_setting_name": r.key,
        "value": r.value,
        "domain_setting_value": r.value,
        "enabled": r.enabled,
        "domain_setting_enabled": r.enabled,
    })
}

async fn create_domain_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<DomainSettingCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_setting_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let category = payload
        .category
        .or(payload.domain_setting_category)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let subcategory = payload
        .subcategory
        .or(payload.domain_setting_subcategory)
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    let key = payload
        .name
        .or(payload.key)
        .or(payload.domain_setting_name)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let (Some(category), Some(key)) = (category, key) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "category and name/key required"})),
        )
            .into_response();
    };
    let value = payload
        .value
        .or(payload.domain_setting_value)
        .unwrap_or_default();
    let am = domain_settings::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        domain_id: Set(domain_id),
        category: Set(category),
        subcategory: Set(subcategory),
        key: Set(key),
        value: Set(value),
        enabled: Set(payload
            .enabled
            .or(payload.domain_setting_enabled)
            .unwrap_or(true)),
        created_at: Set(crate::routing_db::now()),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(dser(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_domain_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<SettingsListQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_setting_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = crate::PageQuery {
        show: None,
        page: pq.page,
        per_page: pq.per_page,
    }
    .normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "domain_setting_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/domain_settings", page, per_page, 0),
                Json(crate::paged_body(
                    Vec::<serde_json::Value>::new(),
                    0,
                    page,
                    per_page,
                )),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let mut sel = domain_settings::Entity::find();
    if !show_all {
        sel = sel.filter(domain_settings::Column::DomainId.eq(domain_id));
    }
    if let Some(q) = pq.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pat = format!("%{q}%");
        sel = sel.filter(
            sea_orm::Condition::any()
                .add(domain_settings::Column::Category.like(pat.clone()))
                .add(domain_settings::Column::Key.like(pat.clone()))
                .add(domain_settings::Column::Value.like(pat)),
        );
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(dser).collect();
    (
        crate::paged_headers("/api/v1/domain_settings", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_domain_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_setting_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    match domain_settings::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .filter(|r| r.domain_id == domain_id)
    {
        Some(r) => (StatusCode::OK, Json(dser(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_domain_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_setting_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(row) = domain_settings::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .filter(|r| r.domain_id == domain_id)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: domain_settings::ActiveModel = row.into();
    if let Some(v) = payload
        .get("value")
        .or_else(|| payload.get("domain_setting_value"))
        .and_then(|v| v.as_str())
    {
        am.value = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("enabled")
        .or_else(|| payload.get("domain_setting_enabled"))
        .and_then(|v| v.as_bool())
    {
        am.enabled = Set(v);
    }
    if let Some(v) = payload
        .get("category")
        .or_else(|| payload.get("domain_setting_category"))
        .and_then(|v| v.as_str())
    {
        am.category = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("subcategory")
        .or_else(|| payload.get("domain_setting_subcategory"))
        .and_then(|v| v.as_str())
    {
        am.subcategory = Set(v.to_owned());
    }
    if let Some(v) = payload
        .get("name")
        .or_else(|| payload.get("key"))
        .or_else(|| payload.get("domain_setting_name"))
        .and_then(|v| v.as_str())
    {
        am.key = Set(v.to_owned());
    }
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(dser(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_domain_setting(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_setting_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(row) = domain_settings::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .filter(|r| r.domain_id == domain_id)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: domain_settings::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn settings_router() -> Router {
    Router::new()
        .route(
            "/api/v1/default_settings",
            post(create_default_setting).get(list_default_settings),
        )
        .route(
            "/api/v1/default_settings/",
            post(create_default_setting).get(list_default_settings),
        )
        .route(
            "/api/v1/default_settings/{id}",
            axum::routing::get(get_default_setting)
                .patch(patch_default_setting)
                .delete(delete_default_setting),
        )
        .route(
            "/api/v1/domain_settings",
            post(create_domain_setting).get(list_domain_settings),
        )
        .route(
            "/api/v1/domain_settings/",
            post(create_domain_setting).get(list_domain_settings),
        )
        .route(
            "/api/v1/domain_settings/{id}",
            axum::routing::get(get_domain_setting)
                .patch(patch_domain_setting)
                .delete(delete_domain_setting),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
