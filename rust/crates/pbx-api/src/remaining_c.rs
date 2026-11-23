//! Spillover replacement (part C): per-table CRUD for access_controls
//! (+ nodes), fifo (+ members), fifo_list status view, webrtc_settings
//! (legacy CRUD + Python-canonical settings routes), emergency_logs and
//! event_guard_logs.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use serde_json::Value;

// ---- models ----
type Claims = pbx_core::auth::Claims;
use crate::remaining_a::{b, envelope, s, tenant_domain, unprocessable};
use pbx_db::entities::{
    access_control_nodes, access_controls, emergency_logs, event_guard_logs, fifo, fifo_members,
    webrtc_settings,
};

// ---- handlers: access_controls (+ nodes child) ----
fn ser_acl(r: &access_controls::Model) -> Value {
    serde_json::json!({
        "access_control_uuid": r.access_control_uuid,
        "domain_id": r.domain_id.to_string(),
        "access_control_name": r.access_control_name,
        "name": r.access_control_name,
        "access_control_default": r.access_control_default,
        "access_control_description": r.access_control_description,
    })
}

fn ser_acl_node(n: &access_control_nodes::Model) -> Value {
    serde_json::json!({
        "access_control_node_uuid": n.access_control_node_uuid,
        "access_control_uuid": n.access_control_uuid,
        "node_type": n.node_type,
        "type": n.node_type,
        "node_cidr": n.node_cidr,
        "cidr": n.node_cidr,
        "node_description": n.node_description,
    })
}

