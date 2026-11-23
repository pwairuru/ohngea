//! Dialplans router — mirrors `app/dialplans/router.py`.
//!
//! SeaORM backed with per-request tenant txn (`BEGIN` + `SET LOCAL
//! app.domain_uuid`, FORCE RLS parity). Legacy `domain` string is carried
//! in `hostname` (compat shim); prod scoping uses `domain_id`.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialplan {
    pub dialplan_uuid: String,
    pub name: String,
    pub context: String,
    pub domain: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub dialplan_continue: bool,
    pub dialplan_order: Option<i32>,
    pub hostname: Option<String>,
    pub app_uuid: Option<String>,
    pub dialplan_number: Option<String>,
    pub dialplan_destination: Option<bool>,
    pub dialplan_xml: Option<String>,
    #[serde(default)]
    pub details: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DialplanCreate {
    #[serde(default)] name: Option<String>,
    #[serde(default)] context: Option<String>,
    #[serde(default)] domain: Option<String>,
    #[serde(default)] enabled: Option<bool>,
    #[serde(default)] description: Option<String>,
    #[serde(default)] dialplan_continue: Option<bool>,
    #[serde(default)] dialplan_order: Option<i32>,
    #[serde(default)] hostname: Option<String>,
    #[serde(default)] app_uuid: Option<String>,
    #[serde(default)] dialplan_number: Option<String>,
    #[serde(default)] dialplan_destination: Option<bool>,
    #[serde(default)] dialplan_xml: Option<String>,
    #[serde(default)] details: Option<Vec<serde_json::Value>>,
}

fn ser_detail(r: &pbx_db::entities::dialplan_details::Model) -> serde_json::Value {
    serde_json::json!({
        "tag": r.tag, "type": r.detail_type, "data": r.data, "order": r.order,
        "dialplan_detail_break": r.dialplan_detail_break,
        "dialplan_detail_inline": r.dialplan_detail_inline,
        "dialplan_detail_group": r.dialplan_detail_group,
        "detail_enabled": r.detail_enabled,
    })
}

/// Time-condition app id (PHP `time_condition_edit.php` writes dialplans
/// with this app_uuid so the dialplan list can attribute them).
pub(crate) const TC_APP_UUID: &str = "4b821450-926b-175a-af93-a03c441818b1";

/// Maintain the backing v_dialplans row for a time condition (PHP parity:
/// name/number/order/context/enabled/description + destination condition,
/// timezone action, provided details, true/false transfers). Returns the
/// dialplan uuid; `existing` reuses the linked row on update.
pub(crate) async fn sync_tc_dialplan(
    db: &sea_orm::DatabaseConnection,
    domain_id: uuid::Uuid,
    name: &str,
    number: &str,
    order: Option<i32>,
    context: Option<String>,
    enabled: bool,
    description: Option<String>,
    timezone: Option<String>,
    true_dest: Option<String>,
    false_dest: Option<String>,
    details: Option<&serde_json::Value>,
    existing: Option<uuid::Uuid>,
) -> uuid::Uuid {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    let now = crate::routing_db::now();
    let id = match existing {
        Some(u) => u,
        None => uuid::Uuid::now_v7(),
    };
    let ctx = context.unwrap_or_else(|| "public".to_owned());
    let am = pbx_db::entities::dialplans::ActiveModel {
        dialplan_uuid: Set(id),
        name: Set(name.to_owned()),
        context: Set(ctx.clone()),
        enabled: Set(enabled),
        domain_id: Set(domain_id),
        description: Set(description),
        dialplan_continue: Set(false),
        dialplan_order: Set(order.or(Some(100))),
        hostname: Set(None),
        app_uuid: Set(Some(TC_APP_UUID.to_owned())),
        dialplan_number: Set(Some(number.to_owned())),
        dialplan_destination: Set(None),
        dialplan_xml: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    // Upsert: try update, insert when missing.
    let exists = pbx_db::entities::dialplans::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();
    if exists {
        let mut am_u: pbx_db::entities::dialplans::ActiveModel = pbx_db::entities::dialplans::Entity::find_by_id(id).one(db).await.unwrap_or(None).unwrap().into();
        am_u.name = Set(name.to_owned());
        am_u.context = Set(ctx.clone());
        am_u.enabled = Set(enabled);
        am_u.dialplan_order = Set(order.or(Some(100)));
        am_u.updated_at = Set(now);
        let _ = am_u.update(db).await;
    } else if am.insert(db).await.is_err() {
        return id;
    }
    // Replace details (PHP deletes + rebuilds on update).
    let _ = pbx_db::entities::dialplan_details::Entity::delete_many()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(id))
        .exec(db)
        .await;
    let mut seed: Vec<serde_json::Value> = vec![
        serde_json::json!({"tag": "condition", "type": "destination_number", "data": format!("^{number}$"), "order": 10, "group": 0}),
    ];
    if let Some(tz) = timezone.filter(|s| !s.trim().is_empty()) {
        seed.push(serde_json::json!({"tag": "action", "type": "set", "data": format!("timezone={tz}"), "inline": "true", "order": 20, "group": 0}));
    }
    if let Some(arr) = details.and_then(|v| v.as_array()) {
        seed.extend(arr.clone());
    } else {
        if let Some(t) = true_dest.filter(|s| !s.trim().is_empty()) {
            seed.push(serde_json::json!({"tag": "action", "type": "transfer", "data": format!("{t} XML {ctx}"), "order": 30, "group": 1}));
        }
        if let Some(f) = false_dest.filter(|s| !s.trim().is_empty()) {
            seed.push(serde_json::json!({"tag": "action", "type": "transfer", "data": format!("{f} XML {ctx}"), "order": 40, "group": 1}));
        }
    }
    insert_details_txn(db, id, domain_id, &seed).await;
    id
}

