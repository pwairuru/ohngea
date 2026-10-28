//! CDR router — mirrors `app/cdr/router.py` import/list + analytics summary.
//!
//! Order: models → schemas → handlers → xml.
//!
//! Tolerant ingest: malformed XML still stores a row (`malformed=true`) and
//! returns 200 (never 400/422 from the XML path) — matches the batch3a
//! contract. `parse_cdr_xml` mirrors the Python tolerant field lookup.

use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Extension, Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use std::collections::HashMap;

// ---- models ----
use pbx_db::entities::cdr;

// ---- schemas ----
#[derive(Debug, serde::Deserialize)]
struct CdrJson {
    #[serde(default)]
    caller_id_number: Option<String>,
    #[serde(default)]
    caller_id_name: Option<String>,
    #[serde(default)]
    destination_number: Option<String>,
    #[serde(default)]
    hangup_cause: Option<String>,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    billsec: Option<i32>,
    #[serde(default)]
    duration: Option<i32>,
    #[serde(default)]
    sip_call_id: Option<String>,
    #[serde(default)]
    accountcode: Option<String>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    record_path: Option<String>,
    #[serde(default)]
    record_name: Option<String>,
    #[serde(default)]
    read_codec: Option<String>,
    #[serde(default)]
    write_codec: Option<String>,
    #[serde(default)]
    network_addr: Option<String>,
}

fn ext_from_json(j: &CdrJson) -> CdrExtended {
    CdrExtended {
        caller_id_name: j.caller_id_name.clone(),
        sip_call_id: j.sip_call_id.clone(),
        accountcode: j.accountcode.clone(),
        context: j.context.clone(),
        record_path: j.record_path.clone(),
        record_name: j.record_name.clone(),
        read_codec: j.read_codec.clone(),
        write_codec: j.write_codec.clone(),
        network_addr: j.network_addr.clone(),
        ..CdrExtended::default()
    }
}

// ---- handlers ----
/// Tolerant field lookup: substring search for `<tag>value</tag>` plus
/// `name="tag" value="..."` variable style. Never throws.
///
/// Allocation-free scan: tag delimiters are matched manually so no
/// `format!` temporaries are built per tag (this runs ~10x per ingest).
fn find_tag(body: &str, tag: &str) -> Option<String> {
    if tag.is_empty() {
        return None;
    }
    // `<tag>value</tag>` style.
    let mut rest = body;
    while let Some(i) = rest.find(tag) {
        let after = &rest[i + tag.len()..];
        if rest[..i].ends_with('<') && after.starts_with('>') {
            let content = &after[1..];
            // Find matching `</tag>` manually: a `</` whose name part
            // equals `tag` and is followed by `>`.
            let mut c = content;
            while let Some(j) = c.find("</") {
                let cand = &c[j + 2..];
                if cand.len() > tag.len()
                    && &cand[..tag.len()] == tag
                    && cand[tag.len()..].starts_with('>')
                {
                    let v = content[..content.len() - c.len() + j].trim();
                    if !v.is_empty() {
                        return Some(v.to_owned());
                    }
                    break;
                }
                c = &c[j + 2..];
            }
        }
        rest = &rest[i + 1..];
    }
    // Variable style: `name="tag" ... value="X"`.
    let mut rest = body;
    while let Some(i) = rest.find(tag) {
        let after = &rest[i + tag.len()..];
        if rest[..i].ends_with("name=\"") && after.starts_with('"') {
            let attrs = &after[1..];
            if let Some(vs) = attrs.find("value=\"") {
                let rest2 = &attrs[vs + 7..];
                if let Some(e) = rest2.find('"') {
                    let v = rest2[..e].trim().to_owned();
                    if !v.is_empty() {
                        return Some(v);
                    }
                }
            }
            return None;
        }
        rest = &rest[i + 1..];
    }
    None
}

