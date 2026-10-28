//! Users router — mirrors `app/auth/models.py` User + users parity.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{users}`.
//! Tenant-guarded (domain_id = routing_db::domain); list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

// ---- models ----
use pbx_db::entities::users;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct UserCreate {
    username: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct UserPatch {
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
}

// ---- handlers ----
async fn create_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<UserCreate>,
) -> impl IntoResponse {
    let username = payload.username.trim().to_owned();
    if username.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "username required"})),
        );
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    // duplicate username within tenant → 409 (parity with Python unique index).
    let exists = users::Entity::find()
        .filter(users::Column::DomainId.eq(domain))
        .filter(users::Column::Username.eq(username.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if exists.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": "username exists"})),
        );
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    // password_hash is a stub (argon2 lives in pbx-core; wiring hash here
    // is a follow-up) — never store blank.
    let pw = payload.password.unwrap_or_default();
    let pw_hash = if pw.is_empty() {
        format!("stub:{id}")
    } else {
        format!("stub:{pw}")
    };
    let am = users::ActiveModel {
        user_uuid: Set(id),
        domain_id: Set(domain),
        username: Set(username.clone()),
        email: Set(payload.email.clone()),
        password_hash: Set(pw_hash),
        enabled: Set(true),
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
            "user_uuid": id,
            "username": username,
            "email": payload.email,
            "domain_id": domain,
            "enabled": true,
        })),
    )
}

async fn list_users(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = users::Entity::find()
        .filter(users::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "user_uuid": r.user_uuid,
                "username": r.username,
                "email": r.email,
                "domain_id": r.domain_id,
                "enabled": r.enabled,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn get_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "user_uuid": r.user_uuid,
                "username": r.username,
                "email": r.email,
                "domain_id": r.domain_id,
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

async fn patch_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<UserPatch>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
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
    let mut am: users::ActiveModel = r.into();
    if let Some(u) = payload.username {
        let u = u.trim().to_owned();
        if u.is_empty() {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": "username required"})),
            )
                .into_response();
        }
        am.username = Set(u);
    }
    if let Some(e) = payload.email {
        am.email = Set(Some(e));
    }
    if let Some(en) = payload.enabled {
        am.enabled = Set(en);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "user_uuid": r.user_uuid,
                "username": r.username,
                "email": r.email,
                "enabled": r.enabled,
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

async fn delete_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
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
    let am: users::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!(null))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}

pub fn users_router() -> Router {
    Router::new()
        .route("/api/v1/users", post(create_user).get(list_users))
        .route("/api/v1/users/", post(create_user).get(list_users))
        .route(
            "/api/v1/users/{id}",
            axum::routing::get(get_user)
                .patch(patch_user)
                .delete(delete_user),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
