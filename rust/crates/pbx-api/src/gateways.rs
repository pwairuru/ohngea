//! Gateways router — mirrors `app/gateways/router.py`.
//!
//! SeaORM backed with per-request tenant txn (FORCE RLS parity).
//! Password masked `***` on GET.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gateway {
    pub gateway_uuid: String,
    pub gateway: String,
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    pub proxy: Option<String>,
    #[serde(default, alias = "gateway_register")]
    pub register: Option<bool>,
    pub profile: String,
    pub context: String,
    pub enabled: bool,
    pub distinct_to: Option<bool>,
    pub auth_username: Option<String>,
    pub realm: Option<String>,
    pub from_user: Option<String>,
    pub from_domain: Option<String>,
    pub register_proxy: Option<String>,
    pub outbound_proxy: Option<String>,
    pub expire_seconds: Option<i32>,
    pub register_transport: Option<String>,
    pub contact_params: Option<String>,
    pub retry_seconds: Option<i32>,
    pub extension: Option<String>,
    pub ping: Option<String>,
    pub ping_min: Option<i32>,
    pub ping_max: Option<i32>,
    pub contact_in_ping: Option<bool>,
    pub caller_id_in_from: Option<bool>,
    pub supress_cng: Option<bool>,
    pub sip_cid_type: Option<String>,
    pub codec_prefs: Option<String>,
    pub channels: Option<i32>,
    pub extension_in_contact: Option<String>,
    pub hostname: Option<String>,
    pub description: Option<String>,
    /// Tenant UUID, or None for a global/shared gateway
    /// (PHP `gateway_domain` parity).
    pub domain_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GatewayCreate {
    gateway: Option<String>,
    name: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    proxy: Option<String>,
    #[serde(default)]
    register: Option<bool>,
    #[serde(default)]
    gateway_register: Option<bool>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    distinct_to: Option<bool>,
    #[serde(default)]
    auth_username: Option<String>,
    #[serde(default)]
    realm: Option<String>,
    #[serde(default)]
    from_user: Option<String>,
    #[serde(default)]
    from_domain: Option<String>,
    #[serde(default)]
    register_proxy: Option<String>,
    #[serde(default)]
    outbound_proxy: Option<String>,
    #[serde(default)]
    expire_seconds: Option<i32>,
    #[serde(default)]
    register_transport: Option<String>,
    #[serde(default)]
    contact_params: Option<String>,
    #[serde(default)]
    retry_seconds: Option<i32>,
    #[serde(default)]
    extension: Option<String>,
    #[serde(default)]
    ping: Option<String>,
    #[serde(default)]
    ping_min: Option<i32>,
    #[serde(default)]
    ping_max: Option<i32>,
    #[serde(default)]
    contact_in_ping: Option<bool>,
    #[serde(default)]
    caller_id_in_from: Option<bool>,
    #[serde(default)]
    supress_cng: Option<bool>,
    #[serde(default)]
    sip_cid_type: Option<String>,
    #[serde(default)]
    codec_prefs: Option<String>,
    #[serde(default)]
    channels: Option<i32>,
    #[serde(default)]
    extension_in_contact: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default)]
    description: Option<String>,
    /// Target tenant: UUID string, empty string (global, needs
    /// `gateway_domain`), or absent (caller's tenant). Without the perm the
    /// value is ignored and the tenant is forced (PHP parity; PHP forms
    /// post "" for an empty domain select). (Plain Option because serde
    /// collapses explicit JSON null into absent for nested Options.)
    #[serde(default)]
    domain_id: Option<String>,
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
        proxy: r.proxy.clone(),
        register: Some(r.gateway_register),
        profile: r
            .profile
            .clone()
            .unwrap_or_else(|| pbx_db::gateways::default_profile().to_owned()),
        context: r
            .context
            .clone()
            .unwrap_or_else(|| pbx_db::gateways::default_context().to_owned()),
        enabled: r.enabled,
        distinct_to: r.distinct_to,
        auth_username: r.auth_username.clone(),
        realm: r.realm.clone(),
        from_user: r.from_user.clone(),
        from_domain: r.from_domain.clone(),
        register_proxy: r.register_proxy.clone(),
        outbound_proxy: r.outbound_proxy.clone(),
        expire_seconds: r.expire_seconds,
        register_transport: r.register_transport.clone(),
        contact_params: r.contact_params.clone(),
        retry_seconds: r.retry_seconds,
        extension: r.extension.clone(),
        ping: r.ping.clone(),
        ping_min: r.ping_min,
        ping_max: r.ping_max,
        contact_in_ping: r.contact_in_ping,
        caller_id_in_from: r.caller_id_in_from,
        supress_cng: r.supress_cng,
        sip_cid_type: r.sip_cid_type.clone(),
        codec_prefs: r.codec_prefs.clone(),
        channels: r.channels,
        extension_in_contact: r.extension_in_contact.clone(),
        hostname: r.hostname.clone(),
        description: r.description.clone(),
        domain_id: r.domain_id.map(|d| d.to_string()),
    }
}

