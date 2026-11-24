//! Round-2 parity: music_on_hold chime/timer fields, call_broadcast AVMD/
//! account/toll fields, fifo agent/queue/exit fields, voicemail
//! transcription/forward/alternate fields — PHP ground truth, all dropped
//! by the API today.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

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

fn auth(req: Request<Body>, token: &str) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    parts
        .headers
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    Request::from_parts(parts, body)
}

async fn call(
    app: axum::Router,
    token: &str,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let b = body.map(|v| v.to_string()).unwrap_or_default();
    let req = match method {
        "POST" => Request::post(path),
        "PATCH" => Request::patch(path),
        _ => Request::get(path),
    };
    let res = app
        .oneshot(auth(
            req.header("content-type", "application/json")
                .body(Body::from(b))
                .unwrap(),
            token,
        ))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn moh_chime_timer_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/music_on_hold",
        Some(serde_json::json!({"music_name": "moh-p", "music_on_hold_timer_name": "soft", "music_on_hold_chime_list": "a:b", "music_on_hold_chime_freq": "30", "music_on_hold_chime_max": 3})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create moh: {b}");
    for key in [
        "music_timer_name",
        "music_chime_list",
        "music_chime_freq",
        "music_chime_max",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["music_on_hold_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/music_on_hold/{uuid}"),
        Some(serde_json::json!({"music_chime_max": 5})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["music_chime_max"], 5);
}

#[tokio::test]
async fn broadcast_avmd_account_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/call_broadcasts",
        Some(serde_json::json!({"broadcast_name": "bc-p", "broadcast_avmd": true, "broadcast_accountcode": "a1", "broadcast_toll_allow": "dom", "broadcast_start_time": 1700000000})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create bc: {b}");
    for key in [
        "broadcast_avmd",
        "broadcast_accountcode",
        "broadcast_toll_allow",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["call_broadcast_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/call_broadcasts/{uuid}"),
        Some(serde_json::json!({"broadcast_avmd": false})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["broadcast_avmd"], false);
}

#[tokio::test]
async fn fifo_agent_queue_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/fifo",
        Some(serde_json::json!({"fifo_name": "fifo-p", "fifo_agent_queue": "agents", "fifo_agent_status": "avail", "fifo_timeout_seconds": "30", "fifo_exit_key": "0", "fifo_strategy": "ringall", "fifo_order": 3})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create fifo: {b}");
    for key in [
        "fifo_agent_queue",
        "fifo_timeout_seconds",
        "fifo_exit_key",
        "fifo_strategy",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["fifo_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/fifo/{uuid}"),
        Some(serde_json::json!({"fifo_timeout_seconds": "45"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["fifo_timeout_seconds"], "45");
}

#[tokio::test]
async fn fax_email_and_channels_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/fax",
        Some(serde_json::json!({"fax_extension": "7799", "fax_name": "Fax P", "fax_toll_allow": "dom", "fax_send_channels": 2, "fax_email_connection_host": "mail.x", "fax_user_list": "u1"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create fax: {b}");
    let uuid = b["fax_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/fax/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(g["fax_toll_allow"], "dom", "persisted: {g}");
    assert_eq!(g["fax_send_channels"], 2);
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/fax/{uuid}"),
        Some(serde_json::json!({"fax_send_channels": 4})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["fax_send_channels"], 4);
}

#[tokio::test]
async fn voicemail_forward_alternate_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/voicemails",
        Some(serde_json::json!({"voicemail_id": "5599", "voicemail_password": "pw1234", "voicemail_transcription_enabled": true, "voicemail_transcription_prompt": "say it", "voicemail_local_after_forward": true, "alternate_voicemail_enabled": true, "voicemail_local_after_email": false})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create vm: {b}");
    for key in [
        "voicemail_transcription_enabled",
        "voicemail_transcription_prompt",
        "voicemail_local_after_forward",
        "alternate_voicemail_enabled",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["voicemail_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/voicemails/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(
        g["voicemail_transcription_prompt"], "say it",
        "persisted: {g}"
    );
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/voicemails/{uuid}"),
        Some(serde_json::json!({"voicemail_transcription_enabled": false})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
}
