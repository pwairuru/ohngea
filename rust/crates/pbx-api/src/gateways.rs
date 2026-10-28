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
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            );
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
        );
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
    (StatusCode::CREATED, Json(serde_json::json!(masked(rec))))
}

async fn list_gateways(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return Json(serde_json::json!({"data": [], "total": 0})),
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
    Json(serde_json::json!({"data": items, "total": total}))
}

async fn get_gateway(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
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

pub fn gateways_router() -> Router {
    Router::new()
        .route("/api/v1/gateways", post(create_gateway).get(list_gateways))
        .route("/api/v1/gateways/{id}", axum::routing::get(get_gateway))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