/// FreeSWITCH propagation after gateway writes: per-key cache invalidate
/// + `reloadxml` + `sofia profile {profile} rescan` over real ESL.
/// Deletes additionally `killgw` the removed gateway (lowercase uuid, the
/// FS-side gateway name). ESL failures are LOUD (stderr), never a silent
/// mock fallback; the DB write already succeeded so HTTP still succeeds.
fn propagate_gateway(profile: &str, kill: Option<&str>) {
    crate::fs_xml::invalidate_key("configuration:sofia.conf");
    let mut cmds = vec![
        "reloadxml".to_owned(),
        format!("sofia profile {profile} rescan"),
    ];
    if let Some(gw) = kill {
        cmds.push(format!("sofia profile {profile} killgw {gw}"));
    }
    let refs: Vec<&str> = cmds.iter().map(String::as_str).collect();
    crate::fs_xml::propagate_esl(&refs);
}

async fn create_gateway(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<GatewayCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let tenant_id = crate::routing_db::tenant_domain(db, &claims).await;
    // PHP `gateway_domain` parity: perm holders may target another tenant
    // (UUID string) or NULL (global/shared gateway). Everyone else — and
    // absent values — resolve to the caller's own tenant.
    let domain_id: Option<uuid::Uuid> = match payload.domain_id.as_deref().map(str::trim) {
        Some("") if crate::tenant::has_permission(&claims, "gateway_domain") => None,
        Some(s) if crate::tenant::has_permission(&claims, "gateway_domain") => {
            match s.parse::<uuid::Uuid>() {
                Ok(id) => Some(id),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_id must be a UUID or empty"})),
                    )
                        .into_response();
                }
            }
        }
        _ => Some(tenant_id),
    };
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
        proxy: Set(payload.proxy.clone()),
        gateway_register: Set(payload
            .register
            .or(payload.gateway_register)
            .unwrap_or(true)),
        enabled: Set(payload.enabled.unwrap_or(true)),
        domain_id: Set(domain_id),
        distinct_to: Set(payload.distinct_to),
        auth_username: Set(payload.auth_username.clone()),
        realm: Set(payload.realm.clone()),
        from_user: Set(payload.from_user.clone()),
        from_domain: Set(payload.from_domain.clone()),
        register_proxy: Set(payload.register_proxy.clone()),
        outbound_proxy: Set(payload.outbound_proxy.clone()),
        expire_seconds: Set(payload
            .expire_seconds
            .or(Some(pbx_db::gateways::default_expire_seconds()))),
        register_transport: Set(payload.register_transport.clone()),
        contact_params: Set(payload.contact_params.clone()),
        retry_seconds: Set(payload
            .retry_seconds
            .or(Some(pbx_db::gateways::default_retry_seconds()))),
        extension: Set(payload.extension.clone()),
        ping: Set(payload.ping.clone()),
        ping_min: Set(payload.ping_min),
        ping_max: Set(payload.ping_max),
        contact_in_ping: Set(payload.contact_in_ping),
        caller_id_in_from: Set(payload.caller_id_in_from),
        supress_cng: Set(payload.supress_cng),
        sip_cid_type: Set(payload.sip_cid_type.clone()),
        codec_prefs: Set(payload.codec_prefs.clone()),
        channels: Set(payload.channels),
        extension_in_contact: Set(payload.extension_in_contact.clone()),
        context: Set(Some(context.clone())),
        profile: Set(Some(profile.clone())),
        hostname: Set(payload.hostname.clone()),
        description: Set(payload.description.clone()),
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
        proxy: payload.proxy,
        register: payload.register.or(payload.gateway_register).or(Some(true)),
        profile: profile.clone(),
        context,
        enabled: payload.enabled.unwrap_or(true),
        distinct_to: payload.distinct_to,
        auth_username: payload.auth_username,
        realm: payload.realm,
        from_user: payload.from_user,
        from_domain: payload.from_domain,
        register_proxy: payload.register_proxy,
        outbound_proxy: payload.outbound_proxy,
        expire_seconds: payload
            .expire_seconds
            .or(Some(pbx_db::gateways::default_expire_seconds())),
        register_transport: payload.register_transport,
        contact_params: payload.contact_params,
        retry_seconds: payload
            .retry_seconds
            .or(Some(pbx_db::gateways::default_retry_seconds())),
        extension: payload.extension,
        ping: payload.ping,
        ping_min: payload.ping_min,
        ping_max: payload.ping_max,
        contact_in_ping: payload.contact_in_ping,
        caller_id_in_from: payload.caller_id_in_from,
        supress_cng: payload.supress_cng,
        sip_cid_type: payload.sip_cid_type,
        codec_prefs: payload.codec_prefs,
        channels: payload.channels,
        extension_in_contact: payload.extension_in_contact,
        hostname: payload.hostname,
        description: payload.description,
        domain_id: domain_id.map(|d| d.to_string()),
    };
    propagate_gateway(&profile, None);
    (StatusCode::CREATED, Json(serde_json::json!(masked(rec)))).into_response()
}

