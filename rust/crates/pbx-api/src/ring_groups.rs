//! Ring groups router — mirrors `app/ring_groups/router.py`.
//!
//! Order: models → schemas → handlers → xml.
//!
//! models: re-exports `pbx_db::entities::{ring_groups, ring_group_members}`.
//! Runtime dialplan goes through `dialplans` (FS XML section).

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

// ---- models ----
use pbx_db::entities::{ring_group_members, ring_groups};

// ---- schemas ----
#[derive(Debug, Deserialize)]
struct MemberIn {
    destination: String,
    #[serde(default)]
    delay: i32,
    #[serde(default = "default_timeout")]
    timeout: i32,
}

fn default_timeout() -> i32 {
    20
}

#[derive(Debug, Deserialize)]
struct RingGroupCreate {
    name: String,
    extension: String,
    #[serde(default = "default_strategy")]
    strategy: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    members: Vec<MemberIn>,
}

fn default_strategy() -> String {
    "simultaneous".to_owned()
}

#[derive(Debug, Serialize)]
struct MemberOut {
    destination: String,
    delay: i32,
    timeout: i32,
}

#[derive(Debug, Serialize)]
struct RingGroupOut {
    ring_group_uuid: String,
    name: String,
    extension: String,
    strategy: String,
    enabled: bool,
    members: Vec<MemberOut>,
}

// ---- handlers ----
fn valid_extension(ext: &str) -> bool {
    let s = ext.trim();
    (2..=7).contains(&s.len()) && s.chars().all(|c| c.is_ascii_digit())
}

async fn create_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<RingGroupCreate>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "name required"})),
        );
    }
    if !valid_extension(&payload.extension) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "extension must be 2-7 digits"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let id = crate::routing_db::new_uuid();
    let now = crate::routing_db::now();
    let am = ring_groups::ActiveModel {
        ring_group_uuid: Set(id.clone()),
        domain_id: Set(domain),
        name: Set(payload.name.trim().to_owned()),
        extension: Set(payload.extension.trim().to_owned()),
        strategy: Set(payload.strategy.clone()),
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
    let mut members = Vec::new();
    for m in &payload.members {
        let mam = ring_group_members::ActiveModel {
            ring_group_member_uuid: Set(crate::routing_db::new_uuid()),
            ring_group_uuid: Set(id.clone()),
            domain_id: Set(domain),
            destination: Set(m.destination.clone()),
            delay: Set(m.delay),
            timeout: Set(m.timeout),
        };
        let _ = mam.insert(db).await;
        members.push(MemberOut {
            destination: m.destination.clone(),
            delay: m.delay,
            timeout: m.timeout,
        });
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(RingGroupOut {
            ring_group_uuid: id,
            name: payload.name,
            extension: payload.extension,
            strategy: payload.strategy,
            enabled: true,
            members,
        })),
    )
}

async fn list_rg(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = ring_groups::Entity::find()
        .filter(ring_groups::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "ring_group_uuid": r.ring_group_uuid,
                "name": r.name,
                "extension": r.extension,
                "strategy": r.strategy,
                "enabled": r.enabled,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = ring_groups::Entity::find_by_id(id.clone())
        .filter(ring_groups::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        );
    };
    let mems = ring_group_members::Entity::find()
        .filter(ring_group_members::Column::RingGroupUuid.eq(id))
        .filter(ring_group_members::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let members: Vec<MemberOut> = mems
        .into_iter()
        .map(|m| MemberOut {
            destination: m.destination,
            delay: m.delay,
            timeout: m.timeout,
        })
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!(RingGroupOut {
            ring_group_uuid: r.ring_group_uuid,
            name: r.name,
            extension: r.extension,
            strategy: r.strategy,
            enabled: r.enabled,
            members,
        })),
    )
}

async fn health_rg() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "ring_groups"}))
}

async fn rg_or_404(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<ring_groups::Model> {
    ring_groups::Entity::find_by_id(id.to_owned())
        .filter(ring_groups::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn patch_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = rg_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Ring group not found"})),
        )
            .into_response();
    };
    let mut am: ring_groups::ActiveModel = r.into();
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(v);
    }
    if let Some(v) = payload.get("strategy").and_then(|v| v.as_str()) {
        am.strategy = Set(v.to_owned());
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "ring_group_uuid": r.ring_group_uuid,
                "name": r.name,
                "extension": r.extension,
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

async fn delete_rg(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = rg_or_404(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Ring group not found"})),
        )
            .into_response();
    };
    let mems = ring_group_members::Entity::find()
        .filter(ring_group_members::Column::RingGroupUuid.eq(id.clone()))
        .filter(ring_group_members::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    for m in mems {
        let mam: ring_group_members::ActiveModel = m.into();
        let _ = mam.delete(db).await;
    }
    let am: ring_groups::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
// Dialplan for a ring group (sequential/simultaneous bridge of members).
// Mirrors `app/ring_groups/service.py` dialplan render + FS `dialplan` section.
pub fn build_ring_group_dialplan_xml(name: &str, members: &[String]) -> String {
    let dial = members
        .iter()
        .map(|d| format!("<action application=\"bridge\" data=\"user/{d}\"/>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<extension name=\"{name}\"><condition>{dial}</condition></extension>")
}

pub fn ring_groups_router() -> Router {
    Router::new()
        .route("/api/v1/ring_groups/health", axum::routing::get(health_rg))
        .route("/api/v1/ring_groups", post(create_rg).get(list_rg))
        .route("/api/v1/ring_groups/", post(create_rg).get(list_rg))
        .route(
            "/api/v1/ring_groups/{id}",
            axum::routing::get(get_rg).patch(patch_rg).delete(delete_rg),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::build_ring_group_dialplan_xml;

    #[test]
    fn ring_group_xml_bridges_each_member() {
        let xml = build_ring_group_dialplan_xml("sales", &["1001".to_owned(), "1002".to_owned()]);
        assert!(xml.contains("sales") && xml.contains("user/1001") && xml.contains("user/1002"));
    }
}
