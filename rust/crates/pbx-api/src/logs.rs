//! Log viewer — mirrors `app/log_viewer/router.py`
//! (`GET /logs`, `GET /logs/freeswitch`).
//!
//! Order: models → schemas → handlers → esl.
//!
//! Prod tails FS logs; here ESL-backed state with mock fallback:
//! `sofia_status` parse, `registrations_raw`, `channels`, and an honest
//! `fs_status.connected` flag.

use axum::{http::StatusCode, response::IntoResponse, Extension, Json, Router};
use pbx_fs::esl::{
    is_connected_body, parse_sofia_status, strip_esl_body, truncate_head, EsClient, RealEs,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use std::collections::HashMap;
use crate::remaining_a::{envelope, s, tenant_domain};

async fn logs_list(
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let lines: usize = q
        .get("lines")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
        .clamp(1, 1000);
    let level = q.get("level").cloned().unwrap_or_default();
    let level_l = level.to_lowercase();
    let f = q.get("q").cloned().unwrap_or_default().to_lowercase();
    let client = RealEs::from_env();
    let mut content: Vec<String> = Vec::new();
    // ESL-derived lines first (sofia state + FS connectivity).
    for (name, state) in client.sofia_status() {
        content.push(format!("[sofia] {name} {state}"));
    }
    match client.try_api("status") {
        Some(r) if is_connected_body(strip_esl_body(&r)) => {
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
    // Single pass, one lowercase alloc per line (was: two passes, two
    // allocs per line via `to_uppercase` + `to_lowercase`).
    if !level_l.is_empty() || !f.is_empty() {
        content.retain(|l| {
            if !level_l.is_empty() && !f.is_empty() {
                let ll = l.to_lowercase();
                ll.contains(&level_l) && ll.contains(&f)
            } else if !level_l.is_empty() {
                l.to_lowercase().contains(&level_l)
            } else {
                l.to_lowercase().contains(&f)
            }
        });
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
    // One `sofia status` round-trip: parse the same reply (was: two —
    // `api` + `sofia_status()` re-issuing the command).
    let sofia_raw = client.api("sofia status");
    let sofia_parsed = parse_sofia_status(&sofia_raw);
    let regs = client.api("show registrations");
    let channels = client.api("show channels");
    let (connected, _, error) = match client.try_api("status") {
        Some(r) => {
            let body = strip_esl_body(&r);
            if is_connected_body(body) {
                (true, None::<u64>, None::<String>)
            } else {
                (false, None, Some(truncate_head(body, 300)))
            }
        }
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
        "sofia_status": truncate_head(&sofia_raw, 4000),
        "sofia_parsed": sofia_parsed,
        "registrations_raw": truncate_head(&regs, 4000),
        "channels": truncate_head(&channels, 4000),
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
        .route(
            "/api/v1/user_logs",
            axum::routing::post(create_user_log).get(list_user_logs),
        )
        .route(
            "/api/v1/user_logs/{id}",
            axum::routing::get(get_user_log).delete(delete_user_log),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

// ---- user_logs (PHP v_user_logs + Python UserLog superset) ----
type Claims = pbx_core::auth::Claims;
use pbx_db::entities::user_logs;

fn ser_user_log(r: &user_logs::Model) -> serde_json::Value {
    serde_json::json!({
        "user_log_uuid": r.user_log_uuid,
        "domain_id": r.domain_id.map(|d| d.to_string()),
        "user_uuid": r.user_uuid,
        "username": r.username,
        "type": r.log_type,
        "action": r.log_type,
        "result": r.result,
        "detail": r.detail,
        "remote_address": r.remote_address,
        "ip_address": r.remote_address,
        "user_agent": r.user_agent,
        "timestamp": r.timestamp,
        "created_at": r.created_at,
    })
}

async fn create_user_log(
    Extension(claims): Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_log_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = tenant_domain(db, &claims).await;
    let now = crate::routing_db::now();
    let ts = payload
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
        .map(|d| d.with_timezone(&chrono::Utc))
        .or(Some(now));
    let am = user_logs::ActiveModel {
        user_log_uuid: Set(crate::routing_db::new_uuid()),
        domain_id: Set(Some(domain)),
        user_uuid: Set(s(&payload, &["user_uuid"])),
        username: Set(s(&payload, &["username"])),
        log_type: Set(s(&payload, &["type", "action"])),
        result: Set(s(&payload, &["result"])),
        detail: Set(s(&payload, &["detail"])),
        remote_address: Set(s(&payload, &["remote_address", "ip_address"])),
        user_agent: Set(s(&payload, &["user_agent"])),
        timestamp: Set(ts),
        created_at: Set(now),
    };
    match am.insert(db).await {
        Ok(r) => (StatusCode::CREATED, Json(ser_user_log(&r))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response(),
    }
}

async fn list_user_logs(Extension(claims): Extension<Claims>,
    axum::extract::Query(show_q): axum::extract::Query<crate::ShowQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_log_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let show_all = crate::tenant::wants_all(&claims, show_q.show.as_deref(), "user_log_all");
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => return envelope(vec![]).into_response(),
    };
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain = crate::routing_db::domain(&claims);
    let mut sel = user_logs::Entity::find();
    if !show_all {
        sel = sel.filter(user_logs::Column::DomainId.eq(Some(domain)));
    }
    let rows = sel.all(&txn).await.unwrap_or_default();
    envelope(rows.iter().map(ser_user_log).collect()).into_response()
}

async fn get_user_log(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_log_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match user_logs::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == Some(domain) => {
            (StatusCode::OK, Json(ser_user_log(&r))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn delete_user_log(
    Extension(claims): Extension<Claims>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "user_log_delete") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    match user_logs::Entity::find_by_id(id).one(db).await.unwrap_or(None) {
        Some(r) if r.domain_id == Some(domain) => {
            let am: user_logs::ActiveModel = r.into();
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
