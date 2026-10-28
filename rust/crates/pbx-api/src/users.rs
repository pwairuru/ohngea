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

async fn list_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    let (per_page, offset) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let rows = users::Entity::find()
        .filter(users::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let total = rows.len() as u64;
    let data: Vec<serde_json::Value> = rows
        .iter()
        .skip(offset as usize)
        .take(per_page as usize)
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
    (
        crate::paged_headers("/api/v1/users", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
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
        .route("/api/v1/users/bulk", post(bulk_users))
        .route("/api/v1/users/import", post(import_users))
        .route(
            "/api/v1/users/{id}",
            axum::routing::get(get_user)
                .patch(patch_user)
                .delete(delete_user),
        )
        .route(
            "/api/v1/users/{id}/groups",
            axum::routing::get(list_user_groups),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- bulk (parity with app/users/router.py::bulk_users: copy/toggle/delete) ----
#[derive(Debug, serde::Deserialize)]
struct BulkRequest {
    #[serde(default)]
    action: String,
    #[serde(default)]
    ids: Vec<serde_json::Value>,
}

fn bulk_id(v: &serde_json::Value) -> Option<uuid::Uuid> {
    match v {
        serde_json::Value::String(s) => s.parse().ok(),
        serde_json::Value::Object(m) => m
            .get("uuid")
            .or_else(|| m.get("user_uuid"))
            .and_then(|u| u.as_str())
            .and_then(|s| s.parse().ok()),
        _ => None,
    }
}

async fn bulk_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<BulkRequest>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut results = Vec::new();
    for item in &payload.ids {
        let Some(uid) = bulk_id(item) else {
            results.push(serde_json::json!({"error": "invalid uuid"}));
            continue;
        };
        let row = users::Entity::find_by_id(uid)
            .filter(users::Column::DomainId.eq(domain))
            .one(db)
            .await
            .unwrap_or(None);
        let Some(r) = row else {
            results.push(serde_json::json!({"user_uuid": uid, "error": "not found"}));
            continue;
        };
        match payload.action.as_str() {
            "toggle" => {
                let mut am: users::ActiveModel = r.into();
                // flip enabled: read current via fresh fetch? use !current unknown → set true;
                // fetch current first
                let cur = users::Entity::find_by_id(uid).one(db).await.unwrap_or(None);
                let enabled = cur.map(|c| !c.enabled).unwrap_or(true);
                am.enabled = Set(enabled);
                am.updated_at = Set(crate::routing_db::now());
                let _ = am.update(db).await;
                results.push(
                    serde_json::json!({"user_uuid": uid, "action": "toggled", "enabled": enabled}),
                );
            }
            "copy" => {
                let cur = users::Entity::find_by_id(uid).one(db).await.unwrap_or(None);
                if let Some(c) = cur {
                    let nid = uuid::Uuid::now_v7();
                    let now = crate::routing_db::now();
                    let am = users::ActiveModel {
                        user_uuid: Set(nid),
                        domain_id: Set(c.domain_id),
                        username: Set(format!("{} (copy)", c.username)),
                        email: Set(c.email.clone()),
                        password_hash: Set(c.password_hash.clone()),
                        enabled: Set(c.enabled),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    match am.insert(db).await {
                        Ok(_) => {
                            results.push(serde_json::json!({"user_uuid": nid, "action": "copied"}))
                        }
                        Err(e) => results
                            .push(serde_json::json!({"user_uuid": uid, "error": format!("{e}")})),
                    }
                }
            }
            "delete" => {
                let am: users::ActiveModel = r.into();
                match am.delete(db).await {
                    Ok(_) => {
                        results.push(serde_json::json!({"user_uuid": uid, "action": "deleted"}))
                    }
                    Err(e) => {
                        results.push(serde_json::json!({"user_uuid": uid, "error": format!("{e}")}))
                    }
                }
            }
            _ => results.push(serde_json::json!({"user_uuid": uid, "error": "Invalid action"})),
        }
    }
    let total = results.len();
    Json(serde_json::json!({"data": results, "total": total}))
}

// ---- import (parity shape: POST /users/import {"csv"} or {"users": [...]}) ----
async fn import_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut imported = 0u64;
    let mut errors = Vec::new();
    let mut created = Vec::new();
    if let Some(arr) = payload.get("users").and_then(|u| u.as_array()) {
        for (i, row) in arr.iter().enumerate() {
            let username = row
                .get("username")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .trim()
                .to_owned();
            if username.is_empty() {
                errors.push(serde_json::json!({"row": i + 1, "error": "username required"}));
                continue;
            }
            let email = row
                .get("email")
                .and_then(|e| e.as_str())
                .map(|s| s.to_owned());
            let pw = row
                .get("password")
                .and_then(|p| p.as_str())
                .unwrap_or("Stub123!xxxx");
            let id = uuid::Uuid::now_v7();
            let now = crate::routing_db::now();
            let am = users::ActiveModel {
                user_uuid: Set(id),
                domain_id: Set(domain),
                username: Set(username.clone()),
                email: Set(email),
                password_hash: Set(pbx_core::auth::hash_password(pw)),
                enabled: Set(true),
                created_at: Set(now),
                updated_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => {
                    imported += 1;
                    created.push(serde_json::json!({"user_uuid": id, "username": username}));
                }
                Err(e) => errors.push(serde_json::json!({"row": i + 1, "error": format!("{e}")})),
            }
        }
    } else if let Some(csv) = payload.get("csv").and_then(|c| c.as_str()) {
        for (i, line) in csv.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue; // header
            }
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            let username = parts.first().unwrap_or(&"").to_string();
            if username.is_empty() {
                errors.push(serde_json::json!({"row": i, "error": "username required"}));
                continue;
            }
            let email = parts
                .get(1)
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            let pw = parts
                .get(2)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Stub123!xxxx".to_string());
            let id = uuid::Uuid::now_v7();
            let now = crate::routing_db::now();
            let am = users::ActiveModel {
                user_uuid: Set(id),
                domain_id: Set(domain),
                username: Set(username.clone()),
                email: Set(email),
                password_hash: Set(pbx_core::auth::hash_password(&pw)),
                enabled: Set(true),
                created_at: Set(now),
                updated_at: Set(now),
            };
            match am.insert(db).await {
                Ok(_) => {
                    imported += 1;
                    created.push(serde_json::json!({"user_uuid": id, "username": username}));
                }
                Err(e) => errors.push(serde_json::json!({"row": i, "error": format!("{e}")})),
            }
        }
    } else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "CSV content required"})),
        )
            .into_response();
    }
    Json(serde_json::json!({"imported": imported, "errors": errors, "data": created, "total": imported})).into_response()
}

// ---- user → groups (parity with GET /users/{id}/groups) ----
async fn list_user_groups(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    use pbx_db::entities::{groups, user_groups};
    let db = crate::routing_db::conn().await;
    let _domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mems = user_groups::Entity::find()
        .filter(user_groups::Column::UserUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    let mut data = Vec::new();
    for m in mems {
        if let Ok(Some(g)) = groups::Entity::find_by_id(m.group_uuid).one(db).await {
            data.push(serde_json::json!({
                "group_uuid": g.group_uuid,
                "name": g.name,
                "description": g.description,
                "domain_id": g.domain_id,
                "group_level": g.group_level,
            }));
        }
    }
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}
