//! Permissions router — mirrors `app/auth/models.py` Permission catalog.
//!
//! Order: models → schemas → handlers.
//!
//! Global table (no domain_id). GET lazily seeds the static catalog
//! (INSERT-then-ignore-conflict) so the list is never empty.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

// ---- models ----
use pbx_db::entities::permissions;

// ---- schemas / catalog ----
// Static catalog (parity subset of Python permission names).
static CATALOG: &[(&str, &str)] = &[
    ("admin", "full access"),
    ("superadmin", "superuser"),
    ("domain_add", "create domains"),
    ("domain_view", "view domains"),
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
    ("cdr_view", "view call records"),
    ("email_queue_view", "view email queue"),
    ("email_queue_add", "enqueue email"),
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
    Extension(_claims): Extension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    seed_once(db).await;
    let rows = permissions::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    if total == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": "no permissions"})),
        );
    }
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
        Json(serde_json::json!({"data": data, "total": total})),
    )
}

pub fn permissions_router() -> Router {
    Router::new()
        .route("/api/v1/permissions", axum::routing::get(list_permissions))
        .route("/api/v1/permissions/", axum::routing::get(list_permissions))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