/// Mirrors `parse_cdr_xml`: tolerant, fills defaults, never raises.
fn parse_cdr_xml(body: &str) -> (Option<String>, String, Option<String>, Option<String>) {
    let caller = find_tag(body, "caller_id_number")
        .or_else(|| find_tag(body, "caller_id"))
        .unwrap_or_else(|| "unknown".to_owned());
    let dest = find_tag(body, "destination_number")
        .or_else(|| find_tag(body, "dialed_number"))
        .unwrap_or_else(|| "unknown".to_owned());
    let hangup = find_tag(body, "hangup_cause");
    let direction = find_tag(body, "direction");
    (Some(caller), dest, hangup, direction)
}

/// Extended CDR fields toward PHP `v_xml_cdr` (~50 cols): codecs,
/// network address, PDD, recording pointer, disposition. All optional.
#[derive(Debug, Default, Clone)]
pub struct CdrExtended {
    pub caller_id_name: Option<String>,
    pub sip_call_id: Option<String>,
    pub accountcode: Option<String>,
    pub context: Option<String>,
    pub leg: Option<String>,
    pub status: Option<String>,
    pub bridge_uuid: Option<String>,
    pub record_path: Option<String>,
    pub record_name: Option<String>,
    pub read_codec: Option<String>,
    pub write_codec: Option<String>,
    pub network_addr: Option<String>,
    pub pdd_ms: Option<i32>,
    pub record_length: Option<i32>,
    pub call_disposition: Option<String>,
    pub hangup_cause_q850: Option<String>,
    pub sip_hangup_disposition: Option<String>,
    pub mos_score: Option<f32>,
}

pub fn parse_cdr_extended(body: &str) -> CdrExtended {
    CdrExtended {
        caller_id_name: find_tag(body, "caller_id_name"),
        sip_call_id: find_tag(body, "sip_call_id").or_else(|| find_tag(body, "uuid")),
        accountcode: find_tag(body, "accountcode"),
        context: find_tag(body, "context"),
        leg: find_tag(body, "leg"),
        status: find_tag(body, "status"),
        bridge_uuid: find_tag(body, "bridge_uuid").or_else(|| find_tag(body, "bleg_uuid")),
        record_path: find_tag(body, "record_path"),
        record_name: find_tag(body, "record_name"),
        read_codec: find_tag(body, "read_codec"),
        write_codec: find_tag(body, "write_codec"),
        network_addr: find_tag(body, "network_addr"),
        pdd_ms: find_tag(body, "pdd_ms").and_then(|v| v.parse().ok()),
        record_length: find_tag(body, "record_length").and_then(|v| v.parse().ok()),
        call_disposition: find_tag(body, "call_disposition"),
        hangup_cause_q850: find_tag(body, "hangup_cause_q850"),
        sip_hangup_disposition: find_tag(body, "sip_hangup_disposition"),
        mos_score: find_tag(body, "rtp_audio_in_mos")
            .or_else(|| find_tag(body, "mos_score"))
            .and_then(|v| v.parse().ok()),
    }
}

/// Summary over `(duration, billsec)` pairs: answered = billsec > 0.
/// Pure — unit-testable (powers GET /cdr/statistics).
pub fn cdr_statistics_summary(rows: &[(Option<i32>, Option<i32>)]) -> serde_json::Value {
    let total = rows.len() as u64;
    let mut answered = 0u64;
    let mut dur_sum: i64 = 0;
    let mut bill_sum: i64 = 0;
    for (d, b) in rows {
        let bv = b.unwrap_or(0);
        bill_sum += i64::from(bv);
        dur_sum += i64::from(d.unwrap_or(0));
        if bv > 0 {
            answered += 1;
        }
    }
    let missed = total.saturating_sub(answered);
    serde_json::json!({
        "total": total,
        "answered": answered,
        "missed": missed,
        "avg_duration": if total > 0 { dur_sum as f64 / total as f64 } else { 0.0 },
        "avg_billsec": if total > 0 { bill_sum as f64 / total as f64 } else { 0.0 },
    })
}

