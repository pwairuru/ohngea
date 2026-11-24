//! Round-3 parity: recordings description column + filename aliases,
//! SIP profile network fields (ext IPs/realm/context/hostname),
//! conference-room create full fields.

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
async fn recordings_description_and_filename_alias() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/recordings",
        Some(serde_json::json!({"recording_name": "greet", "recording_filename": "greet.wav", "recording_description": "main greeting", "recording_voice": "callie"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create rec: {b}");
    for key in [
        "recording_description",
        "recording_filename",
        "recording_name",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["recording_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/recordings/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(
        g["recording_description"], "main greeting",
        "persisted: {g}"
    );
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/recordings/{uuid}"),
        Some(serde_json::json!({"recording_description": "v2"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["recording_description"], "v2");
}

#[tokio::test]
async fn sip_profile_network_fields_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/sip_profiles",
        Some(serde_json::json!({"name": "t recoil", "ext_sip_ip": "10.0.0.1", "ext_rtp_ip": "10.0.0.1", "realm": "10.0.0.1", "context": "public", "description": "test prof", "hostname": "fs1"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create sip: {b}");
    for key in [
        "ext_sip_ip",
        "ext_rtp_ip",
        "realm",
        "context",
        "description",
        "hostname",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["sip_profile_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/sip_profiles/{uuid}"),
        Some(serde_json::json!({"realm": "10.0.0.2"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["realm"], "10.0.0.2");
}

#[tokio::test]
async fn conference_room_create_full_fields() {
    let token = login_token().await;
    let (s, c) = call(pbx_api::test_app(), &token, "POST", "/api/v1/conference_centers", Some(serde_json::json!({"conference_center_name": "CC3", "conference_center_extension": "3800"}))).await;
    assert_eq!(s, StatusCode::CREATED, "create center: {c}");
    let cid = c["conference_center_uuid"].as_str().unwrap().to_string();
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", &format!("/api/v1/conference_centers/{cid}/rooms"),
        Some(serde_json::json!({"conference_room_name": "Board", "moderator_endconf": true, "announce_recording": true, "mute": true, "account_code": "rm-acct", "start_datetime": "2026-01-01T10:00:00Z", "sounds": "conf-sounds", "max_members": 20})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create room: {b}");
    for key in [
        "moderator_endconf",
        "announce_recording",
        "mute",
        "account_code",
        "sounds",
        "max_members",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    assert_eq!(b["moderator_endconf"], true);
}
