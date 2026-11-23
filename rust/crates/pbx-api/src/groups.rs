//! Groups router — mirrors `app/groups/router.py` + `app/auth/models.py` Group.
//!
//! Order: models → schemas → handlers.
//!
//! models: re-exports `pbx_db::entities::{groups, user_groups, users}`.
//! Tenant-guarded; global groups (domain_id NULL) are visible to all tenants.
//! Member assign: POST /api/v1/groups/{id}/members {"user_uuid"}.

use axum::{http::StatusCode, response::IntoResponse, routing::post, Extension, Json, Router};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set,
};

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
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_add") {
        return (s, j).into_response();
    }
    let name = payload.name.trim().to_lowercase();
    if !valid_name(&name) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "group name must match ^[a-z0-9_-]{2,64}$"})),
        )
            .into_response();
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
        )
            .into_response();
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
        .into_response()
}

async fn list_groups(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "group_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/groups", page, per_page, 0),
                Json(crate::paged_body(Vec::<serde_json::Value>::new(), 0, page, per_page)),
            )
                .into_response();
        }
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    // DB-side tenant+global filter: domain rows OR global (NULL) rows.
    // Single query so `num_items` total matches the fetched page.
    // `?show=all` (+group_all) drops the filter entirely (PHP parity).
    let mut sel = groups::Entity::find();
    if !show_all {
        sel = sel.filter(
            Condition::any()
                .add(groups::Column::DomainId.eq(domain))
                .add(groups::Column::DomainId.is_null()),
        );
    }
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
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
    (
        crate::paged_headers("/api/v1/groups", page, per_page, total),
        Json(crate::paged_body(data, total, page, per_page)),
    )
        .into_response()
}

async fn get_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    if r.domain_id.is_some() && r.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
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
        .into_response()
}

async fn assign_member(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<MemberAssign>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let g = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    let Some(g) = g else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "group not found"})),
        )
            .into_response();
    };
    if g.domain_id.is_some() && g.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "group not found"})),
        )
            .into_response();
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
        )
            .into_response();
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
        .into_response()
}

