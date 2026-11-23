//! Services registry — mirrors `fusionpbx/core/services` (`v_services`
//! CRUD pages, perms `service_view/add/edit/delete/all`).
//!
//! Order: models → schemas → handlers.
//!
//! models: `pbx_db::entities::services` (global table, no domain_id —
//! parity with PHP `table.parent = ''`). No tenant filter; every handler
//! gates on its `service_*` permission via `deny_if_missing`.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde::Deserialize;

// ---- models ----
use pbx_db::entities::services;

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct ServiceCreate {
    #[serde(default)]
    service_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    service_category: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    service_file: Option<String>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    service_enabled: Option<bool>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    service_description: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ServicesListQuery {
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

fn ser(r: &services::Model) -> serde_json::Value {
    serde_json::json!({
        "service_uuid": r.service_uuid,
        "service_name": r.service_name,
        "service_category": r.service_category,
        "service_file": r.service_file,
        "service_enabled": r.service_enabled,
        "enabled": r.service_enabled,
        "service_description": r.service_description,
    })
}

// ---- handlers ----
async fn create_service(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<ServiceCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "service_add") {
        return (s, j).into_response();
    }
    let Some(name) = pick(&payload.service_name, &payload.name) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "service_name required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let now = crate::routing_db::now();
    let am = services::ActiveModel {
        service_uuid: Set(uuid::Uuid::now_v7()),
        service_name: Set(Some(name)),
        service_category: Set(pick(&payload.service_category, &payload.category)),
        service_file: Set(pick(&payload.service_file, &payload.file)),
        service_enabled: Set(payload.service_enabled.or(payload.enabled).unwrap_or(true)),
        service_description: Set(pick(&payload.service_description, &payload.description)),
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

async fn list_services(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<ServicesListQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "service_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = crate::PageQuery { show: None,
        page: pq.page,
        per_page: pq.per_page,
    }
    .normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let mut sel = services::Entity::find();
    if let Some(q) = pq.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pat = format!("%{q}%");
        sel = sel.filter(
            sea_orm::Condition::any()
                .add(services::Column::ServiceName.like(pat.clone()))
                .add(services::Column::ServiceCategory.like(pat.clone()))
                .add(services::Column::ServiceFile.like(pat)),
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
        crate::paged_headers("/api/v1/services", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_service(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "service_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match services::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) => (StatusCode::OK, Json(ser(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Service not found"})),
        )
            .into_response(),
    }
}

async fn patch_service(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "service_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = services::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Service not found"})),
        )
            .into_response();
    };
    let mut am: services::ActiveModel = row.into();
    if let Some(v) = payload
        .get("service_name")
        .or_else(|| payload.get("name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        am.service_name = Set(Some(v.to_owned()));
    }
    if payload.get("service_category").or_else(|| payload.get("category")).is_some() {
        am.service_category = Set(payload
            .get("service_category")
            .or_else(|| payload.get("category"))
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if payload.get("service_file").or_else(|| payload.get("file")).is_some() {
        am.service_file = Set(payload
            .get("service_file")
            .or_else(|| payload.get("file"))
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }
    if let Some(v) = payload
        .get("service_enabled")
        .or_else(|| payload.get("enabled"))
        .and_then(|v| v.as_bool())
    {
        am.service_enabled = Set(v);
    }
    if payload.get("service_description").or_else(|| payload.get("description")).is_some() {
        am.service_description = Set(payload
            .get("service_description")
            .or_else(|| payload.get("description"))
            .and_then(|v| v.as_str())
            .map(str::to_owned));
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

async fn delete_service(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "service_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let Some(row) = services::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Service not found"})),
        )
            .into_response();
    };
    let am: services::ActiveModel = row.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn services_router() -> Router {
    Router::new()
        .route("/api/v1/services", post(create_service).get(list_services))
        .route("/api/v1/services/", post(create_service).get(list_services))
        .route(
            "/api/v1/services/{id}",
            axum::routing::get(get_service)
                .patch(patch_service)
                .delete(delete_service),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