async fn create_acl(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["access_control_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = access_controls::ActiveModel {
        access_control_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        access_control_name: Set(name),
        access_control_default: Set(
            s(&payload, &["access_control_default"]).unwrap_or_else(|| "deny".to_owned())
        ),
        access_control_description: Set(s(
            &payload,
            &["access_control_description", "description"],
        )),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_acl(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_acls(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = access_controls::Entity::find()
        .filter(access_controls::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_acl).collect()).into_response()
}

async fn scoped_acl(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<access_controls::Model> {
    match access_controls::Entity::find_by_id(id.to_owned())
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => Some(r),
        _ => None,
    }
}

async fn get_acl(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_acl(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_acl(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_acl(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = scoped_acl(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: access_controls::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["access_control_name", "name"]) {
        am.access_control_name = Set(v);
    }
    if let Some(v) = s(&payload, &["access_control_default"]) {
        am.access_control_default = Set(v);
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_acl(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_acl(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_acl(db, domain, &id).await {
        Some(r) => {
            let am: access_controls::ActiveModel = r.into();
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

async fn create_acl_node(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_acl(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let (Some(node_type), Some(cidr)) = (
        s(&payload, &["node_type", "type"]),
        s(&payload, &["node_cidr", "cidr"]),
    ) else {
        return unprocessable("node_type and node_cidr required").into_response();
    };
    let now = crate::routing_db::now();
    let am = access_control_nodes::ActiveModel {
        access_control_node_uuid: Set(crate::routing_db::new_uuid()),
        access_control_uuid: Set(id.clone()),
        domain_id: Set(domain),
        node_type: Set(node_type),
        node_cidr: Set(cidr),
        node_description: Set(s(&payload, &["node_description", "description"])),
        created_at: Set(now),
        updated_at: Set(now),
    };
    match am.insert(db).await {
        Ok(n) => (StatusCode::CREATED, Json(ser_acl_node(&n))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_acl_nodes(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_acl(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let rows = access_control_nodes::Entity::find()
        .filter(access_control_nodes::Column::AccessControlUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_acl_node).collect()).into_response()
}

async fn delete_acl_node(
    Extension(claims): Extension<Claims>,
    axum::extract::Path((id, node)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "access_control_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_acl(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let hit = access_control_nodes::Entity::find_by_id(node)
        .one(db)
        .await
        .unwrap_or(None)
        .filter(|n| n.access_control_uuid == id);
    match hit {
        Some(n) => {
            let am: access_control_nodes::ActiveModel = n.into();
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

fn acl_router() -> Router {
    Router::new()
        .route(
            "/api/v1/access_controls",
            axum::routing::post(create_acl).get(list_acls),
        )
        .route(
            "/api/v1/access_controls/{id}",
            axum::routing::get(get_acl)
                .patch(patch_acl)
                .delete(delete_acl),
        )
        .route(
            "/api/v1/access_controls/{id}/nodes",
            axum::routing::post(create_acl_node).get(list_acl_nodes),
        )
        .route(
            "/api/v1/access_controls/{id}/nodes/{node}",
            axum::routing::delete(delete_acl_node),
        )
}

// ---- handlers: fifo (+ members child) ----
fn ser_fifo(r: &fifo::Model) -> Value {
    serde_json::json!({
        "fifo_uuid": r.fifo_uuid,
        "domain_id": r.domain_id.to_string(),
        "dialplan_uuid": r.dialplan_uuid,
        "fifo_name": r.fifo_name,
        "name": r.fifo_name,
        "fifo_extension": r.fifo_extension,
        "extension": r.fifo_extension,
        "fifo_agent_queue": r.fifo_agent_queue,
        "fifo_agent_status": r.fifo_agent_status,
        "fifo_enabled": r.fifo_enabled,
        "enabled": r.fifo_enabled,
        "fifo_description": r.fifo_description,
        "fifo_strategy": r.fifo_strategy,
        "fifo_timeout_seconds": r.fifo_timeout_seconds,
        "fifo_exit_key": r.fifo_exit_key,
        "fifo_exit_action": r.fifo_exit_action,
        "fifo_music": r.fifo_music,
        "fifo_order": r.fifo_order,
        "fifo_caller_id_name": r.fifo_caller_id_name,
        "fifo_caller_id_number": r.fifo_caller_id_number,
    })
}

fn ser_fifo_member(m: &fifo_members::Model) -> Value {
    serde_json::json!({
        "fifo_member_uuid": m.fifo_member_uuid,
        "fifo_uuid": m.fifo_uuid,
        "member_contact": m.member_contact,
        "member_enabled": m.member_enabled,
    })
}

async fn create_fifo(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_add") {
        return (s, j).into_response();
    }
    let Some(name) = s(&payload, &["fifo_name", "name"]) else {
        return unprocessable("name required").into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = fifo::ActiveModel {
        fifo_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        dialplan_uuid: Set(s(&payload, &["dialplan_uuid"])),
        fifo_name: Set(name),
        fifo_extension: Set(s(&payload, &["fifo_extension", "extension"])),
        fifo_agent_queue: Set(s(&payload, &["fifo_agent_queue", "agent_queue"])),
        fifo_agent_status: Set(s(&payload, &["fifo_agent_status", "agent_status"])),
        fifo_enabled: Set(b(&payload, &["fifo_enabled", "enabled"], true)),
        fifo_description: Set(s(&payload, &["fifo_description", "description"])),
        fifo_strategy: Set(s(&payload, &["fifo_strategy", "strategy"])),
        fifo_timeout_seconds: Set(s(&payload, &["fifo_timeout_seconds", "timeout_seconds"])),
        fifo_exit_key: Set(s(&payload, &["fifo_exit_key", "exit_key"])),
        fifo_exit_action: Set(s(&payload, &["fifo_exit_action", "exit_action"])),
        fifo_music: Set(s(&payload, &["fifo_music", "music"])),
        fifo_order: Set(payload.get("fifo_order").or_else(|| payload.get("order")).and_then(|v| v.as_i64()).map(|v| v as i32)),
        fifo_caller_id_name: Set(s(&payload, &["fifo_caller_id_name"])),
        fifo_caller_id_number: Set(s(&payload, &["fifo_caller_id_number"])),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_fifo(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_fifo(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "fifo_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = fifo::Entity::find();
    if !show_all {
        sel = sel.filter(fifo::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_fifo).collect()).into_response()
}

async fn scoped_fifo(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    id: &str,
) -> Option<fifo::Model> {
    match fifo::Entity::find_by_id(id.to_owned())
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => Some(r),
        _ => None,
    }
}

async fn get_fifo(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_fifo(db, domain, &id).await {
        Some(r) => (StatusCode::OK, Json(ser_fifo(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_fifo(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let Some(r) = scoped_fifo(db, domain, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: fifo::ActiveModel = r.into();
    if let Some(v) = s(&payload, &["fifo_name", "name"]) {
        am.fifo_name = Set(v);
    }
    if let Some(v) = s(&payload, &["fifo_extension", "extension"]) {
        am.fifo_extension = Set(Some(v));
    }
    if payload
        .get("fifo_enabled")
        .or_else(|| payload.get("enabled"))
        .is_some()
    {
        am.fifo_enabled = Set(b(&payload, &["fifo_enabled", "enabled"], true));
    }
    for key in ["dialplan_uuid", "fifo_agent_queue", "agent_queue", "fifo_agent_status", "agent_status", "fifo_timeout_seconds", "timeout_seconds", "fifo_exit_key", "exit_key", "fifo_exit_action", "exit_action", "fifo_music", "music", "fifo_strategy", "strategy", "fifo_description", "description", "fifo_caller_id_name", "fifo_caller_id_number"] {
        if let Some(v) = s(&payload, &[key]) {
            match key {
                "dialplan_uuid" => am.dialplan_uuid = Set(Some(v)),
                "fifo_agent_queue" | "agent_queue" => am.fifo_agent_queue = Set(Some(v)),
                "fifo_agent_status" | "agent_status" => am.fifo_agent_status = Set(Some(v)),
                "fifo_timeout_seconds" | "timeout_seconds" => am.fifo_timeout_seconds = Set(Some(v)),
                "fifo_exit_key" | "exit_key" => am.fifo_exit_key = Set(Some(v)),
                "fifo_exit_action" | "exit_action" => am.fifo_exit_action = Set(Some(v)),
                "fifo_music" | "music" => am.fifo_music = Set(Some(v)),
                "fifo_strategy" | "strategy" => am.fifo_strategy = Set(Some(v)),
                "fifo_description" | "description" => am.fifo_description = Set(Some(v)),
                "fifo_caller_id_name" => am.fifo_caller_id_name = Set(Some(v)),
                _ => am.fifo_caller_id_number = Set(Some(v)),
            }
        }
    }
    if let Some(v) = payload.get("fifo_order").or_else(|| payload.get("order")).and_then(|v| v.as_i64()) {
        am.fifo_order = Set(Some(v as i32));
    }
    am.updated_at = Set(crate::routing_db::now());
    match am.update(db).await {
        Ok(r) => (StatusCode::OK, Json(ser_fifo(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_fifo(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match scoped_fifo(db, domain, &id).await {
        Some(r) => {
            let am: fifo::ActiveModel = r.into();
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

async fn create_fifo_member(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_member_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_fifo(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let am = fifo_members::ActiveModel {
        fifo_member_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        fifo_uuid: Set(id.clone()),
        member_contact: Set(s(&payload, &["member_contact", "contact"])),
        member_enabled: Set(b(&payload, &["member_enabled", "enabled"], true)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(m) => (StatusCode::CREATED, Json(ser_fifo_member(&m))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_fifo_members(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_member_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if scoped_fifo(db, domain, &id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    let rows = fifo_members::Entity::find()
        .filter(fifo_members::Column::FifoUuid.eq(id))
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_fifo_member).collect()).into_response()
}

fn fifo_router() -> Router {
    Router::new()
        .route(
            "/api/v1/fifo",
            axum::routing::post(create_fifo).get(list_fifo),
        )
        .route(
            "/api/v1/fifo/{id}",
            axum::routing::get(get_fifo)
                .patch(patch_fifo)
                .delete(delete_fifo),
        )
        .route(
            "/api/v1/fifo/{id}/members",
            axum::routing::post(create_fifo_member).get(list_fifo_members),
        )
}

// ---- handlers: fifo_list status view (Python parity: ESL `fifo list` → DB fallback) ----
async fn list_fifo_status(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = fifo::Entity::find()
        .filter(fifo::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let data: Vec<Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "fifo_name": r.fifo_name,
                "fifo_extension": r.fifo_extension,
                "fifo_enabled": r.fifo_enabled,
                "fifo_description": r.fifo_description,
                "consumer_count": 0,
                "caller_count": 0,
                "status": "DB",
                "fifo_uuid": r.fifo_uuid,
            })
        })
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total, "source": "db"})).into_response()
}

async fn fifo_status_summary(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "fifo_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let total = fifo::Entity::find()
        .filter(fifo::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    Json(serde_json::json!({"data": [], "total": total, "source": "db"})).into_response()
}

async fn list_fifo_callers(
    axum::extract::Path(_fifo): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Live callers come from ESL `fifo list <name>`; the mock transport has
    // no FIFO stream, so the DB-backed API reports an empty caller list.
    Json(serde_json::json!({"data": [], "total": 0}))
}

async fn delete_fifo_caller(
    axum::extract::Path((fifo_name, uuid)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    Json(
        serde_json::json!({"result": "delete queued (ESL offline)", "fifo_name": fifo_name, "uuid": uuid}),
    )
}

fn fifo_list_router() -> Router {
    Router::new()
        .route("/api/v1/fifo_list", axum::routing::get(list_fifo_status))
        .route(
            "/api/v1/fifo_list/status",
            axum::routing::get(fifo_status_summary),
        )
        .route(
            "/api/v1/fifo_list/{fifo}/callers",
            axum::routing::get(list_fifo_callers),
        )
        .route(
            "/api/v1/fifo_list/{fifo}/callers/{uuid}",
            axum::routing::delete(delete_fifo_caller),
        )
}

// ---- handlers: webrtc_settings (legacy CRUD + canonical settings routes) ----
fn ser_webrtc(r: &webrtc_settings::Model) -> Value {
    serde_json::json!({
        "domain_id": r.domain_id.to_string(),
        "webrtc_setting_uuid": r.domain_id.to_string(),
        "enabled": r.enabled,
        "auth_mode": r.auth_mode,
        "wss_url": r.wss_url,
        "sip_domain": r.sip_domain,
        "ice_servers": r.ice_servers,
    })
}

fn default_webrtc(domain: uuid::Uuid) -> Value {
    serde_json::json!({
        "domain_id": domain.to_string(),
        "webrtc_setting_uuid": domain.to_string(),
        "enabled": false,
        "auth_mode": "extension",
        "wss_url": serde_json::Value::Null,
        "sip_domain": serde_json::Value::Null,
        "ice_servers": serde_json::Value::Null,
    })
}

fn valid_auth_mode(v: &str) -> bool {
    matches!(v, "extension" | "webrtc" | "both")
}

fn valid_wss(v: &str) -> bool {
    v.starts_with("wss://") || v.starts_with("ws://")
}

async fn upsert_webrtc(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    payload: &Value,
) -> Result<webrtc_settings::Model, String> {
    let now = crate::routing_db::now();
    match webrtc_settings::Entity::find_by_id(domain).one(db).await {
        Err(e) => Err(format!("lookup failed: {e}")),
        Ok(Some(r)) => {
            let mut am: webrtc_settings::ActiveModel = r.into();
            if payload.get("enabled").is_some() {
                am.enabled = Set(b(payload, &["enabled"], false));
            }
            if let Some(v) = s(payload, &["auth_mode"]) {
                if !valid_auth_mode(&v) {
                    return Err("auth_mode must be extension|webrtc|both".to_owned());
                }
                am.auth_mode = Set(v);
            }
            if let Some(v) = s(payload, &["wss_url"]) {
                if !valid_wss(&v) {
                    return Err("wss_url must start with wss:// or ws://".to_owned());
                }
                am.wss_url = Set(Some(v));
            }
            if let Some(v) = s(payload, &["sip_domain"]) {
                am.sip_domain = Set(Some(v));
            }
            if payload.get("ice_servers").is_some() {
                am.ice_servers = Set(payload.get("ice_servers").cloned());
            }
            am.updated_at = Set(now);
            am.update(db)
                .await
                .map_err(|e| format!("update failed: {e}"))
        }
        Ok(None) => {
            if let Some(v) = s(payload, &["auth_mode"]) {
                if !valid_auth_mode(&v) {
                    return Err("auth_mode must be extension|webrtc|both".to_owned());
                }
            }
            if let Some(v) = s(payload, &["wss_url"]) {
                if !valid_wss(&v) {
                    return Err("wss_url must start with wss:// or ws://".to_owned());
                }
            }
            let am = webrtc_settings::ActiveModel {
                domain_id: Set(domain),
                enabled: Set(b(payload, &["enabled"], false)),
                auth_mode: Set(s(payload, &["auth_mode"]).unwrap_or_else(|| "extension".to_owned())),
                wss_url: Set(s(payload, &["wss_url"])),
                sip_domain: Set(s(payload, &["sip_domain"])),
                ice_servers: Set(payload.get("ice_servers").cloned()),
                created_at: Set(now),
                updated_at: Set(now),
            };
            am.insert(db)
                .await
                .map_err(|e| format!("insert failed: {e}"))
        }
    }
}

async fn create_webrtc(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    match upsert_webrtc(db, domain, &payload).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_webrtc(&r))).into_response(),
        Err(e) if e.contains("auth_mode") || e.contains("wss_url") => {
            unprocessable(&e).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": e})),
        )
            .into_response(),
    }
}

async fn list_webrtc(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match webrtc_settings::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => envelope(vec![ser_webrtc(&r)]),
        None => envelope(vec![default_webrtc(domain)]),
    }
    .into_response()
}

async fn get_webrtc(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if id != domain.to_string() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    match webrtc_settings::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_webrtc(&r))).into_response(),
        None => (StatusCode::OK, Json(default_webrtc(domain))).into_response(),
    }
}

async fn delete_webrtc(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    if id != domain.to_string() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    }
    if let Some(r) = webrtc_settings::Entity::find_by_id(domain)
        .one(db)
        .await
        .unwrap_or(None)
    {
        let am: webrtc_settings::ActiveModel = r.into();
        let _ = am.delete(db).await;
    }
    StatusCode::NO_CONTENT.into_response()
}

async fn get_webrtc_settings(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(domain_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_view") {
        return (s, j).into_response();
    }
    let req_domain: uuid::Uuid = match domain_id.parse() {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"detail": "invalid domain"})),
            )
                .into_response()
        }
    };
    if crate::routing_db::domain(&claims) != req_domain {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"detail": "Domain mismatch"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    match webrtc_settings::Entity::find_by_id(req_domain)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_webrtc(&r))).into_response(),
        None => (StatusCode::OK, Json(default_webrtc(req_domain))).into_response(),
    }
}

async fn put_webrtc_settings(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(domain_id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "webrtc_edit") {
        return (s, j).into_response();
    }
    let req_domain: uuid::Uuid = match domain_id.parse() {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"detail": "invalid domain"})),
            )
                .into_response()
        }
    };
    if crate::routing_db::domain(&claims) != req_domain {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"detail": "Domain mismatch"})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let _ = tenant_domain(db, &claims).await;
    match upsert_webrtc(db, req_domain, &payload).await {
        Ok(r) => (StatusCode::OK, Json(ser_webrtc(&r))).into_response(),
        Err(e) if e.contains("auth_mode") || e.contains("wss_url") => {
            unprocessable(&e).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": e})),
        )
            .into_response(),
    }
}

fn webrtc_router() -> Router {
    Router::new()
        .route(
            "/api/v1/webrtc_settings",
            axum::routing::post(create_webrtc).get(list_webrtc),
        )
        .route(
            "/api/v1/webrtc_settings/{id}",
            axum::routing::get(get_webrtc).delete(delete_webrtc),
        )
        .route(
            "/api/v1/webrtc-settings/domains/{domain_id}/settings",
            axum::routing::get(get_webrtc_settings).put(put_webrtc_settings),
        )
}

// ---- handlers: emergency_logs ----
fn ser_emergency(r: &emergency_logs::Model) -> Value {
    serde_json::json!({
        "emergency_log_uuid": r.emergency_log_uuid,
        "domain_id": r.domain_id.to_string(),
        "extension": r.extension,
        "event": r.event,
    })
}

async fn create_emergency(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "emergency_logs_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let am = emergency_logs::ActiveModel {
        emergency_log_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(domain),
        extension: Set(s(&payload, &["extension"])),
        event: Set(s(&payload, &["event"])),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_emergency(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_emergency(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "emergency_logs_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "emergency_logs_view_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = emergency_logs::Entity::find();
    if !show_all {
        sel = sel.filter(emergency_logs::Column::DomainId.eq(domain));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_emergency).collect()).into_response()
}

async fn get_emergency(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "emergency_logs_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match emergency_logs::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            (StatusCode::OK, Json(ser_emergency(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_emergency(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "emergency_logs_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match emergency_logs::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) if r.domain_id == domain => {
            let am: emergency_logs::ActiveModel = r.into();
            let _ = am.delete(db).await;
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

fn emergency_router() -> Router {
    Router::new()
        .route(
            "/api/v1/emergency_logs",
            axum::routing::post(create_emergency).get(list_emergency),
        )
        .route(
            "/api/v1/emergency_logs/{id}",
            axum::routing::get(get_emergency).delete(delete_emergency),
        )
}

// ---- handlers: event_guard (global, no domain_id — Python parity) ----
fn ser_event_guard(r: &event_guard_logs::Model) -> Value {
    serde_json::json!({
        "event_guard_log_uuid": r.event_guard_log_uuid,
        "hostname": r.hostname,
        "filter": r.filter,
        "ip_address": r.ip_address,
        "extension": r.extension,
        "user_agent": r.user_agent,
        "log_status": r.log_status,
    })
}

async fn create_event_guard(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "event_guard_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let now = crate::routing_db::now();
    let am = event_guard_logs::ActiveModel {
        event_guard_log_uuid: Set(crate::routing_db::new_uuid()),
        hostname: Set(s(&payload, &["hostname"])),
        log_date: Set(now),
        filter: Set(s(&payload, &["filter"])),
        ip_address: Set(s(&payload, &["ip_address"])),
        extension: Set(s(&payload, &["extension"])),
        user_agent: Set(s(&payload, &["user_agent"])),
        log_status: Set(s(&payload, &["log_status", "status"])),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_event_guard(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_event_guard(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "event_guard_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let rows = event_guard_logs::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    envelope(rows.iter().map(ser_event_guard).collect()).into_response()
}

async fn get_event_guard(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "event_guard_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match event_guard_logs::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => (StatusCode::OK, Json(ser_event_guard(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_event_guard(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "event_guard_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    match event_guard_logs::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(r) => {
            let am: event_guard_logs::ActiveModel = r.into();
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

fn event_guard_router() -> Router {
    Router::new()
        .route(
            "/api/v1/event_guard",
            axum::routing::post(create_event_guard).get(list_event_guard),
        )
        .route(
            "/api/v1/event_guard/{id}",
            axum::routing::get(get_event_guard).delete(delete_event_guard),
        )
}

// ---- registry ----
pub fn spillover_c_router() -> Router {
    acl_router()
        .merge(fifo_router())
        .merge(fifo_list_router())
        .merge(webrtc_router())
        .merge(emergency_router())
        .merge(event_guard_router())
}
