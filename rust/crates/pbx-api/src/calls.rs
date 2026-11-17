//! Calls router — ESL originate/kill over live FS with mock fallback.
//!
//! Types → validation → handlers → esl. Uses `RealEs::from_env`:
//! originate threads the tenant domain
//! (`originate user/from@domain &bridge(user/to@domain)` when available,
//! e2e parity); channels queries live FS via `show channels as json` when
//! `try_api` succeeds, else the mock; kill uses `uuid_kill <fs-uuid>` for
//! real ids and the mock transport for `call-…` ids.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Extension as AxExtension, Json, Router,
};
use pbx_fs::esl::{EsClient, RealEs};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OriginateReq {
    from: String,
    to: String,
    /// Optional explicit domain; defaults to the caller's tenant domain
    /// (`domains.domain_name`, falling back to the claims display name).
    #[serde(default)]
    domain: Option<String>,
}

fn client() -> RealEs {
    RealEs::from_env()
}

/// Tenant domain name for originate legs (PHP `$_SESSION['domain_name']`
/// parity): `domains.domain_name` for the caller's tenant, falling back to
/// the claims display name (`default` in tests) when no row exists.
async fn tenant_domain_name(claims: &pbx_core::auth::Claims) -> String {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(claims);
    let fallback = crate::routing_db::tenant_display_name(claims);
    crate::ring_groups::resolve_domain_name(db, domain, &fallback).await
}

async fn originate(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<OriginateReq>,
) -> impl IntoResponse {
    if payload.from.trim().is_empty() || payload.to.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "from/to required"})),
        );
    }
    let explicit = payload.domain.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let domain = match explicit {
        Some(d) => Some(d.to_owned()),
        None => {
            let name = tenant_domain_name(&claims).await;
            (!name.trim().is_empty()).then_some(name)
        }
    };
    let call_id = client().originate_with_domain(
        payload.from.trim(),
        payload.to.trim(),
        domain.as_deref(),
    );
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"call_id": call_id, "from": payload.from, "to": payload.to})),
    )
}

async fn channels() -> impl IntoResponse {
    // Live FS first (`show channels as json`); mock transport when down.
    if let Some(live) = pbx_fs::esl::live_channels(&client()) {
        let data: Vec<serde_json::Value> = live
            .iter()
            .map(|c| serde_json::json!({"call_id": c.uuid, "from": c.from, "to": c.to, "state": c.state}))
            .collect();
        let total = data.len();
        return Json(serde_json::json!({"data": data, "total": total}));
    }
    let chans = pbx_fs::esl::mock_channels();
    let data: Vec<serde_json::Value> = chans
        .iter()
        .map(|c| serde_json::json!({"call_id": c.uuid, "from": c.from, "to": c.to, "state": c.state}))
        .collect();
    let total = data.len();
    Json(serde_json::json!({"data": data, "total": total}))
}

async fn kill(axum::extract::Path(call_id): axum::extract::Path<String>) -> impl IntoResponse {
    if client().kill(&call_id) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "call not found"})),
        )
            .into_response()
    }
}

pub fn calls_router() -> Router {
    Router::new()
        .route("/api/v1/calls/originate", post(originate))
        .route("/api/v1/calls/channels", axum::routing::get(channels))
        .route("/api/v1/calls/{id}", axum::routing::delete(kill))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
