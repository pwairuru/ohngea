//! Permissions router — mirrors `app/auth/models.py` Permission catalog.
//!
//! Order: models → schemas → handlers.
//!
//! Global table (no domain_id). GET lazily seeds the static catalog
//! (INSERT-then-ignore-conflict) so the list is never empty.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};

// ---- models ----
use pbx_db::entities::permissions;

// ---- schemas / catalog ----
// Static catalog (parity subset of Python permission names).
static CATALOG: &[(&str, &str)] = &[
    ("admin", "full access"),
    ("superadmin", "superuser"),
    ("domain_add", "create domains"),
    ("domain_view", "view domains"),
    ("domain_edit", "edit domains"),
    ("domain_delete", "delete domains"),
    ("extension_view", "view extensions"),
    ("extension_add", "create extensions"),
    ("extension_edit", "edit extensions"),
    ("extension_delete", "delete extensions"),
    ("group_view", "view groups"),
    ("group_add", "create groups"),
    ("group_edit", "edit groups"),
    ("group_delete", "delete groups"),
    ("contact_view", "view contacts"),
    ("contact_add", "create contacts"),
    ("contact_edit", "edit contacts"),
    ("contact_delete", "delete contacts"),
    ("device_view", "view devices"),
    ("device_add", "provision devices"),
    ("device_edit", "edit devices"),
    ("device_delete", "delete devices"),
    ("cdr_view", "view call records"),
    ("cdr_add", "create call records"),
    ("email_queue_view", "view email queue"),
    ("email_queue_add", "enqueue email"),
    ("user_view", "view users"),
    ("user_add", "create users"),
    ("user_edit", "edit users"),
    ("user_delete", "delete users"),
    ("group_permission_view", "view group permissions"),
    ("group_permission_edit", "edit group permissions"),
    ("default_setting_view", "view default settings"),
    ("default_setting_add", "create default settings"),
    ("default_setting_edit", "edit default settings"),
    ("default_setting_delete", "delete default settings"),
    ("user_setting_view", "view user settings"),
    ("user_setting_edit", "edit user settings"),
];

async fn seed_once(db: &sea_orm::DatabaseConnection) {
    for (name, desc) in CATALOG {
        let am = permissions::ActiveModel {
            permission_uuid: Set(uuid::Uuid::now_v7()),
            name: Set((*name).to_owned()),
            description: Set(Some((*desc).to_owned())),
        };
        // Conflict (already seeded) → ignore.
        let _ = am.insert(db).await;
    }
}

// ---- handlers ----
async fn list_permissions(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_permission_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    seed_once(db).await;
    let paginator = permissions::Entity::find().paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    if total == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            crate::paged_headers("/api/v1/permissions", page, per_page, 0),
            Json(serde_json::json!({"detail": "no permissions"})),
        )
            .into_response();
    }
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "permission_uuid": r.permission_uuid,
                "name": r.name,
                "description": r.description,
            })
        })
        .collect();
    (
        StatusCode::OK,
        crate::paged_headers("/api/v1/permissions", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

pub fn permissions_router() -> Router {
    Router::new()
        .route("/api/v1/permissions", axum::routing::get(list_permissions))
        .route("/api/v1/permissions/", axum::routing::get(list_permissions))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