async fn list_gateways(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "gateway_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})).into_response(),
    };
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "gateway_all");
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain_id = crate::routing_db::domain(&claims);
    let mut sel = pbx_db::entities::gateways::Entity::find();
    if !show_all {
        // PHP `gateway_domain` parity: holders also see global (NULL) rows.
        if crate::tenant::sees_domain_nulls(&claims, "gateway_domain") {
            sel = sel.filter(
                Condition::any()
                    .add(pbx_db::entities::gateways::Column::DomainId.eq(domain_id))
                    .add(pbx_db::entities::gateways::Column::DomainId.is_null()),
            );
        } else {
            sel = sel.filter(pbx_db::entities::gateways::Column::DomainId.eq(domain_id));
        }
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
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
    let row = {
        let see_global = crate::tenant::sees_domain_nulls(&claims, "gateway_domain");
        find_gateway_row(&txn, domain_id, uid, see_global).await
    };
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
    see_global: bool,
) -> Option<pbx_db::entities::gateways::Model> {
    // Tenant rows are always visible to their tenant; global (NULL-domain)
    // rows only to `gateway_domain` holders (PHP list/edit parity).
    let scope = if see_global {
        Condition::any()
            .add(pbx_db::entities::gateways::Column::DomainId.eq(domain_id))
            .add(pbx_db::entities::gateways::Column::DomainId.is_null())
    } else {
        Condition::any().add(pbx_db::entities::gateways::Column::DomainId.eq(domain_id))
    };
    pbx_db::entities::gateways::Entity::find()
        .filter(
            Condition::all()
                .add(pbx_db::entities::gateways::Column::GatewayUuid.eq(id))
                .add(scope),
        )
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
    let see_global = crate::tenant::sees_domain_nulls(&claims, "gateway_domain");
    let Some(row) = find_gateway_row(&txn, domain_id, id, see_global).await else {
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
    // PHP `gateway_domain` parity: perm holders may re-home a gateway
    // (UUID string) or globalize it (empty string). Without the perm the
    // key is ignored and the row keeps its domain.
    if get("domain_id").or_else(|| get("domain_uuid")).is_some()
        && crate::tenant::has_permission(&claims, "gateway_domain")
    {
        let raw = get("domain_id")
            .or_else(|| get("domain_uuid"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if raw.is_empty() {
            am.domain_id = Set(None);
        } else {
            match raw.parse::<uuid::Uuid>() {
                Ok(id) => am.domain_id = Set(Some(id)),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_id must be a UUID or empty"})),
                    )
                        .into_response();
                }
            }
        }
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
    propagate_gateway(updated.profile.as_deref().unwrap_or("external"), None);
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
    let see_global = crate::tenant::sees_domain_nulls(&claims, "gateway_domain");
    let Some(row) = find_gateway_row(&txn, domain_id, id, see_global).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Gateway not found"})),
        )
            .into_response();
    };
    let profile = row.profile.clone().unwrap_or_else(|| "external".to_owned());
    let gw_name = row.gateway_uuid.to_string().to_lowercase();
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
    propagate_gateway(&profile, Some(&gw_name));
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
