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
    #[serde(default, alias = "domain_description")]
    domain_description: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, serde::Deserialize)]
struct Bootstrap {
    domain_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, alias = "domain_description")]
    domain_description: Option<String>,
    /// First admin login (Python `create_domain` parity: defaults admin/admin).
    #[serde(default)]
    admin_username: Option<String>,
    #[serde(default)]
    admin_password: Option<String>,
}

// ---- handlers ----
async fn bootstrap_status() -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    // Setup is needed until an admin user exists: boot seeds the `default`
    // domain row with no users, so domain presence alone must NOT count.
    let has = pbx_db::entities::users::Entity::find()
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();
    let has_domains = domains::Entity::find()
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();
    Json(serde_json::json!({"needs_setup": !has, "has_domains": has_domains}))
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
    // Already set up once a user exists (boot seeds the `default` domain
    // with no users, so a domain row alone must NOT block first bootstrap).
    if pbx_db::entities::users::Entity::find()
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
        description: Set(payload.description.clone().or(payload.domain_description.clone())),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        );
    }
    // Seed global groups + permission catalog + full superadmin grants, then
    // the tenant admin user (Python `create_domain` / PHP `groups::defaults`
    // parity). On seed failure roll the domain back so a retry is possible
    // instead of stranding a half-bootstrapped tenant behind the 409 above.
    let seed_result = async {
        pbx_db::seeds::seed_bootstrap_permissions(db).await?;
        let superadmin = pbx_db::seeds::seed_bootstrap_groups(db).await?;
        pbx_db::seeds::seed_bootstrap_group_permissions(db, superadmin).await?;
        let username = payload
            .admin_username
            .clone()
            .map(|u| u.trim().to_owned())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| "admin".to_owned());
        let password = payload
            .admin_password
            .clone()
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| "admin".to_owned());
        let hash = pbx_core::auth::hash_password(&password);
        pbx_db::seeds::seed_bootstrap_admin(db, id, superadmin, &username, &hash).await?;
        Ok::<String, sea_orm::DbErr>(username)
    }
    .await;
    let admin_username = match seed_result {
        Ok(u) => u,
        Err(e) => {
            let _ = domains::Entity::delete_by_id(id).exec(db).await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("bootstrap seed failed: {e}")})),
            );
        }
    };
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "domain_id": id,
            "domain_name": name,
            "enabled": true,
            "admin_username": admin_username,
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
        description: Set(payload.description.clone().or(payload.domain_description.clone())),
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
    // Isolation: `domain_all` (or superadmin/* bypass) sees every tenant;
    // a plain `domain_view` holder sees only their own domain — never the
    // full tenant list. Missing own row → empty, never auto-inserted.
    let rows: Vec<domains::Model> = if crate::tenant::has_permission(&claims, "domain_all") {
        domains::Entity::find().all(db).await.unwrap_or_default()
    } else {
        // Required (never auto-inserting): unknown own tenant → empty list.
        match crate::routing_db::tenant_domain_required(db, &claims).await {
            Ok(own) => {
                domains::Entity::find_by_id(own)
                    .one(db)
                    .await
                    .unwrap_or(None)
                    .into_iter()
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    };
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "domain_id": r.domain_id,
                "domain_name": r.domain_name,
                "enabled": r.enabled,
                "description": r.description,
                "domain_description": r.description,
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
    // Isolation: other tenants are hidden (404, not 403 — no existence oracle)
    // unless the caller holds `domain_all` (superadmin/* bypass included).
    if id != crate::routing_db::domain(&claims)
        && !crate::tenant::has_permission(&claims, "domain_all")
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
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
                "description": r.description,
                "domain_description": r.description,
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
    if let Some(v) = payload.get("description").or_else(|| payload.get("domain_description")).and_then(|v| v.as_str()) {
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

/// Operational tables whose rows block a domain delete (409 with per-table
/// counts). Auth/settings rows (users, groups, memberships, vars, …) cascade
/// instead — they are tenant-owned and deletable with the tenant.
macro_rules! count_blockers {
    ($db:expr, $id:expr, $blockers:expr, $(($table:literal, $entity:ty, $col:expr)),+ $(,)?) => {{
        $(
            let n = <$entity>::find()
                .filter($col.eq($id))
                .count($db)
                .await
                .unwrap_or(0);
            if n > 0 {
                $blockers.insert($table.to_owned(), n);
            }
        )+
    }};
}

async fn delete_domain(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "domain_delete") {
        return (s, j).into_response();
    }
    // Isolation: same rule as get (hidden tenants → 404).
    if id != crate::routing_db::domain(&claims)
        && !crate::tenant::has_permission(&claims, "domain_all")
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    if domains::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    // Live service config blocks the delete: report what references the
    // tenant instead of orphaning a single row.
    let mut blockers: std::collections::BTreeMap<String, u64> = Default::default();
    {
        use pbx_db::entities::*;
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
        count_blockers!(
            db, id, blockers,
            ("extensions", extensions::Entity, extensions::Column::DomainId),
            ("gateways", gateways::Entity, gateways::Column::DomainId),
            ("dialplans", dialplans::Entity, dialplans::Column::DomainId),
            ("sip_profiles", sip_profiles::Entity, sip_profiles::Column::DomainId),
            ("devices", devices::Entity, devices::Column::DomainId),
            ("contacts", contacts::Entity, contacts::Column::DomainId),
            ("call_centers", call_centers::Entity, call_centers::Column::DomainId),
            ("ivr_menus", ivr_menus::Entity, ivr_menus::Column::DomainId),
            ("conferences", conferences::Entity, conferences::Column::DomainId),
            ("voicemails", voicemails::Entity, voicemails::Column::DomainId),
            ("ring_groups", ring_groups::Entity, ring_groups::Column::DomainId),
            ("faxes", faxes::Entity, faxes::Column::DomainId),
            ("destinations", destinations::Entity, destinations::Column::DomainId),
        );
    }
    if !blockers.is_empty() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "detail": "domain has dependent rows",
                "blockers": blockers,
            })),
        )
            .into_response();
    }
    // Cascade tenant-owned rows (children before parents for FK order), then
    // the domain itself. Works on sqlite (no FK enforcement) and postgres
    // (mirrors its ON DELETE CASCADE targets explicitly so no orphan survives
    // on either backend).
    {
        use pbx_db::entities::*;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let _ = user_groups::Entity::delete_many()
            .filter(user_groups::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = group_permissions::Entity::delete_many()
            .filter(group_permissions::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = user_settings::Entity::delete_many()
            .filter(user_settings::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = user_logs::Entity::delete_many()
            .filter(user_logs::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = users::Entity::delete_many()
            .filter(users::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = groups::Entity::delete_many()
            .filter(groups::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = domain_settings::Entity::delete_many()
            .filter(domain_settings::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = vars::Entity::delete_many()
            .filter(vars::Column::DomainId.eq(id))
            .exec(db)
            .await;
        let _ = email_queue::Entity::delete_many()
            .filter(email_queue::Column::DomainId.eq(id))
            .exec(db)
            .await;
    }
    match domains::Entity::delete_by_id(id).exec(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