async fn insert_details_txn<C>(
    db: &C,
    dialplan_id: uuid::Uuid,
    domain_id: uuid::Uuid,
    details: &[serde_json::Value],
) where
    C: sea_orm::ConnectionTrait,
{
    use sea_orm::ActiveModelTrait;
    for (idx, det) in details.iter().enumerate() {
        let s = |k: &str| det.get(k).and_then(|v| v.as_str()).map(str::to_owned);
        let am = pbx_db::entities::dialplan_details::ActiveModel {
            detail_uuid: Set(uuid::Uuid::now_v7()),
            dialplan_uuid: Set(dialplan_id),
            tag: Set(s("tag").or_else(|| s("dialplan_detail_tag")).unwrap_or_else(|| "condition".to_owned())),
            detail_type: Set(s("type").or_else(|| s("dialplan_detail_type")).unwrap_or_default()),
            data: Set(s("data").or_else(|| s("dialplan_detail_data")).unwrap_or_default()),
            order: Set(det.get("order").or_else(|| det.get("dialplan_detail_order")).and_then(|v| v.as_i64()).map(|n| n as i32).unwrap_or(idx as i32)),
            domain_id: Set(det.get("domain_id").and_then(|v| v.as_str()).and_then(|x| x.parse::<uuid::Uuid>().ok()).or(Some(domain_id))),
            dialplan_detail_break: Set(s("dialplan_detail_break").or_else(|| s("break"))),
            dialplan_detail_inline: Set(s("dialplan_detail_inline").or_else(|| s("inline"))),
            dialplan_detail_group: Set(det.get("dialplan_detail_group").or_else(|| det.get("group")).and_then(|v| v.as_i64()).map(|n| n as i32).or(Some((idx / 10) as i32))),
            detail_enabled: Set(det.get("detail_enabled").or_else(|| det.get("dialplan_detail_enabled")).and_then(|v| v.as_bool()).or(Some(true))),
            created_at: Set(crate::routing_db::now()),
            updated_at: Set(crate::routing_db::now()),
        };
        let _ = am.insert(db).await;
    }
}

pub(crate) async fn delete_tc_dialplan(db: &sea_orm::DatabaseConnection, id: uuid::Uuid) {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let _ = pbx_db::entities::dialplan_details::Entity::delete_many()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(id))
        .exec(db)
        .await;
    if let Ok(Some(row)) = pbx_db::entities::dialplans::Entity::find_by_id(id).one(db).await {
        let am: pbx_db::entities::dialplans::ActiveModel = row.into();
        use sea_orm::ActiveModelTrait;
        let _ = am.delete(db).await;
    }
}

fn row_to_dialplan(
    r: &pbx_db::entities::dialplans::Model,
    fallback: &str,
    details: Vec<serde_json::Value>,
) -> Dialplan {
    Dialplan {
        dialplan_uuid: r.dialplan_uuid.to_string(),
        name: r.name.clone(),
        context: r.context.clone(),
        domain: r.hostname.clone().unwrap_or_else(|| fallback.to_owned()),
        enabled: r.enabled,
        description: r.description.clone(),
        dialplan_continue: r.dialplan_continue,
        dialplan_order: r.dialplan_order,
        hostname: r.hostname.clone(),
        app_uuid: r.app_uuid.clone(),
        dialplan_number: r.dialplan_number.clone(),
        dialplan_destination: r.dialplan_destination,
        dialplan_xml: r.dialplan_xml.clone(),
        details,
    }
}