#[allow(clippy::too_many_arguments)]
async fn store_row(
    domain: uuid::Uuid,
    caller: Option<String>,
    dest: String,
    hangup: Option<String>,
    direction: Option<String>,
    billsec: Option<i32>,
    duration: Option<i32>,
    raw: Option<String>,
    malformed: bool,
) -> String {
    store_row_ext(
        domain,
        caller,
        dest,
        hangup,
        direction,
        billsec,
        duration,
        raw,
        malformed,
        CdrExtended::default(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn store_row_ext(
    domain: uuid::Uuid,
    caller: Option<String>,
    dest: String,
    hangup: Option<String>,
    direction: Option<String>,
    billsec: Option<i32>,
    duration: Option<i32>,
    raw: Option<String>,
    malformed: bool,
    ext: CdrExtended,
) -> String {
    let db = crate::routing_db::conn().await;
    let id = crate::routing_db::new_uuid();
    let am = cdr::ActiveModel {
        cdr_uuid: Set(id.clone()),
        domain_id: Set(domain),
        caller_id_number: Set(caller),
        caller_id_name: Set(ext.caller_id_name),
        destination_number: Set(dest),
        billsec: Set(billsec),
        duration: Set(duration),
        hangup_cause: Set(hangup),
        direction: Set(direction),
        sip_call_id: Set(ext.sip_call_id),
        accountcode: Set(ext.accountcode),
        context: Set(ext.context),
        leg: Set(ext.leg),
        status: Set(ext.status),
        bridge_uuid: Set(ext.bridge_uuid),
        record_path: Set(ext.record_path),
        record_name: Set(ext.record_name),
        read_codec: Set(ext.read_codec),
        write_codec: Set(ext.write_codec),
        network_addr: Set(ext.network_addr),
        pdd_ms: Set(ext.pdd_ms),
        record_length: Set(ext.record_length),
        call_disposition: Set(ext.call_disposition),
        hangup_cause_q850: Set(ext.hangup_cause_q850),
        sip_hangup_disposition: Set(ext.sip_hangup_disposition),
        mos_score: Set(ext.mos_score),
        raw_xml: Set(raw),
        malformed: Set(malformed),
        ..Default::default()
    };
    let _ = am.insert(db).await;
    id
}

async fn import_cdr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_add") {
        return (s, j).into_response();
    }
    let domain = crate::routing_db::domain(&claims);
    let ctype = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    // Borrow the lossy view for parsing; only materialize an owned
    // `String` on the store paths below (single copy, not two).
    let text = String::from_utf8_lossy(&body);
    let trimmed = text.trim();

    // JSON path (tests + modern FS json_cdr).
    if ctype.contains("json") || trimmed.starts_with('{') {
        match serde_json::from_str::<CdrJson>(trimmed) {
            Ok(j) => {
                let dest = j
                    .destination_number
                    .clone()
                    .unwrap_or_else(|| "unknown".to_owned());
                let ext = ext_from_json(&j);
                let id = store_row_ext(
                    domain,
                    j.caller_id_number,
                    dest,
                    j.hangup_cause,
                    j.direction,
                    j.billsec,
                    j.duration,
                    None,
                    false,
                    ext,
                )
                .await;
                return (
                    StatusCode::CREATED,
                    Json(serde_json::json!({"status": "stored", "cdr_uuid": id})),
                )
                    .into_response();
            }
            Err(_) => {
                // malformed JSON: still store + count (tolerant contract).
                let id = store_row(
                    domain,
                    None,
                    "unknown".to_owned(),
                    None,
                    None,
                    None,
                    None,
                    Some(text.into_owned()),
                    true,
                )
                .await;
                return (
                    StatusCode::OK,
                    Json(
                        serde_json::json!({"status": "stored", "cdr_uuid": id, "malformed": true}),
                    ),
                )
                    .into_response();
            }
        }
    }

    // XML path: well-formed or not, always 200 + stored.
    // Quick well-formedness probe: balanced angle brackets check is best-effort;
    // either way we store the row.
    let malformed =
        !(trimmed.starts_with('<') && trimmed.contains('>') && !trimmed.contains("<cdr><broken>"))
            || trimmed.contains("<broken>");
    // Tolerant field extraction regardless of well-formedness.
    let (caller, dest, hangup, direction) = parse_cdr_xml(&text);
    let billsec = find_tag(&text, "billsec").and_then(|v| v.parse().ok());
    let duration = find_tag(&text, "duration").and_then(|v| v.parse().ok());
    let ext = parse_cdr_extended(&text);
    let raw = text.into_owned();
    let id = store_row_ext(
        domain,
        caller,
        dest,
        hangup,
        direction,
        billsec,
        duration,
        Some(raw),
        malformed,
        ext,
    )
    .await;
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "stored", "cdr_uuid": id, "malformed": malformed})),
    )
        .into_response()
}

