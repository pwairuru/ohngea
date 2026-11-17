//! Vars CRUD + sofia XML regen — mirrors `app/vars/router.py:13-74`
//! and `service.py`. SeaORM backed with per-request tenant txn
//! (FORCE RLS parity). PATCH/DELETE invalidate the FS XML cache and regen
//! sofia XML from DB.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

fn var_to_json(r: &pbx_db::entities::vars::Model) -> serde_json::Value {
    serde_json::json!({
        "var_uuid": r.var_uuid.to_string(),
        "var_category": r.var_category,
        "var_name": r.var_name,
        "var_value": r.var_value,
        "var_enabled": r.var_enabled,
        "var_description": r.var_description,
        "var_order": r.var_order,
        "var_command": r.var_command,
        "var_hostname": r.var_hostname,
    })
}

/// FreeSWITCH propagation after var writes: per-key `vars.xml` + `sofia.conf`
/// invalidates (vars feed sofia settings interpolation) + `reloadxml` over
/// real ESL (loud failures, never a silent mock fallback).
fn propagate_vars() {
    crate::fs_xml::invalidate_key("configuration:vars.xml");
    crate::fs_xml::invalidate_key("configuration:sofia.conf");
    crate::fs_xml::propagate_esl(&["reloadxml"]);
}

async fn create_var(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "var_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let now = crate::routing_db::now();
    let id = uuid::Uuid::now_v7();
    let am = pbx_db::entities::vars::ActiveModel {
        var_uuid: Set(id),
        domain_id: Set(domain_id),
        var_category: Set(payload
            .get("var_category")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_owned()),
        var_name: Set(payload
            .get("var_name")
            .and_then(|v| v.as_str())
            .unwrap_or("var")
            .to_owned()),
        var_value: Set(payload
            .get("var_value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned()),
        var_enabled: Set(payload
            .get("var_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)),
        var_description: Set(payload
            .get("var_description")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        var_order: Set(payload
            .get("var_order")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)),
        var_command: Set(payload
            .get("var_command")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        var_hostname: Set(payload
            .get("var_hostname")
            .and_then(|v| v.as_str())
            .map(str::to_owned)),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let row = match am.insert(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    propagate_vars();
    (StatusCode::CREATED, Json(var_to_json(&row))).into_response()
}

async fn list_vars(AxExtension(claims): AxExtension<pbx_core::auth::Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "var_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let domain_id = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::vars::Entity::find()
        .filter(pbx_db::entities::vars::Column::DomainId.eq(domain_id))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<serde_json::Value> = rows.iter().map(var_to_json).collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total})).into_response()
}

async fn get_var(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "var_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::vars::Entity::find()
        .filter(pbx_db::entities::vars::Column::VarUuid.eq(uid))
        .filter(pbx_db::entities::vars::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    let _ = txn.commit().await;
    match row {
        Some(r) => (StatusCode::OK, Json(var_to_json(&r))).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn patch_var(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(patch): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "var_edit") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let row = pbx_db::entities::vars::Entity::find()
        .filter(pbx_db::entities::vars::Column::VarUuid.eq(uid))
        .filter(pbx_db::entities::vars::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    let Some(r) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::vars::ActiveModel = r.into();
    if let Some(v) = patch.get("var_value").and_then(|v| v.as_str()) {
        am.var_value = Set(v.to_owned());
    }
    if let Some(v) = patch.get("var_enabled").and_then(|v| v.as_bool()) {
        am.var_enabled = Set(v);
    }
    let row = match am.update(&txn).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("update failed: {e}")})),
            )
                .into_response();
        }
    };
    let _ = txn.commit().await;
    propagate_vars();
    (StatusCode::OK, Json(var_to_json(&row))).into_response()
}

async fn delete_var(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "var_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let domain_id = crate::routing_db::domain(&claims);
    let Ok(uid) = id.parse::<uuid::Uuid>() else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response();
    };
    let res = pbx_db::entities::vars::Entity::delete_many()
        .filter(pbx_db::entities::vars::Column::VarUuid.eq(uid))
        .filter(pbx_db::entities::vars::Column::DomainId.eq(domain_id))
        .exec(&txn)
        .await;
    match res {
        Ok(r) if r.rows_affected > 0 => {
            let _ = txn.commit().await;
            propagate_vars();
            StatusCode::NO_CONTENT.into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

pub fn vars_router() -> Router {
    Router::new()
        .route("/api/v1/vars", post(create_var).get(list_vars))
        .route(
            "/api/v1/vars/{id}",
            axum::routing::get(get_var)
                .patch(patch_var)
                .delete(delete_var),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
