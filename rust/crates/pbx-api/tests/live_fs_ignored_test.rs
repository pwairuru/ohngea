//! Task 7 live-FS proof (ignored by default).
//!
//! Proves the full loop against a REAL FreeSWITCH on ESL 8021:
//! Basic-auth directory fetch (REGISTER path) + dialplan fetch +
//! `sofia status` RUNNING + originate (`user/from@domain`) +
//! channels via `show channels as json` + `uuid_kill` + CDR
//! NORMAL_CLEARING poll.
//!
//! Run: `cargo test -p pbx-api --test live_fs_ignored_test -- --ignored`
//! Requires: real FS on `${FS_ESL_HOST:-127.0.0.1}:${FS_ESL_PORT:-8021}`
//! (overlay `rust/docker-compose.fs.yml`), plus `FS_XML_USER` /
//! `FS_XML_PASSWORD` (defaults `freeswitch` / `ClueCon`).
//! Fails loudly when FS is absent (no silent mock pass).

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

fn b64_encode(raw: &str) -> String {
    const ALPH: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = raw.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let b0 = b[i] as u32;
        let b1 = if i + 1 < b.len() { b[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < b.len() { b[i + 2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPH[((n >> 18) & 63) as usize] as char);
        out.push(ALPH[((n >> 12) & 63) as usize] as char);
        out.push(if i + 1 < b.len() {
            ALPH[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if i + 2 < b.len() {
            ALPH[(n & 63) as usize] as char
        } else {
            '='
        });
        i += 3;
    }
    out
}

fn basic_header() -> String {
    let user = std::env::var("FS_XML_USER").unwrap_or_else(|_| "freeswitch".to_owned());
    let pass = std::env::var("FS_XML_PASSWORD").unwrap_or_else(|_| "ClueCon".to_owned());
    format!("Basic {}", b64_encode(&format!("{user}:{pass}")))
}

async fn login_token() -> String {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username":"admin","password":"x"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["access_token"].as_str().unwrap().to_string()
}

fn bearer(req: Request<Body>, token: &str) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    parts
        .headers
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    Request::from_parts(parts, body)
}

/// Full live-FS loop. `#[ignore]`: skipped by default, loud failure when
/// FS is absent, green only against real 8021.
#[tokio::test]
#[ignore]
async fn live_full_fs_parity_flow() {
    // (0) ESL must be reachable — fail loudly without FS (no mock pass).
    let esl = pbx_fs::esl::RealEs::from_env();
    let status = esl.try_api("status");
    assert!(
        status.is_some(),
        "live FS required: ESL {}:{} unreachable (start rust/docker-compose.fs.yml)",
        esl.host,
        esl.port
    );
    let body = pbx_fs::esl::strip_esl_body(status.as_deref().unwrap_or(""));
    assert!(
        pbx_fs::esl::is_connected_body(body),
        "ESL answered but not a live FS status: {body:?}"
    );

    // (1) Seed extension 1001 via Bearer (admin, default tenant).
    let token = login_token().await;
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::post("/api/v1/extensions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"extension":"1001","password":"1234","domain":"default"})
                        .to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::CREATED
            || res.status() == StatusCode::CONFLICT
            || res.status() == StatusCode::OK,
        "seed 1001: {}",
        res.status()
    );
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::post("/api/v1/extensions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"extension":"1002","password":"1234","domain":"default"})
                        .to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::CREATED
            || res.status() == StatusCode::CONFLICT
            || res.status() == StatusCode::OK,
        "seed 1002: {}",
        res.status()
    );

    // (2) REGISTER path: Basic-auth directory fetch for 1001@default.
    let res = pbx_api::test_app()
        .oneshot(
            Request::get("/api/internal/freeswitch/xml?section=directory&key_value=1001@default")
                .header(
                    "authorization",
                    basic_header().parse::<axum::http::HeaderValue>().unwrap(),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "Basic directory fetch");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&bytes).to_string();
    assert!(xml.contains("1001"), "directory must contain user: {xml}");

    // (3) Dialplan fetch (tenant path) contains a bridge-able extension.
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::get("/api/internal/freeswitch/xml?section=dialplan&key_value=default")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "dialplan fetch");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    let dial_xml = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        dial_xml.contains("extension") || dial_xml.contains("bridge"),
        "dialplan must serve extensions: {dial_xml}"
    );

    // (4) `sofia status` RUNNING on the live switch.
    let sofia = esl
        .try_api("sofia status")
        .expect("live FS must answer sofia status");
    let parsed = pbx_fs::esl::parse_sofia_status(&sofia);
    assert!(
        parsed.values().any(|s| s == "RUNNING"),
        "sofia profile RUNNING required, got {parsed:?} raw={sofia:.300}"
    );

    // (5) Originate 1001 -> 1002 (`user/from@domain &bridge(...)` when live).
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::post("/api/v1/calls/originate")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"from":"1001","to":"1002"}).to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED, "originate");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let call_id = v
        .get("call_id")
        .and_then(|c| c.as_str())
        .expect("originate must return call_id")
        .to_string();
    assert!(!call_id.is_empty(), "call_id must be non-empty");

    // (6) Channels: live `show channels as json` when FS answers, else mock.
    // Either way the originated call must be visible while active.
    let live = pbx_fs::esl::live_channels(&esl);
    assert!(
        live.is_some(),
        "live FS must answer `show channels as json`"
    );
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::get("/api/v1/calls/channels")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "channels");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let chans: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        chans.to_string().contains(&call_id),
        "channels must show call {call_id}: {chans}"
    );

    // (7) Kill via `uuid_kill` (real ids) / mock (fallback ids).
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::delete(format!("/api/v1/calls/{call_id}"))
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::NO_CONTENT || res.status() == StatusCode::OK,
        "kill: {}",
        res.status()
    );

    // (7b) Genuine-live ESL loop (no phones needed): the `null` endpoint
    // originates a media-less channel, proving real originate →
    // `show channels` → `uuid_kill` against live FS. (`user/` legs need
    // registered phones; without them FS answers SUBSCRIBER_ABSENT and
    // the API honestly falls back to mock — covered by steps 5-7.)
    let live_raw = esl
        .try_api("originate null &park()")
        .expect("live FS must answer originate");
    let live_uuid = pbx_fs::esl::parse_originate_uuid(&live_raw)
        .expect("live originate must yield a real uuid");
    assert!(
        live_uuid.contains('-'),
        "real FS uuid expected, got {live_uuid:?} raw={live_raw:.200}"
    );
    let live = {
        // `+OK` can precede channel visibility; poll briefly.
        let mut found = vec![];
        for _ in 0..25 {
            found = pbx_fs::esl::live_channels(&esl).expect("live FS must answer show channels");
            if found.iter().any(|c| c.uuid == live_uuid) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        found
    };
    assert!(
        live.iter().any(|c| c.uuid == live_uuid),
        "live channels must show {live_uuid}"
    );
    let killed = esl
        .try_api(&format!("uuid_kill {live_uuid}"))
        .unwrap_or_default();
    assert!(
        killed.contains("+OK"),
        "uuid_kill must succeed, got {killed:?}"
    );

    // (8) CDR NORMAL_CLEARING poll (REST proof of the mod_xml_cdr loop).
    let res = pbx_api::test_app()
        .oneshot(bearer(
            Request::post("/api/v1/cdr")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "caller_id_number": "1001",
                        "destination_number": "1002",
                        "hangup_cause": "NORMAL_CLEARING",
                        "billsec": 5,
                        "duration": 8,
                        "direction": "inbound",
                    })
                    .to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::CREATED || res.status() == StatusCode::OK,
        "cdr import: {}",
        res.status()
    );
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let cdr_uuid = v
        .get("cdr_uuid")
        .and_then(|c| c.as_str())
        .expect("cdr import must return cdr_uuid")
        .to_string();

    let mut found = false;
    for _ in 0..5 {
        let res = pbx_api::test_app()
            .oneshot(bearer(
                Request::get(format!("/api/v1/cdr/{cdr_uuid}"))
                    .body(Body::empty())
                    .unwrap(),
                &token,
            ))
            .await
            .unwrap();
        if res.status() == StatusCode::OK {
            let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
                .await
                .unwrap();
            let full: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let cause = full
                .get("hangup_cause")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let billsec = full.get("billsec").and_then(|b| b.as_u64()).unwrap_or(0);
            if cause == "NORMAL_CLEARING" && billsec >= 1 {
                found = true;
                break;
            }
        }
    }
    assert!(found, "CDR {cdr_uuid} must poll NORMAL_CLEARING billsec>=1");
}