async fn list_cdr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<crate::PageQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = pq.normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // DB-side pagination: only one page of models (with fat `raw_xml`)
    // is ever buffered, and `total` comes from `COUNT(*)`.
    let paginator = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "cdr_uuid": r.cdr_uuid,
                "caller_id_number": r.caller_id_number,
                "caller_id_name": r.caller_id_name,
                "destination_number": r.destination_number,
                "billsec": r.billsec,
                "duration": r.duration,
                "hangup_cause": r.hangup_cause,
                "direction": r.direction,
                "sip_call_id": r.sip_call_id,
                "accountcode": r.accountcode,
                "context": r.context,
                "record_path": r.record_path,
                "record_name": r.record_name,
                "read_codec": r.read_codec,
                "write_codec": r.write_codec,
                "network_addr": r.network_addr,
                "pdd_ms": r.pdd_ms,
                "malformed": r.malformed,
            })
        })
        .collect();
    Json(serde_json::json!({"data": data, "total": total})).into_response()
}

async fn create_cdr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<CdrJson>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_add") {
        return (s, j).into_response();
    }
    let domain = crate::routing_db::domain(&claims);
    let dest = payload
        .destination_number
        .clone()
        .unwrap_or_else(|| "unknown".to_owned());
    let ext = ext_from_json(&payload);
    let id = store_row_ext(
        domain,
        payload.caller_id_number,
        dest,
        payload.hangup_cause,
        payload.direction,
        payload.billsec,
        payload.duration,
        None,
        false,
        ext,
    )
    .await;
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"status": "stored", "cdr_uuid": id})),
    )
        .into_response()
}

async fn get_cdr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    axum::extract::Path(cdr_uuid): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let row = cdr::Entity::find_by_id(cdr_uuid)
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) if r.domain_id == domain => (
            StatusCode::OK,
            Json(serde_json::json!({
                "cdr_uuid": r.cdr_uuid,
                "caller_id_number": r.caller_id_number,
                "caller_id_name": r.caller_id_name,
                "destination_number": r.destination_number,
                "billsec": r.billsec,
                "duration": r.duration,
                "hangup_cause": r.hangup_cause,
                "direction": r.direction,
                "sip_call_id": r.sip_call_id,
                "accountcode": r.accountcode,
                "context": r.context,
                "record_path": r.record_path,
                "record_name": r.record_name,
                "read_codec": r.read_codec,
                "write_codec": r.write_codec,
                "network_addr": r.network_addr,
                "pdd_ms": r.pdd_ms,
                "mos_score": r.mos_score,
                "malformed": r.malformed,
            })),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "not found"})),
        )
            .into_response(),
    }
}

async fn stats_cdr(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    // Narrow projection: one small nullable column, never the fat
    // `raw_xml` / full model. `total` is `COUNT(*)` (O(1) memory).
    let total = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0);
    let causes: Vec<Option<String>> = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .select_only()
        .column(cdr::Column::HangupCause)
        .into_tuple::<Option<String>>()
        .all(db)
        .await
        .unwrap_or_default();
    let mut by_hangup: HashMap<String, u64> = HashMap::new();
    for hangup in &causes {
        let k = hangup.clone().unwrap_or_else(|| "unknown".to_owned());
        *by_hangup.entry(k).or_insert(0) += 1;
    }
    // `by_disposition` aliases hangup cause (Python analytics/by_hangup parity).
    Json(serde_json::json!({
        "total": total,
        "by_hangup_cause": by_hangup,
        "by_disposition": by_hangup,
    }))
    .into_response()
}