/// Insert `details[]` rows for a dialplan (Python `create_dialplan` parity:
/// alias-tolerant, per-index defaults).
async fn insert_details(
    txn: &sea_orm::DatabaseTransaction,
    dialplan_id: uuid::Uuid,
    domain_id: uuid::Uuid,
    details: &[serde_json::Value],
) {
    insert_details_txn(txn, dialplan_id, domain_id, details).await;
}

async fn details_for(
    txn: &sea_orm::DatabaseTransaction,
    dialplan_id: uuid::Uuid,
) -> Vec<serde_json::Value> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let rows = pbx_db::entities::dialplan_details::Entity::find()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(dialplan_id))
        .order_by(pbx_db::entities::dialplan_details::Column::DialplanDetailGroup, sea_orm::Order::Asc)
        .order_by(pbx_db::entities::dialplan_details::Column::Order, sea_orm::Order::Asc)
        .all(txn)
        .await
        .unwrap_or_default();
    rows.iter().map(ser_detail).collect()
}

/// Best-effort FreeSWITCH propagation after dialplan writes (PHP regenerates
/// dialplan XML + `reloadxml`).
fn propagate_dialplan() {
    crate::fs_xml::invalidate_cache();
    let esl = pbx_fs::esl::RealEs::from_env();
    let _ = esl.try_api("reloadxml");
}

async fn create_dialplan(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<DialplanCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["dialplan_add", "inbound_route_add", "outbound_route_add"]) {
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
    let name = payload.name.unwrap_or_else(|| "local".to_owned());
    let context = payload.context.unwrap_or_else(|| "default".to_owned());
    let domain_str = payload.domain.unwrap_or_else(|| claims.domain_id.clone());
    let enabled = payload.enabled.unwrap_or(true);
    let am = pbx_db::entities::dialplans::ActiveModel {
        dialplan_uuid: Set(id),
        name: Set(name.clone()),
        context: Set(context.clone()),
        enabled: Set(enabled),
        domain_id: Set(domain_id),
        description: Set(payload.description.clone()),
        dialplan_continue: Set(payload.dialplan_continue.unwrap_or(false)),
        dialplan_order: Set(payload.dialplan_order),
        hostname: Set(Some(payload.hostname.clone().unwrap_or_else(|| domain_str.clone()))),
        app_uuid: Set(payload.app_uuid.clone()),
        dialplan_number: Set(payload.dialplan_number.clone()),
        dialplan_destination: Set(payload.dialplan_destination),
        dialplan_xml: Set(payload.dialplan_xml.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(&txn).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response();
    }
    if let Some(ref details) = payload.details {
        insert_details(&txn, id, domain_id, details).await;
    }
    let det = details_for(&txn, id).await;
    let _ = txn.commit().await;
    propagate_dialplan();
    let db = crate::routing_db::conn().await;
    let rec = pbx_db::entities::dialplans::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|r| row_to_dialplan(&r, &domain_str, det.clone()))
        .unwrap_or(Dialplan {
            dialplan_uuid: id.to_string(),
            name,
            context,
            domain: domain_str.clone(),
            enabled,
            description: payload.description.clone(),
            dialplan_continue: payload.dialplan_continue.unwrap_or(false),
            dialplan_order: payload.dialplan_order,
            hostname: Some(payload.hostname.clone().unwrap_or(domain_str)),
            app_uuid: payload.app_uuid.clone(),
            dialplan_number: payload.dialplan_number.clone(),
            dialplan_destination: payload.dialplan_destination,
            dialplan_xml: payload.dialplan_xml.clone(),
            details: det,
        });
    (StatusCode::CREATED, Json(serde_json::json!(rec))).into_response()
}

async fn list_dialplans(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["dialplan_view", "inbound_route_view", "outbound_route_view"]) {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "dialplan_all");
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain_id = crate::routing_db::domain(&claims);
    let mut sel = pbx_db::entities::dialplans::Entity::find();
    if !show_all {
        sel = sel.filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    // Batch details (Python parity: one query, grouped per dialplan).
    let mut dsel = pbx_db::entities::dialplan_details::Entity::find();
    if !show_all {
        dsel = dsel.filter(pbx_db::entities::dialplan_details::Column::DomainId.eq(domain_id));
    }
    let all_det = dsel.all(&txn).await.unwrap_or_default();
    use std::collections::HashMap;
    let mut by_parent: HashMap<uuid::Uuid, Vec<serde_json::Value>> = HashMap::new();
    for d in &all_det {
        by_parent.entry(d.dialplan_uuid).or_default().push(ser_detail(d));
    }
    let _ = txn.commit().await;
    let items: Vec<Dialplan> = rows
        .into_iter()
        .map(|r| {
            let det = by_parent.remove(&r.dialplan_uuid).unwrap_or_default();
            row_to_dialplan(&r, &claims.domain_id, det)
        })
        .collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total})).into_response()
}

