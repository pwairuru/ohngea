#![recursion_limit = "256"]
//! Extension full-field parity: PHP `extension_edit.php` exposes ~45 inputs
//! (password, caller IDs, directory, limits, forwarding/DND, advanced SIP…).
//! The API must accept, persist, and return them all — not just the 9-field
//! legacy shape.

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

async fn post_json(
    app: axum::Router,
    token: &str,
    path: &str,
    v: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(auth(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(v.to_string()))
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

async fn get_json(app: axum::Router, token: &str, path: &str) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn patch_json(
    app: axum::Router,
    token: &str,
    path: &str,
    v: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(auth(
            Request::patch(path)
                .header("content-type", "application/json")
                .body(Body::from(v.to_string()))
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
async fn extension_create_upserts_linked_voicemail() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/extensions",
        serde_json::json!({"extension": "7201", "password": "pw123456", "effective_caller_id_name": "VM User", "voicemail_password": "vm1234", "voicemail_enabled": true, "voicemail_transcription_enabled": true}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create: {b}");
    let (s, v) = get_json(pbx_api::test_app(), &token, "/api/v1/voicemails").await;
    assert_eq!(s, StatusCode::OK, "list vm: {v}");
    let found = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["voicemail_id"] == "7201");
    assert!(found, "linked mailbox created: {v}");
}

#[tokio::test]
async fn extension_create_accepts_and_returns_full_field_set() {
    let token = login_token().await;
    let payload = serde_json::json!({
        "extension": "7101",
        "password": "s3cret-pw",
        "number_alias": "7101",
        "accountcode": "acct-1",
        "effective_caller_id_name": "Parity User",
        "effective_caller_id_number": "7101",
        "outbound_caller_id_name": "Out Name",
        "outbound_caller_id_number": "15550001",
        "emergency_caller_id_name": "Emg Name",
        "emergency_caller_id_number": "911",
        "directory_first_name": "Parity",
        "directory_last_name": "User",
        "directory_visible": true,
        "directory_exten_visible": true,
        "max_registrations": "3",
        "limit_max": "5",
        "limit_destination": "!USER_BUSY",
        "missed_call_app": "email",
        "missed_call_data": "a@b.c",
        "toll_allow": "domestic",
        "call_timeout": 45,
        "call_group": "sales",
        "call_screen_enabled": true,
        "user_record": "all",
        "hold_music": "default",
        "auth_acl": "domains",
        "cidr": "10.0.0.0/8",
        "sip_force_contact": "NDLB-connectile-dysfunction",
        "sip_force_expires": 90,
        "nibble_account": "7101",
        "mwi_account": "7101@v",
        "sip_bypass_media": "bypass-media",
        "absolute_codec_string": "PCMU,PCMA",
        "force_ping": true,
        "dial_string": "{leg_timeout=45}user/7101",
        "do_not_disturb": true,
        "forward_all_destination": "7201",
        "forward_all_enabled": true,
        "forward_busy_destination": "7202",
        "forward_busy_enabled": true,
        "forward_no_answer_destination": "7203",
        "forward_no_answer_enabled": true,
        "forward_user_not_registered_destination": "7204",
        "forward_user_not_registered_enabled": true,
        "follow_me_enabled": false,
        "extension_language": "en",
        "extension_dialect": "us",
        "extension_voice": "callie",
        "extension_type": "default",
        "user_context": "example.com",
        "enabled": true,
        "description": "parity probe"
    });
    let (s, b) = post_json(pbx_api::test_app(), &token, "/api/v1/extensions", payload).await;
    assert_eq!(s, StatusCode::CREATED, "create full: {b}");
    for key in [
        "password",
        "number_alias",
        "accountcode",
        "outbound_caller_id_name",
        "outbound_caller_id_number",
        "directory_first_name",
        "call_timeout",
        "call_group",
        "toll_allow",
        "forward_all_destination",
        "forward_all_enabled",
        "do_not_disturb",
        "sip_bypass_media",
        "description",
    ] {
        assert!(
            b.get(key).is_some(),
            "create response must echo `{key}`, got {b}"
        );
    }
    assert_eq!(b["call_timeout"], 45);
    assert_eq!(b["forward_all_enabled"], true);
    assert_eq!(b["do_not_disturb"], true);

    let uuid = b["extension_uuid"].as_str().unwrap().to_string();
    let (s, g) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/extensions/{uuid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(
        g["outbound_caller_id_number"], "15550001",
        "get persists: {g}"
    );
    assert_eq!(g["call_group"], "sales");
    assert_eq!(g["forward_busy_destination"], "7202");

    // PATCH a different subset, verify persistence.
    let (s, p) = patch_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/extensions/{uuid}"),
        serde_json::json!({"call_timeout": 20, "toll_allow": "intl", "do_not_disturb": false}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["call_timeout"], 20, "patch echoes: {p}");
    let (s, g2) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/extensions/{uuid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(g2["toll_allow"], "intl", "patch persists: {g2}");
}
