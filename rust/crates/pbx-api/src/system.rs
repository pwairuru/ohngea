//! System info/metrics — mirrors `app/system/router.py`
//! (`/system/status`, `/system/info`, `/system/metrics`).
//!
//! Order: models → schemas → handlers.
//!
//! Live data only: hostname/load from std + /proc, process uptime from a
//! boot `Instant`, table counts from SeaORM. No static stubs.

use axum::{Extension as AxExtension, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use std::sync::OnceLock;
use std::time::Instant;

fn boot_instant() -> &'static Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now)
}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_owned())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| std::env::var("HOSTNAME").unwrap_or_else(|_| "pbx".to_owned()))
}

fn load_avg() -> Option<f64> {
    // /proc/loadavg parity with Python os.getloadavg (1-min avg).
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
}

fn mem_kb() -> Option<(u64, u64)> {
    // (total_kb, available_kb) from /proc/meminfo.
    let s = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = None;
    let mut avail = None;
    for line in s.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = v.split_whitespace().next()?.parse().ok();
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail = v.split_whitespace().next()?.parse().ok();
        }
    }
    Some((total?, avail?))
}

async fn metrics_body(claims: &pbx_core::auth::Claims) -> serde_json::Value {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(claims);
    // Concrete per-table counts (sqlite + postgres portable).
    let extensions_n = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let devices_n = pbx_db::entities::devices::Entity::find()
        .filter(pbx_db::entities::devices::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let gateways_n = pbx_db::entities::gateways::Entity::find()
        .filter(pbx_db::entities::gateways::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let sip_n = pbx_db::entities::sip_profiles::Entity::find()
        .filter(pbx_db::entities::sip_profiles::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let cdr_n = pbx_db::entities::cdr::Entity::find()
        .filter(pbx_db::entities::cdr::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let (mem_total, mem_avail) = mem_kb().unwrap_or((0, 0));
    serde_json::json!({
        "hostname": hostname(),
        "load_avg": load_avg(),
        "uptime_secs": boot_instant().elapsed().as_secs(),
        "memory_kb": {"total": mem_total, "available": mem_avail},
        "tables": {
            "extensions": extensions_n,
            "devices": devices_n,
            "gateways": gateways_n,
            "sip_profiles": sip_n,
            "cdr": cdr_n,
        },
    })
}

async fn system_metrics(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let mut body = metrics_body(&claims).await;
    body["_links"] = serde_json::json!({
        "self": {"href": "/api/v1/system/metrics"},
        "info": {"href": "/api/v1/system/info"},
    });
    Json(body)
}

async fn system_info(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let mut body = metrics_body(&claims).await;
    body["version"] = serde_json::json!(env!("CARGO_PKG_VERSION"));
    body["crates"] = serde_json::json!({
        "pbx-api": env!("CARGO_PKG_VERSION"),
        "pbx-db": "0.1.0",
        "pbx-fs": "0.1.0",
        "pbx-core": "0.1.0",
    });
    body["os"] = serde_json::json!(std::env::consts::OS);
    body["arch"] = serde_json::json!(std::env::consts::ARCH);
    body["_links"] = serde_json::json!({
        "self": {"href": "/api/v1/system/info"},
        "metrics": {"href": "/api/v1/system/metrics"},
        "status": {"href": "/api/v1/system/status"},
    });
    Json(body)
}

async fn system_status(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    // Reuse the honest FS probe shape (connected flag, never stubbed "ok").
    let fs = crate::fs_status_probe();
    let metrics = metrics_body(&claims).await;
    Json(serde_json::json!({
        "freeswitch": fs,
        "system": {
            "hostname": metrics.get("hostname"),
            "load_avg": metrics.get("load_avg"),
            "uptime_secs": metrics.get("uptime_secs"),
            "memory_kb": metrics.get("memory_kb"),
        },
        "tables": metrics.get("tables"),
        "_links": {
            "self": {"href": "/api/v1/system/status"},
            "info": {"href": "/api/v1/system/info"},
            "metrics": {"href": "/api/v1/system/metrics"},
        },
    }))
}

pub fn system_router() -> Router {
    Router::new()
        .route("/api/v1/system/status", axum::routing::get(system_status))
        .route("/api/v1/system/info", axum::routing::get(system_info))
        .route("/api/v1/system/metrics", axum::routing::get(system_metrics))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
