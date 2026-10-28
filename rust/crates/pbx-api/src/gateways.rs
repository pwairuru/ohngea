//! Gateways router — mirrors `app/gateways/router.py`.
//!
//! SeaORM backed with per-request tenant txn (FORCE RLS parity).
//! Password masked `***` on GET.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gateway {
    pub gateway_uuid: String,
    pub gateway: String,
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    pub profile: String,
    pub context: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
struct GatewayCreate {
    gateway: Option<String>,
    name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    profile: Option<String>,
    context: Option<String>,
}

fn masked(mut g: Gateway) -> Gateway {
    if g.password.as_deref().is_some_and(|p| !p.is_empty()) {
        g.password = Some("***".to_owned());
    }
    g
}

fn row_to_gateway(r: &pbx_db::entities::gateways::Model) -> Gateway {
    Gateway {
        gateway_uuid: r.gateway_uuid.to_string(),
        gateway: r.gateway.clone(),
        username: r.username.clone(),
        password: r.password.clone(),
        profile: r
            .profile
            .clone()
            .unwrap_or_else(|| pbx_db::gateways::default_profile().to_owned()),
        context: r
            .context
            .clone()
            .unwrap_or_else(|| pbx_db::gateways::default_context().to_owned()),
        enabled: r.enabled,
    }
}

