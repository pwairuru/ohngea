//! Analytics routers — mirrors `app/cdr/router.py` `/analytics/*`
//! (summary/daily/by_direction/by_hangup/top_extensions/hourly/
//! system_counts/answer_rate).
//!
//! Order: models → schemas → handlers.
//!
//! Portable aggregation: all grouping happens in Rust over SeaORM rows
//! (sqlite + postgres compatible — no `date_trunc` raw SQL). The Rust
//! `cdr` table carries no timestamps, so time buckets attribute live rows
//! to the current period. Empty tables yield zeros — never 500.

use axum::{Extension as AxExtension, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use std::collections::HashMap;

use pbx_db::entities::{cdr, devices, extensions, gateways, sip_profiles, voicemails};

/// Narrow CDR projection: only the small columns analytics groups over.
/// Skips `raw_xml` / `caller_id_number` fat text so a full-tenant scan
/// never buffers megabytes of XML per row.
struct CdrAgg {
    dest: String,
    billsec: Option<i32>,
    duration: Option<i32>,
    hangup: Option<String>,
    direction: Option<String>,
}

async fn tenant_count(db: &sea_orm::DatabaseConnection, domain: uuid::Uuid) -> u64 {
    cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0)
}

async fn tenant_narrow(db: &sea_orm::DatabaseConnection, domain: uuid::Uuid) -> Vec<CdrAgg> {
    cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .select_only()
        .column(cdr::Column::DestinationNumber)
        .column(cdr::Column::Billsec)
        .column(cdr::Column::Duration)
        .column(cdr::Column::HangupCause)
        .column(cdr::Column::Direction)
        .into_tuple::<(
            String,
            Option<i32>,
            Option<i32>,
            Option<String>,
            Option<String>,
        )>()
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(dest, billsec, duration, hangup, direction)| CdrAgg {
            dest,
            billsec,
            duration,
            hangup,
            direction,
        })
        .collect()
}

// ---- handlers ----
// Python parity: `{"total_calls","today_calls","avg_duration"}`.
// No `start_stamp` column in Rust CDR → today == total (documented).
async fn analytics_summary(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = tenant_narrow(db, domain).await;
    let total = rows.len() as u64;
    let sum: i64 = rows.iter().filter_map(|r| r.duration).map(i64::from).sum();
    let with_dur = rows.iter().filter(|r| r.duration.is_some()).count() as f64;
    let avg = if with_dur > 0.0 {
        sum as f64 / with_dur
    } else {
        0.0
    };
    Json(serde_json::json!({
        "total_calls": total,
        "today_calls": total,
        "avg_duration": avg,
    }))
}

// Python parity: last `days` day buckets `{day, count}`.
async fn analytics_daily(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let days: usize = q
        .get("days")
        .and_then(|v| v.parse().ok())
        .unwrap_or(7)
        .clamp(1, 90);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let total = tenant_count(db, domain).await;
    let today = chrono::Utc::now().date_naive();
    let mut data = Vec::with_capacity(days);
    for i in (0..days).rev() {
        let day = today - chrono::Days::new(i as u64);
        let count = if i == 0 { total } else { 0 };
        data.push(serde_json::json!({"day": day.to_string(), "count": count}));
    }
    Json(serde_json::json!({"data": data}))
}

async fn analytics_by_direction(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = tenant_narrow(db, domain).await;
    let mut map: HashMap<String, u64> = HashMap::new();
    for r in &rows {
        *map.entry(r.direction.clone().unwrap_or_else(|| "unknown".to_owned()))
            .or_insert(0) += 1;
    }
    let mut data: Vec<serde_json::Value> = map
        .into_iter()
        .map(|(direction, count)| serde_json::json!({"direction": direction, "count": count}))
        .collect();
    data.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()));
    Json(serde_json::json!({"data": data}))
}

async fn analytics_by_hangup(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = tenant_narrow(db, domain).await;
    let mut map: HashMap<String, u64> = HashMap::new();
    for r in &rows {
        *map.entry(r.hangup.clone().unwrap_or_else(|| "unknown".to_owned()))
            .or_insert(0) += 1;
    }
    let mut data: Vec<serde_json::Value> = map.into_iter().take(10).map(|(hangup_cause, count)| serde_json::json!({"hangup_cause": hangup_cause, "count": count})).collect();
    data.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()));
    Json(serde_json::json!({"data": data}))
}