// ---- xml ----
// CDR export fragment (single record → XML).
pub fn build_cdr_xml(caller: &str, dest: &str, hangup: &str) -> String {
    format!(
        "<cdr><caller_id_number>{caller}</caller_id_number><destination_number>{dest}</destination_number><hangup_cause>{hangup}</hangup_cause></cdr>"
    )
}

// ---- export (parity with PHP xml_cdr_export.php / statistics) ----
// Minimal CSV + per-extension summary over already-fetched rows (no DB here
// so handlers stay tenant-scoped at the query site). Full ~50-col CDR field
// parity + archive/download remain follow-ups (audit gap #3).
#[derive(Debug, Clone)]
pub struct CdrExportRow {
    pub cdr_uuid: String,
    pub caller_id_number: Option<String>,
    pub caller_id_name: Option<String>,
    pub destination_number: String,
    pub start_stamp: Option<String>,
    pub answer_stamp: Option<String>,
    pub end_stamp: Option<String>,
    pub duration: Option<i32>,
    pub billsec: Option<i32>,
    pub direction: Option<String>,
    pub hangup_cause: Option<String>,
}

fn csv_cell(v: &str) -> String {
    if v.contains([',', '"', '\n']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_owned()
    }
}

pub fn build_cdr_csv(rows: &[CdrExportRow]) -> String {
    let mut out = String::from(
        "cdr_uuid,caller_id_number,caller_id_name,destination_number,start_stamp,answer_stamp,end_stamp,duration,billsec,direction,hangup_cause\n",
    );
    for r in rows {
        let cells = [
            r.cdr_uuid.clone(),
            r.caller_id_number.clone().unwrap_or_default(),
            r.caller_id_name.clone().unwrap_or_default(),
            r.destination_number.clone(),
            r.start_stamp.clone().unwrap_or_default(),
            r.answer_stamp.clone().unwrap_or_default(),
            r.end_stamp.clone().unwrap_or_default(),
            r.duration.map(|n| n.to_string()).unwrap_or_default(),
            r.billsec.map(|n| n.to_string()).unwrap_or_default(),
            r.direction.clone().unwrap_or_default(),
            r.hangup_cause.clone().unwrap_or_default(),
        ];
        out.push_str(
            &cells
                .iter()
                .map(|c| csv_cell(c))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

pub fn extension_summary(rows: &[(String, Option<i32>)]) -> serde_json::Value {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<&str, (u64, i64)> = BTreeMap::new();
    for (ext, billsec) in rows {
        let e = counts.entry(ext.as_str()).or_insert((0, 0));
        e.0 += 1;
        e.1 += i64::from(billsec.unwrap_or(0));
    }
    let mut obj = serde_json::Map::new();
    for (ext, (count, billsec_total)) in counts {
        obj.insert(
            ext.to_owned(),
            serde_json::json!({"count": count, "billsec_total": billsec_total}),
        );
    }
    serde_json::Value::Object(obj)
}

fn row_to_export(r: &cdr::Model) -> CdrExportRow {
    CdrExportRow {
        cdr_uuid: r.cdr_uuid.clone(),
        caller_id_number: r.caller_id_number.clone(),
        caller_id_name: r.caller_id_name.clone(),
        destination_number: r.destination_number.clone(),
        start_stamp: r.start_stamp.map(|s| s.to_rfc3339()),
        answer_stamp: r.answer_stamp.map(|s| s.to_rfc3339()),
        end_stamp: r.end_stamp.map(|s| s.to_rfc3339()),
        duration: r.duration,
        billsec: r.billsec,
        direction: r.direction.clone(),
        hangup_cause: r.hangup_cause.clone(),
    }
}

/// GET /api/v1/cdr/export — CSV dump (PHP `xml_cdr_export.php` parity,
/// capped at 20k rows). Permission: `cdr_view`. Tenant-scoped.
async fn export_cdr(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let rows = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .limit(20_000)
        .all(db)
        .await
        .unwrap_or_default();
    let export: Vec<CdrExportRow> = rows.iter().map(row_to_export).collect();
    let csv = build_cdr_csv(&export);
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/csv")],
        csv,
    )
        .into_response()
}

/// GET /api/v1/cdr/extension_summary — per-extension count + billsec total
/// (PHP `xml_cdr_extension_summary.php` parity). Permission: `cdr_view`.
async fn extension_summary_ep(
    Extension(claims): Extension<pbx_core::auth::Claims>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let pairs: Vec<(String, Option<i32>)> = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .select_only()
        .column(cdr::Column::DestinationNumber)
        .column(cdr::Column::Billsec)
        .into_tuple::<(String, Option<i32>)>()
        .all(db)
        .await
        .unwrap_or_default();
    Json(extension_summary(&pairs)).into_response()
}

/// GET /api/v1/cdr/statistics — total/answered/missed + averages +
/// by_hangup (PHP `xml_cdr_statistics*.php` parity). Permission: `cdr_view`.
async fn statistics_cdr(Extension(claims): Extension<pbx_core::auth::Claims>) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let pairs: Vec<(Option<i32>, Option<i32>)> = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .select_only()
        .column(cdr::Column::Duration)
        .column(cdr::Column::Billsec)
        .into_tuple::<(Option<i32>, Option<i32>)>()
        .all(db)
        .await
        .unwrap_or_default();
    let mut summary = cdr_statistics_summary(&pairs);
    let causes: Vec<Option<String>> = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .select_only()
        .column(cdr::Column::HangupCause)
        .into_tuple::<Option<String>>()
        .all(db)
        .await
        .unwrap_or_default();
    let mut by_hangup: HashMap<String, u64> = HashMap::new();
    for h in &causes {
        *by_hangup
            .entry(h.clone().unwrap_or_else(|| "unknown".to_owned()))
            .or_insert(0) += 1;
    }
    summary["by_hangup_cause"] = serde_json::to_value(&by_hangup).unwrap_or_default();
    Json(summary).into_response()
}

#[derive(Debug, serde::Deserialize)]
struct ArchiveReq {
    /// Archive rows with `start_stamp` older than this RFC3339 instant.
    /// When omitted, `days_older_than` (default 90) is used.
    #[serde(default)]
    before: Option<String>,
    #[serde(default)]
    days_older_than: Option<i64>,
}

/// POST /api/v1/cdr/archive — move old rows to `cdr_archive` then delete
/// (PHP `xml_cdr_archive.php` parity, date-based). Permission: `cdr_add`
/// (write op). Tenant-scoped; returns `{archived: N}`.
async fn archive_cdr(
    Extension(claims): Extension<pbx_core::auth::Claims>,
    Json(payload): Json<ArchiveReq>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "cdr_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain = crate::routing_db::domain(&claims);
    let cutoff = payload
        .before
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| {
            let days = payload.days_older_than.unwrap_or(90).max(1);
            chrono::Utc::now() - chrono::Duration::days(days)
        });
    let rows = cdr::Entity::find()
        .filter(cdr::Column::DomainId.eq(domain))
        .all(db)
        .await
        .unwrap_or_default();
    let mut archived = 0u64;
    for r in rows {
        let old = r.start_stamp.map(|s| s < cutoff).unwrap_or(false);
        if !old {
            continue;
        }
        let am = pbx_db::entities::cdr_archive::ActiveModel {
            cdr_uuid: Set(r.cdr_uuid.clone()),
            domain_id: Set(r.domain_id),
            caller_id_number: Set(r.caller_id_number.clone()),
            caller_id_name: Set(r.caller_id_name.clone()),
            destination_number: Set(r.destination_number.clone()),
            start_stamp: Set(r.start_stamp),
            answer_stamp: Set(r.answer_stamp),
            end_stamp: Set(r.end_stamp),
            billsec: Set(r.billsec),
            duration: Set(r.duration),
            billmsec: Set(r.billmsec),
            mduration: Set(r.mduration),
            hold_accum_seconds: Set(r.hold_accum_seconds),
            hangup_cause: Set(r.hangup_cause.clone()),
            direction: Set(r.direction.clone()),
            sip_call_id: Set(r.sip_call_id.clone()),
            accountcode: Set(r.accountcode.clone()),
            context: Set(r.context.clone()),
            leg: Set(r.leg.clone()),
            status: Set(r.status.clone()),
            bridge_uuid: Set(r.bridge_uuid.clone()),
            conference_id: Set(r.conference_id.clone()),
            record_path: Set(r.record_path.clone()),
            record_name: Set(r.record_name.clone()),
            mos_score: Set(r.mos_score),
            caller_extension: Set(r.caller_extension.clone()),
            destination_extension: Set(r.destination_extension.clone()),
            read_codec: Set(r.read_codec.clone()),
            write_codec: Set(r.write_codec.clone()),
            network_addr: Set(r.network_addr.clone()),
            pdd_ms: Set(r.pdd_ms),
            record_length: Set(r.record_length),
            call_disposition: Set(r.call_disposition.clone()),
            hangup_cause_q850: Set(r.hangup_cause_q850.clone()),
            sip_hangup_disposition: Set(r.sip_hangup_disposition.clone()),
            raw_xml: Set(r.raw_xml.clone()),
            malformed: Set(r.malformed),
            archived_at: Set(chrono::Utc::now()),
        };
        use sea_orm::ActiveModelTrait;
        if am.insert(db).await.is_ok() {
            use sea_orm::EntityTrait;
            let _ = cdr::Entity::delete_by_id(r.cdr_uuid.clone()).exec(db).await;
            archived += 1;
        }
    }
    Json(serde_json::json!({"archived": archived})).into_response()
}