async fn create_gateway(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<GatewayCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_add") {
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
    let profile = payload
        .profile
        .unwrap_or_else(|| pbx_db::gateways::default_profile().to_owned());
    let context = payload
        .context
        .unwrap_or_else(|| pbx_db::gateways::default_context().to_owned());
    let name = payload
        .gateway
        .or(payload.name)
        .unwrap_or_else(|| "gw".to_owned());
    let am = pbx_db::entities::gateways::ActiveModel {
        gateway_uuid: Set(id),
        gateway: Set(name.clone()),
        username: Set(payload.username.clone()),
        password: Set(payload.password.clone()),
        proxy: Set(None),
        gateway_register: Set(true),
        enabled: Set(true),
        domain_id: Set(domain_id),
        distinct_to: Set(None),
        auth_username: Set(None),
        realm: Set(None),
        from_user: Set(None),
        from_domain: Set(None),
        register_proxy: Set(None),
        outbound_proxy: Set(None),
        expire_seconds: Set(Some(pbx_db::gateways::default_expire_seconds())),
        register_transport: Set(None),
        contact_params: Set(None),
        retry_seconds: Set(Some(pbx_db::gateways::default_retry_seconds())),
        extension: Set(None),
        ping: Set(None),
        ping_min: Set(None),
        ping_max: Set(None),
        contact_in_ping: Set(None),
        caller_id_in_from: Set(None),
        supress_cng: Set(None),
        sip_cid_type: Set(None),
        codec_prefs: Set(None),
        channels: Set(None),
        extension_in_contact: Set(None),
        context: Set(Some(context.clone())),
        profile: Set(Some(profile.clone())),
        hostname: Set(None),
        description: Set(None),
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
    let _ = txn.commit().await;
    let rec = Gateway {
        gateway_uuid: id.to_string(),
        gateway: name,
        username: payload.username,
        password: payload.password,
        profile: profile.clone(),
        context,
        enabled: true,
    };
    let _rescan_reply = pbx_fs::esl::EsClient::sofia_rescan(&pbx_fs::esl::MockEsClient, &profile);
    (StatusCode::CREATED, Json(serde_json::json!(masked(rec)))).into_response()
}

async fn list_gateways(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let domain_id = crate::routing_db::domain(&claims);
    let rows = pbx_db::entities::gateways::Entity::find()
        .filter(pbx_db::entities::gateways::Column::DomainId.eq(domain_id))
        .all(&txn)
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<Gateway> = rows.iter().map(row_to_gateway).map(masked).collect();
    let total = items.len();
    Json(serde_json::json!({"data": items, "total": total})).into_response()
}

async fn get_gateway(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_view") {
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
    let row = pbx_db::entities::gateways::Entity::find()
        .filter(pbx_db::entities::gateways::Column::GatewayUuid.eq(uid))
        .filter(pbx_db::entities::gateways::Column::DomainId.eq(domain_id))
        .one(&txn)
        .await
        .unwrap_or(None);
    let _ = txn.commit().await;
    match row {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!(masked(row_to_gateway(&r)))),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn find_gateway_row(
    txn: &sea_orm::DatabaseTransaction,
    domain_id: uuid::Uuid,
    id: uuid::Uuid,
) -> Option<pbx_db::entities::gateways::Model> {
    pbx_db::entities::gateways::Entity::find()
        .filter(pbx_db::entities::gateways::Column::GatewayUuid.eq(id))
        .filter(pbx_db::entities::gateways::Column::DomainId.eq(domain_id))
        .one(txn)
        .await
        .unwrap_or(None)
}

/// PATCH /api/v1/gateways/{id} — parity with Python `patch_gateway`: any
/// table column except keys may be set; `register` maps to `gateway_register`.
async fn patch_gateway(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_edit") {
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
    let Some(row) = find_gateway_row(&txn, domain_id, id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Gateway not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::gateways::ActiveModel = row.into();
    let get = |k: &str| payload.get(k);
    if let Some(s) = get("gateway").and_then(|v| v.as_str()) {
        am.gateway = Set(s.to_owned());
    }
    if let Some(v) = get("username") {
        if v.is_null() {
            am.username = Set(None);
        } else if let Some(s) = v.as_str() {
            am.username = Set(Some(s.to_owned()));
        }
    }
    if let Some(v) = get("password") {
        if v.is_null() {
            am.password = Set(None);
        } else if let Some(s) = v.as_str() {
            am.password = Set(Some(s.to_owned()));
        }
    }
    if let Some(v) = get("proxy") {
        if v.is_null() {
            am.proxy = Set(None);
        } else if let Some(s) = v.as_str() {
            am.proxy = Set(Some(s.to_owned()));
        }
    }
    if let Some(b) = get("register").and_then(|v| v.as_bool()) {
        am.gateway_register = Set(b);
    }
    if let Some(b) = get("gateway_register").and_then(|v| v.as_bool()) {
        am.gateway_register = Set(b);
    }
    if let Some(b) = get("enabled").and_then(|v| v.as_bool()) {
        am.enabled = Set(b);
    }
    for key in [
        "distinct_to",
        "contact_in_ping",
        "caller_id_in_from",
        "supress_cng",
    ] {
        if let Some(b) = get(key).and_then(|v| v.as_bool()) {
            match key {
                "distinct_to" => am.distinct_to = Set(Some(b)),
                "contact_in_ping" => am.contact_in_ping = Set(Some(b)),
                "caller_id_in_from" => am.caller_id_in_from = Set(Some(b)),
                _ => am.supress_cng = Set(Some(b)),
            }
        }
    }
    for key in [
        "auth_username",
        "realm",
        "from_user",
        "from_domain",
        "register_proxy",
        "outbound_proxy",
        "register_transport",
        "contact_params",
        "extension",
        "ping",
        "sip_cid_type",
        "codec_prefs",
        "extension_in_contact",
        "context",
        "profile",
        "hostname",
        "description",
    ] {
        if let Some(v) = get(key) {
            let val = v.as_str().map(|s| s.to_owned());
            match key {
                "auth_username" => am.auth_username = Set(val),
                "realm" => am.realm = Set(val),
                "from_user" => am.from_user = Set(val),
                "from_domain" => am.from_domain = Set(val),
                "register_proxy" => am.register_proxy = Set(val),
                "outbound_proxy" => am.outbound_proxy = Set(val),
                "register_transport" => am.register_transport = Set(val),
                "contact_params" => am.contact_params = Set(val),
                "extension" => am.extension = Set(val),
                "ping" => am.ping = Set(val),
                "sip_cid_type" => am.sip_cid_type = Set(val),
                "codec_prefs" => am.codec_prefs = Set(val),
                "extension_in_contact" => am.extension_in_contact = Set(val),
                "context" => am.context = Set(val),
                "profile" => am.profile = Set(val),
                "hostname" => am.hostname = Set(val),
                _ => am.description = Set(val),
            }
        }
    }
    for key in [
        "expire_seconds",
        "retry_seconds",
        "ping_min",
        "ping_max",
        "channels",
    ] {
        if let Some(v) = get(key) {
            let val = v.as_i64().map(|n| n as i32);
            match key {
                "expire_seconds" => am.expire_seconds = Set(val),
                "retry_seconds" => am.retry_seconds = Set(val),
                "ping_min" => am.ping_min = Set(val),
                "ping_max" => am.ping_max = Set(val),
                _ => am.channels = Set(val),
            }
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    use sea_orm::ActiveModelTrait;
    let updated = match am.update(&txn).await {
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
    let _rescan_reply = pbx_fs::esl::EsClient::sofia_rescan(
        &pbx_fs::esl::MockEsClient,
        updated.profile.as_deref().unwrap_or("external"),
    );
    (
        StatusCode::OK,
        Json(serde_json::json!(masked(row_to_gateway(&updated)))),
    )
        .into_response()
}

/// DELETE /api/v1/gateways/{id} → 204 (parity with Python `delete_gateway`).
async fn delete_gateway(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_delete") {
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
    let Some(row) = find_gateway_row(&txn, domain_id, id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Gateway not found"})),
        )
            .into_response();
    };
    let profile = row.profile.clone().unwrap_or_else(|| "external".to_owned());
    let am: pbx_db::entities::gateways::ActiveModel = row.into();
    use sea_orm::ActiveModelTrait;
    if let Err(e) = am.delete(&txn).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
        )
            .into_response();
    }
    let _ = txn.commit().await;
    let _rescan_reply = pbx_fs::esl::EsClient::sofia_rescan(&pbx_fs::esl::MockEsClient, &profile);
    StatusCode::NO_CONTENT.into_response()
}

pub fn gateways_router() -> Router {
    Router::new()
        .route("/api/v1/gateways", post(create_gateway).get(list_gateways))
        .route(
            "/api/v1/gateways/{id}",
            axum::routing::get(get_gateway)
                .patch(patch_gateway)
                .delete(delete_gateway),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