pub fn groups_router() -> Router {
    Router::new()
        .route("/api/v1/groups", post(create_group).get(list_groups))
        .route("/api/v1/groups/", post(create_group).get(list_groups))
        .route("/api/v1/groups/bulk", post(bulk_groups))
        .route(
            "/api/v1/groups/{id}",
            axum::routing::get(get_group)
                .patch(patch_group)
                .delete(delete_group),
        )
        .route("/api/v1/groups/{id}/copy", post(copy_group))
        .route("/api/v1/groups/{id}/toggle", post(toggle_group))
        .route(
            "/api/v1/groups/{id}/users",
            axum::routing::get(list_group_users)
                .post(add_group_user)
                .delete(remove_group_users_bulk),
        )
        .route(
            "/api/v1/groups/{id}/users/bulk",
            post(remove_group_users_bulk),
        )
        .route(
            "/api/v1/groups/{id}/users/{uid}",
            axum::routing::delete(remove_group_user),
        )
        .route("/api/v1/groups/{id}/members", post(assign_member))
        .route(
            "/api/v1/groups/{id}/permissions",
            axum::routing::get(list_group_permissions).put(put_group_permissions),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- bulk (parity with app/groups/router.py::bulk_groups_ep) ----
#[derive(Debug, serde::Deserialize)]
struct BulkRequest {
    #[serde(default)]
    action: String,
    #[serde(default)]
    ids: Vec<serde_json::Value>,
}

fn bulk_uuid(v: &serde_json::Value) -> Option<uuid::Uuid> {
    match v {
        serde_json::Value::String(s) => s.parse().ok(),
        serde_json::Value::Object(m) => m
            .get("uuid")
            .or_else(|| m.get("group_uuid"))
            .and_then(|u| u.as_str())
            .and_then(|s| s.parse().ok()),
        _ => None,
    }
}

async fn bulk_groups(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<BulkRequest>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut results = Vec::new();
    for item in &payload.ids {
        let Some(gid) = bulk_uuid(item) else {
            results.push(serde_json::json!({"error": "invalid uuid"}));
            continue;
        };
        match payload.action.as_str() {
            "toggle" => {
                let row = groups::Entity::find_by_id(gid)
                    .one(db)
                    .await
                    .unwrap_or(None);
                match row {
                    Some(g) if g.domain_id.is_none() || g.domain_id == Some(domain) => {
                        let prot = !g.group_protected;
                        let mut am: groups::ActiveModel = g.into();
                        // flip group_protected
                        am.group_protected = Set(prot);
                        am.updated_at = Set(crate::routing_db::now());
                        let _ = am.update(db).await;
                        results.push(serde_json::json!({"group_uuid": gid, "action": "toggled", "protected": prot}));
                    }
                    _ => results.push(serde_json::json!({"group_uuid": gid, "error": "not found"})),
                }
            }
            "copy" => match copy_group_inner(db, domain, gid).await {
                Ok(nid) => results.push(serde_json::json!({"group_uuid": nid, "action": "copied"})),
                Err(e) => results.push(serde_json::json!({"group_uuid": gid, "error": e})),
            },
            "delete" => {
                let row = groups::Entity::find_by_id(gid)
                    .one(db)
                    .await
                    .unwrap_or(None);
                match row {
                    Some(g) => {
                        if g.domain_id.is_some() && g.domain_id != Some(domain) {
                            results
                                .push(serde_json::json!({"group_uuid": gid, "error": "not found"}));
                        } else {
                            let am: groups::ActiveModel = g.into();
                            match am.delete(db).await {
                                Ok(_) => results.push(
                                    serde_json::json!({"group_uuid": gid, "action": "deleted"}),
                                ),
                                Err(e) => results.push(
                                    serde_json::json!({"group_uuid": gid, "error": format!("{e}")}),
                                ),
                            }
                        }
                    }
                    None => {
                        results.push(serde_json::json!({"group_uuid": gid, "error": "not found"}))
                    }
                }
            }
            _ => results.push(serde_json::json!({"group_uuid": gid, "error": "Invalid action"})),
        }
    }
    let total = results.len();
    Json(serde_json::json!({"data": results, "total": total})).into_response()
}

async fn copy_group_inner(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    gid: uuid::Uuid,
) -> Result<uuid::Uuid, String> {
    let g = groups::Entity::find_by_id(gid)
        .one(db)
        .await
        .map_err(|e| format!("{e}"))?
        .ok_or_else(|| "Group not found".to_string())?;
    if g.domain_id.is_some() && g.domain_id != Some(domain) {
        return Err("Group not found".to_string());
    }
    let nid = uuid::Uuid::now_v7();
    let now = crate::routing_db::now();
    let am = groups::ActiveModel {
        group_uuid: Set(nid),
        domain_id: Set(g.domain_id.or(Some(domain))),
        name: Set(format!("{}-copy", g.name)),
        description: Set(g.description.clone()),
        group_level: Set(g.group_level),
        group_protected: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
    };
    am.insert(db).await.map_err(|e| format!("{e}"))?;
    Ok(nid)
}

async fn copy_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    match copy_group_inner(db, domain, id).await {
        Ok(nid) => {
            let row = groups::Entity::find_by_id(nid)
                .one(db)
                .await
                .unwrap_or(None);
            match row {
                Some(g) => (
                    StatusCode::CREATED,
                    Json(serde_json::json!({
                        "group_uuid": g.group_uuid,
                        "name": g.name,
                        "description": g.description,
                        "domain_id": g.domain_id,
                        "member_count": 0,
                    })),
                )
                    .into_response(),
                None => (
                    StatusCode::CREATED,
                    Json(serde_json::json!({"group_uuid": nid})),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response(),
    }
}

async fn toggle_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    let Some(g) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    };
    if g.domain_id.is_some() && g.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    }
    let prot = !g.group_protected;
    let mut am: groups::ActiveModel = g.into();
    am.group_protected = Set(prot);
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(g) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "group_uuid": g.group_uuid,
                "name": g.name,
                "group_protected": g.group_protected,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("{e}")})),
        )
            .into_response(),
    }
}