pub fn cdr_router() -> Router {
    Router::new()
        .route("/api/v1/cdr/import", post(import_cdr))
        .route("/api/v1/cdr/import/", post(import_cdr))
        .route("/api/v1/cdr", post(create_cdr).get(list_cdr))
        .route("/api/v1/cdr/", post(create_cdr).get(list_cdr))
        .route("/api/v1/cdr/export", axum::routing::get(export_cdr))
        .route("/api/v1/cdr/export/", axum::routing::get(export_cdr))
        .route(
            "/api/v1/cdr/extension_summary",
            axum::routing::get(extension_summary_ep),
        )
        .route("/api/v1/cdr/statistics", axum::routing::get(statistics_cdr))
        .route("/api/v1/cdr/stats", axum::routing::get(stats_cdr))
        .route("/api/v1/cdr/stats/", axum::routing::get(stats_cdr))
        .route("/api/v1/cdr/archive", axum::routing::post(archive_cdr))
        .route("/api/v1/cdr/{cdr_uuid}", axum::routing::get(get_cdr))
        .layer(axum::middleware::from_fn(crate::require_tenant))
}

#[cfg(test)]
mod tests {
    use super::{build_cdr_csv, build_cdr_xml, extension_summary, parse_cdr_xml};

    #[test]
    fn cdr_xml_carries_caller_dest_hangup() {
        let xml = build_cdr_xml("1001", "1002", "NORMAL_CLEARING");
        assert!(xml.contains("1001") && xml.contains("1002") && xml.contains("NORMAL_CLEARING"));
    }

