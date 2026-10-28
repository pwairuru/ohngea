//! FreeSWITCH + SIP-profile status — mirrors
//! `app/freeswitch/status_router.py` (`/freeswitch/status`,
//! `/sip_profiles/status`).
//!
//! Order: models → schemas → handlers → esl.
//!
//! esl: `RealEs::from_env` with graceful mock fallback. `connected` is
//! honest: true only when a real ESL round-trip succeeds, false otherwise
//! (15s-cache parity is in-memory process-local here).

use axum::{Extension as AxExtension, Json, Router};
use pbx_fs::esl::{EsClient, RealEs};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

fn esl() -> RealEs {
    RealEs::from_env()
}

/// Honest connectivity probe: real TCP round-trip only.
fn probe_connected() -> (bool, Option<u64>, Option<String>) {
    let client = esl();
    match client.try_api("status") {
        Some(resp) if resp.contains("+OK") => (true, Some(0), None),
        Some(resp) => (false, None, Some(resp.chars().take(300).collect())),
        None => (
            false,
            None,
            Some(format!(
                "ESL {}:{} unreachable (mock fallback)",
                client.host, client.port
            )),
        ),
    }
}

pub(crate) fn fs_status_probe() -> serde_json::Value {
    fs_status_body()
}

fn fs_status_body() -> serde_json::Value {
    let client = esl();
    let (connected, latency_ms, error) = probe_connected();
    serde_json::json!({
        "connected": connected,
        "host": client.host,
        "port": client.port,
        "latency_ms": latency_ms,
        "error": error,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

async fn freeswitch_status() -> Json<serde_json::Value> {
    Json(fs_status_body())
}

async fn sip_profiles_status(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let profiles = pbx_db::entities::sip_profiles::Entity::find()
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();

    let client = esl();
    let (connected, _, err) = probe_connected();
    // Parse sofia map only when FS is really up (mock transport yields no
    // profiles → everything UNKNOWN, matching Python's FS-down branch).
    let sofia_map = if connected {
        client.sofia_status()
    } else {
        std::collections::HashMap::new()
    };

    let data: Vec<serde_json::Value> = profiles
        .iter()
        .map(|p| {
            let state = sofia_map.get(&p.name).cloned().or_else(|| {
                sofia_map.iter().find_map(|(k, v)| {
                    if k.eq_ignore_ascii_case(&p.name) {
                        Some(v.clone())
                    } else {
                        None
                    }
                })
            });
            let fs_state = state.unwrap_or_else(|| "UNKNOWN".to_owned());
            let running = fs_state == "RUNNING";
            serde_json::json!({
                "sip_profile_uuid": p.sip_profile_uuid.to_string(),
                "name": p.name,
                "sip_port": p.sip_port,
                "enabled": p.enabled,
                "fs_state": fs_state,
                "running": running,
            })
        })
        .collect();

    Json(serde_json::json!({
        "connected": connected,
        "error": if connected { None } else { err },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "profiles": data,
    }))
}

pub fn fs_status_router() -> Router {
    Router::new()
        .route(
            "/api/v1/freeswitch/status",
            axum::routing::get(freeswitch_status),
        )
        .route(
            "/api/v1/sip_profiles/status",
            axum::routing::get(sip_profiles_status),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
