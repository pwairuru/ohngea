//! Users router — mirrors `app/auth/models.py` User + users parity.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{users}`.
//! Tenant-guarded (domain_id = routing_db::domain); list envelope {data,total}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_add") {
        return (s, j).into_response();
    }
    let username = payload.username.trim().to_owned();
    if username.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "username required"})),
        )
            .into_response();
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
        )
            .into_response();
    }
    let id = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    // Real bcrypt hash (parity with app/auth/service.py hash_password).
    // Blank password hashes a random id so verify never passes on empty.
    let pw = payload.password.unwrap_or_default();
    let pw_hash = if pw.is_empty() {
        pbx_core::auth::hash_password(&id.to_string())
    } else {
        pbx_core::auth::hash_password(&pw)
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
        )
            .into_response();
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
        .into_response()
}

async fn list_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let paginator = users::Entity::find()
        .filter(users::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
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
    (
        crate::paged_headers("/api/v1/users", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_view") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_edit") {
        return (s, j).into_response();
    }
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_delete") {
        return (s, j).into_response();
    }
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
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
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
        .route(
            "/api/v1/users/{id}/settings",
            axum::routing::get(get_user_settings).put(put_user_settings),
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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_edit") {
        return (s, j).into_response();
    }
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
                let enabled = !r.enabled;
                let mut am: users::ActiveModel = r.into();
                am.enabled = Set(enabled);
                am.updated_at = Set(crate::routing_db::now());
                let _ = am.update(db).await;
                results.push(
                    serde_json::json!({"user_uuid": uid, "action": "toggled", "enabled": enabled}),
                );
            }
            "copy" => {
                let c = r;
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
                    Err(e) => {
                        results.push(serde_json::json!({"user_uuid": uid, "error": format!("{e}")}))
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
    Json(serde_json::json!({"data": results, "total": total})).into_response()
}

// ---- import (parity shape: POST /users/import {"csv"} or {"users": [...]}) ----
async fn import_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_add") {
        return (s, j).into_response();
    }
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

// ---- user_settings (parity with app/users/router.py::{get,put}_user_settings) ----
fn user_setting_json(r: &pbx_db::entities::user_settings::Model) -> serde_json::Value {
    serde_json::json!({
        "user_setting_uuid": r.user_setting_uuid,
        "user_uuid": r.user_uuid,
        "domain_id": r.domain_id,
        "category": r.user_setting_category,
        "user_setting_category": r.user_setting_category,
        "subcategory": r.user_setting_subcategory,
        "user_setting_subcategory": r.user_setting_subcategory,
        "name": r.user_setting_name,
        "user_setting_name": r.user_setting_name,
        "value": r.user_setting_value,
        "user_setting_value": r.user_setting_value,
        "enabled": r.user_setting_enabled,
        "user_setting_enabled": r.user_setting_enabled,
    })
}

/// Self-or-perm gate (Python `put_user_settings` parity): the user themself
/// always passes; anyone else needs `user_setting_view`/`user_edit`/`admin`.
fn self_or(
    claims: &pbx_core::auth::Claims,
    target: &uuid::Uuid,
    perm: &str,
) -> Option<(StatusCode, Json<serde_json::Value>)> {
    if claims.sub == target.to_string() {
        return None;
    }
    crate::tenant::deny_if_missing(claims, perm)
}

async fn get_user_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    // Python GET is auth-only; we tighten to self-or-view and note parity.
    if let Some((s, j)) = self_or(&claims, &id, "user_setting_view") {
        // Fall back to user_view so profile readers (not settings admins)
        // can still see a user's settings row.
        if crate::tenant::deny_if_missing(&claims, "user_view").is_some() {
            return (s, j).into_response();
        }
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if user.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "User not found"})),
        )
            .into_response();
    }
    let rows = pbx_db::entities::user_settings::Entity::find()
        .filter(pbx_db::entities::user_settings::Column::UserUuid.eq(id))
        .filter(pbx_db::entities::user_settings::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows.iter().map(user_setting_json).collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn put_user_settings(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Python parity: self bypasses; others need user_setting_view/user_edit.
    if claims.sub != id.to_string()
        && crate::tenant::deny_if_missing(&claims, "user_setting_view").is_some()
        && crate::tenant::deny_if_missing(&claims, "user_edit").is_some()
    {
        let (s, j) = crate::tenant::forbidden("user_setting_view");
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(u) = user else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "User not found"})),
        )
            .into_response();
    };
    let settings = payload
        .get("settings")
        .or_else(|| payload.get("data"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    // Replace-all (Python parity): delete existing, recreate from payload.
    use sea_orm::EntityTrait as _;
    let _ = pbx_db::entities::user_settings::Entity::delete_many()
        .filter(pbx_db::entities::user_settings::Column::UserUuid.eq(id))
        .exec(db)
        .await;
    for s in &settings {
        let am = pbx_db::entities::user_settings::ActiveModel {
            user_setting_uuid: Set(uuid::Uuid::now_v7()),
            user_uuid: Set(id),
            domain_id: Set(u.domain_id),
            user_setting_category: Set(s
                .get("category")
                .or_else(|| s.get("user_setting_category"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned()),
            user_setting_subcategory: Set(s
                .get("subcategory")
                .or_else(|| s.get("user_setting_subcategory"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
            user_setting_name: Set(s
                .get("name")
                .or_else(|| s.get("user_setting_name"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
            user_setting_value: Set(s
                .get("value")
                .or_else(|| s.get("user_setting_value"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
            user_setting_enabled: Set(s
                .get("enabled")
                .or_else(|| s.get("user_setting_enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true)),
            user_setting_description: Set(s
                .get("description")
                .or_else(|| s.get("user_setting_description"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_owned())),
        };
        let _ = am.insert(db).await;
    }
    Json(serde_json::json!({"status": "updated"})).into_response()
}

// ---- user → groups (parity with GET /users/{id}/groups) ----
async fn list_user_groups(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_view") {
        return (s, j).into_response();
    }
    use pbx_db::entities::{groups, user_groups};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    // Tenant check: user must belong to caller's domain, else 404 (no cross-tenant leak).
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::DomainId.eq(domain))
        .one(db)
        .await
        .unwrap_or(None);
    if user.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let mems = user_groups::Entity::find()
        .filter(user_groups::Column::UserUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    let mut data = Vec::new();
    for m in mems {
        if let Ok(Some(g)) = groups::Entity::find_by_id(m.group_uuid)
            .filter(groups::Column::DomainId.eq(domain))
            .one(db)
            .await
        {
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
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}