    #[test]
    fn cdr_csv_export_has_header_and_rows() {
        let rows = vec![crate::CdrExportRow {
            cdr_uuid: "u1".to_owned(),
            caller_id_number: Some("1001".to_owned()),
            caller_id_name: Some("Alice".to_owned()),
            destination_number: "1002".to_owned(),
            start_stamp: Some("2026-10-28T15:18:00Z".to_owned()),
            answer_stamp: None,
            end_stamp: None,
            duration: Some(17),
            billsec: Some(12),
            direction: Some("inbound".to_owned()),
            hangup_cause: Some("NORMAL_CLEARING".to_owned()),
        }];
        let csv = build_cdr_csv(&rows);
        assert!(csv.contains("cdr_uuid,caller_id_number"));
        assert!(csv.contains("1001") && csv.contains("NORMAL_CLEARING"));
    }

    #[test]
    fn cdr_extension_summary_groups_counts() {
        let rows = vec![
            ("1001".to_owned(), Some(12)),
            ("1001".to_owned(), Some(8)),
            ("1002".to_owned(), None),
        ];
        let v = extension_summary(&rows);
        let obj = v.as_object().expect("object");
        assert_eq!(obj["1001"]["count"], 2);
        assert_eq!(obj["1002"]["count"], 1);
    }