async fn analytics_top_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let limit: usize = q
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
        .clamp(1, 20);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = tenant_narrow(db, domain).await;
    let mut counts: HashMap<String, (u64, i64, u64)> = HashMap::new();
    for r in &rows {
        let e = counts.entry(r.dest.clone()).or_insert((0, 0, 0));
        e.0 += 1;
        if let Some(b) = r.billsec {
            e.1 += i64::from(b);
            e.2 += 1;
        }
    }
    let mut data: Vec<serde_json::Value> = counts
        .into_iter()
        .map(|(extension, (count, bill_sum, bill_n))| {
            serde_json::json!({
                "extension": extension,
                "count": count,
                "avg_billsec": if bill_n > 0 { bill_sum as f64 / bill_n as f64 } else { 0.0 },
            })
        })
        .collect();
    data.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()));
    data.truncate(limit);
    Json(serde_json::json!({"data": data}))
}

// Python parity: 24 hour buckets `{hour, count}` for today (UTC).
async fn analytics_hourly(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let total = tenant_count(db, domain).await;
    let now_hour = chrono::Utc::now().hour();
    let base = chrono::Utc::now().date_naive();
    let data: Vec<serde_json::Value> = (0..24)
        .map(|h| {
            serde_json::json!({
                "hour": format!("{}T{:02}:00:00+00:00", base, h),
                "count": if h == now_hour { total } else { 0 },
            })
        })
        .collect();
    Json(serde_json::json!({"data": data}))
}

use chrono::Timelike;

async fn analytics_system_counts(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let extensions_n = extensions::Entity::find()
        .filter(extensions::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let devices_n = devices::Entity::find()
        .filter(devices::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let gateways_n = gateways::Entity::find()
        .filter(gateways::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let sip_n = sip_profiles::Entity::find()
        .filter(sip_profiles::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let vm_n = voicemails::Entity::find()
        .filter(voicemails::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let enabled_n = extensions::Entity::find()
        .filter(extensions::Column::DomainId.eq(domain))
        .filter(extensions::Column::Enabled.eq(true))
        .count(db)
        .await
        .unwrap_or(0);
    Json(serde_json::json!({
        "extensions": extensions_n,
        "devices": devices_n,
        "gateways": gateways_n,
        "sip_profiles": sip_n,
        "voicemails": vm_n,
        "enabled_extensions": enabled_n,
        "disabled_extensions": extensions_n.saturating_sub(enabled_n),
    }))
}

async fn analytics_answer_rate(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
) -> Json<serde_json::Value> {
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = tenant_narrow(db, domain).await;
    let total = rows.len() as u64;
    let answered = rows.iter().filter(|r| r.billsec.unwrap_or(0) > 0).count() as u64;
    let missed = total.saturating_sub(answered);
    let bill_sum: i64 = rows.iter().filter_map(|r| r.billsec).map(i64::from).sum();
    let avg_bill = if total > 0 {
        bill_sum as f64 / total as f64
    } else {
        0.0
    };
    Json(serde_json::json!({
        "total": total,
        "answered": answered,
        "missed": missed,
        "answer_rate": if total > 0 { answered as f64 / total as f64 * 100.0 } else { 0.0 },
        "avg_billsec": avg_bill,
    }))
}

pub fn analytics_router() -> Router {
    Router::new()
        .route(
            "/api/v1/analytics/summary",
            axum::routing::get(analytics_summary),
        )
        .route(
            "/api/v1/analytics/daily",
            axum::routing::get(analytics_daily),
        )
        .route(
            "/api/v1/analytics/by_direction",
            axum::routing::get(analytics_by_direction),
        )
        .route(
            "/api/v1/analytics/by_hangup",
            axum::routing::get(analytics_by_hangup),
        )
        .route(
            "/api/v1/analytics/top_extensions",
            axum::routing::get(analytics_top_extensions),
        )
        .route(
            "/api/v1/analytics/hourly",
            axum::routing::get(analytics_hourly),
        )
        .route(
            "/api/v1/analytics/system_counts",
            axum::routing::get(analytics_system_counts),
        )
        .route(
            "/api/v1/analytics/answer_rate",
            axum::routing::get(analytics_answer_rate),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
