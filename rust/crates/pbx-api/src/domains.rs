//! Domains router — mirrors `app/domains/router.py` bootstrap/status/list.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{domains}`.
//! bootstrap/status is PUBLIC (no tenant guard); the rest is tenant-guarded.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::domains;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct DomainCreate {
    domain_name: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    description: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, serde::Deserialize)]
struct Bootstrap {
    domain_name: String,
    #[serde(default)]
    description: Option<String>,
}

// ---- handlers ----
async fn bootstrap_status() -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let one = domains::Entity::find().one(db).await.unwrap_or(None);
    let has = one.is_some();
    Json(serde_json::json!({"needs_setup": !has, "has_domains": has}))
}

async fn bootstrap(Json(payload): Json<Bootstrap>) -> impl IntoResponse {
    let name = payload.domain_name.trim().to_owned();
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "domain_name required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    if domains::Entity::find()
        .one(db)
        .await
        .unwrap_or(None)
        .is_some()
    {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": "already bootstrapped"})),
        );
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = domains::ActiveModel {
        domain_id: Set(id),
        domain_name: Set(name.clone()),
        enabled: Set(true),
        description: Set(payload.description.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "domain_id": id,
            "domain_name": name,
            "enabled": true,
        })),
    )
}

async fn create_domain(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<DomainCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_add") {
        return (s, j).into_response();
    }
    let name = payload.domain_name.trim().to_owned();
    if name.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "domain_name required"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let exists = domains::Entity::find()
        .filter(domains::Column::DomainName.eq(name.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": "domain exists"})),
        )
            .into_response();
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = domains::ActiveModel {
        domain_id: Set(id),
        domain_name: Set(name.clone()),
        enabled: Set(payload.enabled),
        description: Set(payload.description.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "domain_id": id,
            "domain_name": name,
            "enabled": payload.enabled,
        })),
    )
        .into_response()
}

async fn list_domains(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = domains::Entity::find().all(db).await.unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "domain_id": r.domain_id,
                "domain_name": r.domain_name,
                "enabled": r.enabled,
                "description": r.description,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn get_domain(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let row = domains::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "domain_id": r.domain_id,
                "domain_name": r.domain_name,
                "enabled": r.enabled,
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

pub fn domains_bootstrap_router() -> Router {
    Router::new()
        .route(
            "/api/v1/domains/bootstrap/status",
            axum::routing::get(bootstrap_status),
        )
        .route("/api/v1/domains/bootstrap", post(bootstrap))
}

pub fn domains_router() -> Router {
    Router::new()
        .route("/api/v1/domains", post(create_domain).get(list_domains))
        .route("/api/v1/domains/", post(create_domain).get(list_domains))
        .route(
            "/api/v1/domains/{id}",
            axum::routing::get(get_domain)
                .patch(patch_domain)
                .delete(delete_domain),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

async fn patch_domain(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let row = domains::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: domains::ActiveModel = r.into();
    if let Some(v) = payload.get("domain_name").and_then(|v| v.as_str()) {
        am.domain_name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({"domain_id": r.domain_id, "domain_name": r.domain_name})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_domain(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let row = domains::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: domains::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