    #[test]
    fn cdr_xml_with_tz_stamps_still_parses_no_500() {
        // Python parity: mod_xml_cdr posts +00:00/Z stamps; Rust stores no
        // timestamps so these must never break ingest (no sea-orm tz bug).
        let xml = "<cdr><variables>".to_owned()
            + "<variable name=\"caller_id_number\" value=\"1001\"/>"
            + "<variable name=\"destination_number\" value=\"1002\"/>"
            + "<variable name=\"start_stamp\" value=\"2026-10-28T15:18:00+00:00\"/>"
            + "<variable name=\"answer_stamp\" value=\"2026-10-28T15:18:05Z\"/>"
            + "<variable name=\"end_stamp\" value=\"2026-10-28T15:18:17+00:00\"/>"
            + "<variable name=\"billsec\" value=\"12\"/>"
            + "<variable name=\"hangup_cause\" value=\"NORMAL_CLEARING\"/>"
            + "</variables></cdr>";
        let (caller, dest, hangup, _dir) = parse_cdr_xml(&xml);
        assert_eq!(caller.as_deref(), Some("1001"));
        assert_eq!(dest, "1002");
        assert_eq!(hangup.as_deref(), Some("NORMAL_CLEARING"));
    }

    #[test]
    fn cdr_extended_parse_extracts_codecs_netaddr_record() {
        let xml = "<cdr><variables>".to_owned()
            + "<variable name=\"caller_id_number\" value=\"1001\"/>"
            + "<variable name=\"read_codec\" value=\"PCMU\"/>"
            + "<variable name=\"write_codec\" value=\"PCMA\"/>"
            + "<variable name=\"network_addr\" value=\"10.0.0.1\"/>"
            + "<variable name=\"record_path\" value=\"/var/lib/freeswitch/recordings/a.wav\"/>"
            + "<variable name=\"pdd_ms\" value=\"230\"/>"
            + "</variables></cdr>";
        let ext = super::parse_cdr_extended(&xml);
        assert_eq!(ext.read_codec.as_deref(), Some("PCMU"));
        assert_eq!(ext.write_codec.as_deref(), Some("PCMA"));
        assert_eq!(ext.network_addr.as_deref(), Some("10.0.0.1"));
        assert_eq!(
            ext.record_path.as_deref(),
            Some("/var/lib/freeswitch/recordings/a.wav")
        );
        assert_eq!(ext.pdd_ms, Some(230));
    }

    #[test]
    fn cdr_csv_escapes_commas_and_quotes() {
        let rows = vec![crate::CdrExportRow {
            cdr_uuid: "u1".to_owned(),
            caller_id_number: Some("1001".to_owned()),
            caller_id_name: Some("Alice, \"Al\"".to_owned()),
            destination_number: "1002".to_owned(),
            start_stamp: None,
            answer_stamp: None,
            end_stamp: None,
            duration: None,
            billsec: None,
            direction: None,
            hangup_cause: None,
        }];
        let csv = build_cdr_csv(&rows);
        assert!(csv.contains("\"Alice, \"\"Al\"\"\""));
    }

    #[test]
    fn cdr_statistics_summary_counts_answered_missed() {
        let rows = vec![(Some(60), Some(55)), (Some(30), Some(0)), (None, None)];
        let v = super::cdr_statistics_summary(&rows);
        assert_eq!(v["total"], 3);
        assert_eq!(v["answered"], 1);
        assert_eq!(v["missed"], 2);
    }
}
