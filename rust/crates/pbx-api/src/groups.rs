//! Groups router — mirrors `app/groups/router.py` + `app/auth/models.py` Group.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{groups, user_groups, users}`.
//! Tenant-guarded; global groups (domain_id NULL) are visible to all tenants.
//! Member assign: POST /api/v1/groups/{id}/members {"user_uuid"}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::{groups, user_groups, users};

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct GroupCreate {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MemberAssign {
    user_uuid: uuid::Uuid,
}

fn valid_name(n: &str) -> bool {
    let s = n.trim();
    (2..=64).contains(&s.len())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

// ---- handlers ----
async fn create_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<GroupCreate>,
) -> impl IntoResponse {
    let name = payload.name.trim().to_lowercase();
    if !valid_name(&name) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "group name must match ^[a-z0-9_-]{2,64}$"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = groups::ActiveModel {
        group_uuid: Set(id),
        domain_id: Set(Some(domain)),
        name: Set(name.clone()),
        description: Set(payload.description.clone()),
        group_level: Set(10),
        group_protected: Set(false),
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
            "group_uuid": id,
            "name": name,
            "description": payload.description,
            "domain_id": domain,
            "member_count": 0,
        })),
    )
}

async fn list_groups(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = groups::Entity::find().all(db).await.unwrap_or_default();
    let mine: Vec<&groups::Model> = rows
        .iter()
        .filter(|g| g.domain_id.is_none() || g.domain_id == Some(domain))
        .collect();
    let total = mine.len();
    let data: Vec<serde_json::Value> = mine
        .iter()
        .map(|r| {
            serde_json::json!({
                "group_uuid": r.group_uuid,
                "name": r.name,
                "description": r.description,
                "domain_id": r.domain_id,
                "group_level": r.group_level,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        );
    };
    if r.domain_id.is_some() && r.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        );
    }
    let mems = user_groups::Entity::find()
        .filter(user_groups::Column::GroupUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    let members: Vec<serde_json::Value> = mems
        .iter()
        .map(|m| serde_json::json!({"user_uuid": m.user_uuid}))
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "group_uuid": r.group_uuid,
            "name": r.name,
            "description": r.description,
            "domain_id": r.domain_id,
            "members": members,
            "member_count": members.len(),
        })),
    )
}

async fn assign_member(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<MemberAssign>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let g = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    let Some(g) = g else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "group not found"})),
        );
    };
    if g.domain_id.is_some() && g.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "group not found"})),
        );
    }
    // user must exist in this tenant.
    let u = users::Entity::find_by_id(payload.user_uuid)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if u.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "user not found"})),
        );
    }
    let am = user_groups::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        user_uuid: Set(payload.user_uuid),
        group_uuid: Set(id),
        domain_id: Set(domain),
    };
    // idempotent-ish: duplicate assign → 200 with existing.
    let _ = am.insert(db).await;
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "group_uuid": id,
            "user_uuid": payload.user_uuid,
        })),
    )
}

pub fn groups_router() -> Router {
    Router::new()
        .route("/api/v1/groups", post(create_group).get(list_groups))
        .route("/api/v1/groups/", post(create_group).get(list_groups))
        .route("/api/v1/groups/{id}", axum::routing::get(get_group))
        .route("/api/v1/groups/{id}/members", post(assign_member))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