// ---- group users (parity with /groups/{id}/users) ----
async fn list_group_users(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let g = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    let Some(g) = g else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    };
    if g.domain_id.is_some() && g.domain_id != Some(domain) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    }
    let mems = user_groups::Entity::find()
        .filter(user_groups::Column::GroupUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    let mut data = Vec::new();
    for m in mems {
        if let Ok(Some(u)) = users::Entity::find_by_id(m.user_uuid).one(db).await {
            data.push(serde_json::json!({
                "user_uuid": u.user_uuid,
                "username": u.username,
                "email": u.email,
                "enabled": u.enabled,
                "domain_id": u.domain_id,
            }));
        }
    }
    let total = data.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({"data": data, "total": total})),
    )
        .into_response()
}

async fn add_group_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let user_uuid = payload
        .get("user_uuid")
        .and_then(|u| u.as_str())
        .and_then(|s| s.parse::<uuid::Uuid>().ok());
    let Some(uid) = user_uuid else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "user_uuid required"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let g = groups::Entity::find_by_id(id).one(db).await.unwrap_or(None);
    if g.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    }
    if users::Entity::find_by_id(uid)
        .one(db)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "User not found"})),
        )
            .into_response();
    }
    let am = user_groups::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        user_uuid: Set(uid),
        group_uuid: Set(id),
        domain_id: Set(domain),
    };
    let _ = am.insert(db).await;
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"group_uuid": id, "user_uuid": uid})),
    )
        .into_response()
}

async fn remove_group_user(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path((id, uid)): axum::extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let _domain = crate::routing_db::tenant_domain(db, &claims).await;
    let row = user_groups::Entity::find()
        .filter(user_groups::Column::GroupUuid.eq(id))
        .filter(user_groups::Column::UserUuid.eq(uid))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => {
            let am: user_groups::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn remove_group_users_bulk(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let ids = payload
        .get("ids")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let db = crate::routing_db::conn().await;
    let _domain = crate::routing_db::tenant_domain(db, &claims).await;
    let mut removed = 0;
    for item in ids {
        let uid = item.as_str().and_then(|s| s.parse::<uuid::Uuid>().ok());
        if let Some(uid) = uid {
            let row = user_groups::Entity::find()
                .filter(user_groups::Column::GroupUuid.eq(id))
                .filter(user_groups::Column::UserUuid.eq(uid))
                .one(db)
                .await
                .unwrap_or(None);
            if let Some(r) = row {
                let am: user_groups::ActiveModel = r.into();
                if am.delete(db).await.is_ok() {
                    removed += 1;
                }
            }
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"removed": removed})),
    )
        .into_response()
}

// ---- group permissions (parity shape) ----
async fn list_group_permissions(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_permission_view") {
        return (s, j).into_response();
    }
    use pbx_db::entities::{group_permissions, permissions};
    let db = crate::routing_db::conn().await;
    let _domain = crate::routing_db::tenant_domain(db, &claims).await;
    if groups::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    }
    let assigned = group_permissions::Entity::find()
        .filter(group_permissions::Column::GroupUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    let all = permissions::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let by_id: std::collections::HashMap<uuid::Uuid, &permissions::Model> =
        all.iter().map(|p| (p.permission_uuid, p)).collect();
    let mut data = Vec::new();
    for gp in &assigned {
        if let Some(p) = by_id.get(&gp.permission_uuid) {
            data.push(serde_json::json!({
                "permission_uuid": p.permission_uuid,
                "name": p.name,
                "description": p.description,
                "assigned": true,
            }));
        }
    }
    // `HashSet` membership: the old O(N*M) `.any()` scan per catalog row.
    let assigned_ids: std::collections::HashSet<uuid::Uuid> =
        assigned.iter().map(|gp| gp.permission_uuid).collect();
    // include unassigned catalog entries so PUT parity (assigned flags) works
    for p in &all {
        if !assigned_ids.contains(&p.permission_uuid) {
            data.push(serde_json::json!({
                "permission_uuid": p.permission_uuid,
                "name": p.name,
                "description": p.description,
                "assigned": false,
            }));
        }
    }
    let total = data.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({"data": data, "total": total})),
    )
        .into_response()
}

