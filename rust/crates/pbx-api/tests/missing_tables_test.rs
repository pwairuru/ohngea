//! Parity tests for the previously-missing PHP tables + alias layer.
//! Ground truth: `fusionpbx/app/*` (PHP). Every route here must answer 200
//! with `{data,total}` (or 201 + row echo on POST) under tenant auth.

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

async fn get_json(path: &str, token: &str) -> (StatusCode, serde_json::Value) {
    let res = pbx_api::test_app()
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let s = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (s, v)
}

async fn post_json(
    path: &str,
    token: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let res = pbx_api::test_app()
        .oneshot(auth(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
            token,
        ))
        .await
        .unwrap();
    let s = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
    (s, v)
}

#[tokio::test]
async fn missing_tables_list_200_with_envelope() {
    let token = login_token().await;
    for p in [
        "/api/v1/conference_room_users",
        "/api/v1/extension_users",
        "/api/v1/ring_group_users",
        "/api/v1/voicemail_destinations",
        "/api/v1/device_vendors",
        "/api/v1/database_transactions",
        "/api/v1/email_queue_attachments",
        "/api/v1/xml_cdr_extensions",
        "/api/v1/xml_cdr_json",
        "/api/v1/xml_cdr_flow",
        "/api/v1/xml_cdr_logs",
        "/api/v1/xml_cdr_transcripts",
        "/api/v1/conference_control_details",
        "/api/v1/conference_profile_params",
    ] {
        let (s, v) = get_json(p, &token).await;
        assert_eq!(s, StatusCode::OK, "GET {p} -> {s}");
        assert!(v.get("data").is_some(), "GET {p} has data envelope");
        assert!(v.get("total").is_some(), "GET {p} has total");
    }
}

#[tokio::test]
async fn missing_tables_crud_roundtrip() {
    let token = login_token().await;
    // extension_users
    let (s, v) = post_json(
        "/api/v1/extension_users",
        &token,
        serde_json::json!({"extension_uuid": "ext-1", "user_uuid": "usr-1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create extension_users: {v}");
    assert!(v["extension_user_uuid"].as_str().is_some());
    // ring_group_users
    let (s, v) = post_json(
        "/api/v1/ring_group_users",
        &token,
        serde_json::json!({"ring_group_uuid": "rg-1", "user_uuid": "usr-1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ring_group_users: {v}");
    // voicemail_destinations
    let (s, v) = post_json(
        "/api/v1/voicemail_destinations",
        &token,
        serde_json::json!({"voicemail_uuid": "vm-1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create voicemail_destinations: {v}");
    // device_vendors
    let (s, v) = post_json(
        "/api/v1/device_vendors",
        &token,
        serde_json::json!({"name": "test-vendor"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create device_vendors: {v}");
    // conference_room_users
    let (s, v) = post_json(
        "/api/v1/conference_room_users",
        &token,
        serde_json::json!({"conference_room_uuid": "cr-1", "user_uuid": "usr-1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create conference_room_users: {v}");
    // control detail + profile param (PHP detail tables)
    let (s, v) = post_json(
        "/api/v1/conference_control_details",
        &token,
        serde_json::json!({"conference_control_uuid": "cc-1", "control_digits": "5", "control_action": "mute"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create control detail: {v}");
    assert_eq!(v["control_digits"], "5");
    let (s, v) = post_json(
        "/api/v1/conference_profile_params",
        &token,
        serde_json::json!({"conference_profile_uuid": "cp-1", "profile_param_name": "energy-level"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create profile param: {v}");
    assert_eq!(v["profile_param_name"], "energy-level");
}

#[tokio::test]
async fn php_alias_patch_accepts_queue_prefix() {
    let token = login_token().await;
    // create a queue, then PATCH using PHP-style queue_* names
    let (s, v) = post_json(
        "/api/v1/call_centers",
        &token,
        serde_json::json!({"queue_name": "alias-q", "queue_strategy": "longest-idle-agent"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create queue: {v}");
    let qid = v["call_center_queue_uuid"].as_str().unwrap().to_owned();
    let res = pbx_api::test_app()
        .oneshot(auth(
            Request::patch(format!("/api/v1/call_centers/{qid}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"queue_greeting": "greet.wav", "queue_max_wait_time": 42})
                        .to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["greeting"], "greet.wav", "alias mapped, got {v}");
    assert_eq!(v["max_wait_time"], 42);
    assert_eq!(v["queue_greeting"], "greet.wav", "alias echoed, got {v}");
}

#[tokio::test]
async fn php_alias_patch_accepts_conference_prefix() {
    let token = login_token().await;
    let (s, v) = post_json(
        "/api/v1/conferences",
        &token,
        serde_json::json!({"conference_name": "alias-conf", "conference_extension": "7111"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create conference: {v}");
    let cid = v["conference_uuid"].as_str().unwrap().to_owned();
    let res = pbx_api::test_app()
        .oneshot(auth(
            Request::patch(format!("/api/v1/conferences/{cid}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"conference_pin_number": "9999"}).to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["pin_number"], "9999", "alias mapped, got {v}");
}