async fn get_dialplan(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["dialplan_view", "inbound_route_view", "outbound_route_view"]) {
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
    let row = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DialplanUuid.eq(uid))
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    let det = details_for(&txn, uid).await;
    let _ = txn.commit().await;
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!(row_to_dialplan(&r, &claims.domain_id, det))),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

pub fn dialplans_router() -> Router {
    Router::new()
        .route(
            "/api/v1/dialplans",
            post(create_dialplan).get(list_dialplans),
        )
        .route(
            "/api/v1/dialplans/{id}",
            axum::routing::get(get_dialplan)
                .patch(patch_dialplan)
                .delete(delete_dialplan),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

async fn patch_dialplan(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["dialplan_edit", "inbound_route_edit", "outbound_route_edit"]) {
        return (s, j).into_response();
    }
    use sea_orm::ActiveModelTrait;
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
    let row = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DialplanUuid.eq(uid))
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
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
    let mut am: pbx_db::entities::dialplans::ActiveModel = r.into();
    if let Some(v) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = sea_orm::Set(v.to_owned());
    }
    if let Some(v) = payload.get("description") {
        if v.is_null() {
            am.description = sea_orm::Set(None);
        } else if let Some(s) = v.as_str() {
            am.description = sea_orm::Set(Some(s.to_owned()));
        }
    }
    if let Some(v) = payload.get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = sea_orm::Set(v);
    }
    if let Some(v) = payload.get("context").and_then(|v| v.as_str()) {
        am.context = sea_orm::Set(v.to_owned());
    }
    if let Some(v) = payload.get("dialplan_continue").and_then(|v| v.as_bool()) {
        am.dialplan_continue = sea_orm::Set(v);
    }
    if let Some(v) = payload.get("dialplan_order") {
        if v.is_null() {
            am.dialplan_order = sea_orm::Set(None);
        } else if let Some(n) = v.as_i64() {
            am.dialplan_order = sea_orm::Set(Some(n as i32));
        }
    }
    for key in ["hostname", "app_uuid", "dialplan_number", "dialplan_xml"] {
        if let Some(v) = payload.get(key) {
            let slot: &mut sea_orm::ActiveValue<Option<String>> = match key {
                "hostname" => &mut am.hostname,
                "app_uuid" => &mut am.app_uuid,
                "dialplan_number" => &mut am.dialplan_number,
                _ => &mut am.dialplan_xml,
            };
            if v.is_null() {
                *slot = sea_orm::Set(None);
            } else if let Some(s) = v.as_str() {
                *slot = sea_orm::Set(Some(s.to_owned()));
            }
        }
    }
    if let Some(v) = payload.get("dialplan_destination") {
        if v.is_null() {
            am.dialplan_destination = sea_orm::Set(None);
        } else if let Some(b) = v.as_bool() {
            am.dialplan_destination = sea_orm::Set(Some(b));
        }
    }
    am.updated_at = sea_orm::Set(crate::routing_db::now());
    // `details[]` present → replace rule set (PHP dialplan_edit parity).
    let replace_details = payload.get("details").and_then(|v| v.as_array()).cloned();
    match am.update(&txn).await {
        Ok(r) => {
            if let Some(details) = replace_details {
                use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
                let _ = pbx_db::entities::dialplan_details::Entity::delete_many()
                    .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(r.dialplan_uuid))
                    .exec(&txn)
                    .await;
                let did = r.dialplan_uuid;
                insert_details(&txn, did, domain_id, &details).await;
            }
            let det = details_for(&txn, r.dialplan_uuid).await;
            let _ = txn.commit().await;
            propagate_dialplan();
            (
                StatusCode::OK,
                Json(serde_json::json!(row_to_dialplan(&r, &claims.domain_id, det))),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("update failed: {e}")})),
        )
            .into_response(),
    }
}

async fn delete_dialplan(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing_any(&claims, &["dialplan_delete", "inbound_route_delete", "outbound_route_delete"]) {
        return (s, j).into_response();
    }
    use sea_orm::ActiveModelTrait;
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
    let row = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DialplanUuid.eq(uid))
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
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
    let am: pbx_db::entities::dialplans::ActiveModel = r.into();
    let doomed = uid;
    match am.delete(&txn).await {
        Ok(_) => {
            let _ = pbx_db::entities::dialplan_details::Entity::delete_many()
                .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.eq(doomed))
                .exec(&txn)
                .await;
            let _ = txn.commit().await;
            propagate_dialplan();
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response(),
    }
}