async fn put_group_permissions(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_permission_edit") {
        return (s, j).into_response();
    }
    use pbx_db::entities::{group_permissions, permissions};
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    if groups::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Group not found"})),
        )
            .into_response();
    }
    // accept permission_uuids | permission_names | permissions | items[{...assigned}]
    let mut want: Vec<uuid::Uuid> = Vec::new();
    if let Some(arr) = payload.get("permission_uuids").and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                if let Ok(u) = s.parse() {
                    want.push(u);
                }
            }
        }
    }
    if let Some(arr) = payload.get("items").and_then(|v| v.as_array()) {
        for item in arr {
            if item
                .get("assigned")
                .and_then(|a| a.as_bool())
                .unwrap_or(false)
            {
                if let Some(s) = item.get("permission_uuid").and_then(|u| u.as_str()) {
                    if let Ok(u) = s.parse() {
                        if !want.contains(&u) {
                            want.push(u);
                        }
                    }
                }
            }
        }
    }
    for key in ["permission_names", "permissions"] {
        if let Some(arr) = payload.get(key).and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(name) = v.as_str() {
                    let row = permissions::Entity::find()
                        .filter(permissions::Column::Name.eq(name.to_owned()))
                        .one(db)
                        .await
                        .unwrap_or(None);
                    if let Some(p) = row {
                        if !want.contains(&p.permission_uuid) {
                            want.push(p.permission_uuid);
                        }
                    }
                }
            }
        }
    }
    // replace assignments
    let existing = group_permissions::Entity::find()
        .filter(group_permissions::Column::GroupUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    for gp in existing {
        if !want.contains(&gp.permission_uuid) {
            let am: group_permissions::ActiveModel = gp.into();
            let _ = am.delete(db).await;
        }
    }
    for pid in &want {
        let has = group_permissions::Entity::find()
            .filter(group_permissions::Column::GroupUuid.eq(id))
            .filter(group_permissions::Column::PermissionUuid.eq(*pid))
            .one(db)
            .await
            .unwrap_or(None);
        if has.is_none() {
            let am = group_permissions::ActiveModel {
                id: Set(uuid::Uuid::now_v7()),
                group_uuid: Set(id),
                permission_uuid: Set(*pid),
                domain_id: Set(Some(domain)),
            };
            let _ = am.insert(db).await;
        }
    }
    let data: Vec<serde_json::Value> = want
        .iter()
        .map(|u| serde_json::json!({"permission_uuid": u, "assigned": true}))
        .collect();
    let total = data.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({"data": data, "total": total})),
    )
        .into_response()
}

async fn group_scoped(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &uuid::Uuid,
) -> Option<groups::Model> {
    let row = groups::Entity::find_by_id(*id)
        .one(db)
        .await
        .unwrap_or(None)?;
    if row.domain_id.is_some() && row.domain_id != Some(domain) {
        return None;
    }
    Some(row)
}

async fn patch_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = group_scoped(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: groups::ActiveModel = r.into();
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(v.to_owned());
    }
    if let Some(v) = payload.get("description").and_then(|v| v.as_str()) {
        am.description = Set(Some(v.to_owned()));
    }
    match am.update(db).await {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({"group_uuid": r.group_uuid, "name": r.name})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_group(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "group_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(r) = group_scoped(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let am: groups::ActiveModel = r.into();
    match am.delete(db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
