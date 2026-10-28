//! Log viewer — mirrors `app/log_viewer/router.py`
//! (`GET /logs`, `GET /logs/freeswitch`).
//!
//! Order: models → schemas → handlers → esl.
//!
//! Prod tails FS logs; here ESL-backed state with mock fallback:
//! `sofia_status` parse, `registrations_raw`, `channels`, and an honest
//! `fs_status.connected` flag.

use axum::{Json, Router};
use pbx_fs::esl::{EsClient, RealEs};
use std::collections::HashMap;

async fn logs_list(
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let lines: usize = q
        .get("lines")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
        .clamp(1, 1000);
    let level = q.get("level").cloned().unwrap_or_default().to_uppercase();
    let f = q.get("q").cloned().unwrap_or_default().to_lowercase();
    let client = RealEs::from_env();
    let mut content: Vec<String> = Vec::new();
    // ESL-derived lines first (sofia state + FS connectivity).
    for (name, state) in client.sofia_status() {
        content.push(format!("[sofia] {name} {state}"));
    }
    match client.try_api("status") {
        Some(r) if r.contains("+OK") => {
            content.push(format!("[FS] connected {}:{}", client.host, client.port));
        }
        _ => content.push(format!("[FS] disconnected {}:{}", client.host, client.port)),
    }
    if content.is_empty() {
        // Dev fallback mock (Python parity: never empty, never 500).
        content.extend(
            [
                "[NOTICE] sofia_reg.c:1234 SIP auth success for 1010 (via xml_curl)",
                "[INFO] switch_core.c: dialplan found for default context",
                "[INFO] sofia.c: registration 1010 expires 3600",
            ]
            .iter()
            .map(|s| s.to_string()),
        );
    }
    if !level.is_empty() {
        content.retain(|l| l.to_uppercase().contains(&level));
    }
    if !f.is_empty() {
        content.retain(|l| l.to_lowercase().contains(&f));
    }
    let total = content.len();
    content.truncate(lines);
    Json(serde_json::json!({
        "lines": content,
        "total": total,
        "source": if total > 3 { "esl" } else { "mock" },
    }))
}

async fn freeswitch_logs() -> Json<serde_json::Value> {
    let client = RealEs::from_env();
    let sofia_raw = client.api("sofia status");
    let sofia_parsed = client.sofia_status();
    let regs = client.api("show registrations");
    let channels = client.api("show channels");
    let (connected, _, error) = match client.try_api("status") {
        Some(r) if r.contains("+OK") => (true, None::<u64>, None::<String>),
        Some(r) => (false, None, Some(r.chars().take(300).collect())),
        None => (
            false,
            None,
            Some(format!(
                "ESL {}:{} unreachable (mock fallback)",
                client.host, client.port
            )),
        ),
    };
    Json(serde_json::json!({
        "sofia_status": sofia_raw.chars().take(4000).collect::<String>(),
        "sofia_parsed": sofia_parsed,
        "registrations_raw": regs.chars().take(4000).collect::<String>(),
        "channels": channels.chars().take(4000).collect::<String>(),
        "fs_status": {
            "connected": connected,
            "host": client.host,
            "port": client.port,
            "error": error,
        },
    }))
}

pub fn logs_router() -> Router {
    Router::new()
        .route("/api/v1/logs", axum::routing::get(logs_list))
        .route("/api/v1/logs/", axum::routing::get(logs_list))
        .route(
            "/api/v1/logs/freeswitch",
            axum::routing::get(freeswitch_logs),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
