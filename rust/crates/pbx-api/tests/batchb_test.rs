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

async fn get_json(app: axum::Router, token: &str, path: &str) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

// 1. call_center_active live view
#[tokio::test]
async fn batchb_call_center_active_live() {
    let token = login_token().await;
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/call_center_active").await;
    assert_eq!(s, StatusCode::OK, "cca list: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "data array: {b}"
    );
    assert!(
        b.get("total").and_then(|t| t.as_u64()).is_some(),
        "total: {b}"
    );
    assert!(
        b.get("source").and_then(|v| v.as_str()).is_some(),
        "source esl|db: {b}"
    );
}

// 2. broadcast send/stop
#[tokio::test]
async fn batchb_broadcast_send_stop() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_broadcasts",
        serde_json::json!({
            "broadcast_name": "batchb-send",
            "broadcast_destination_type": "extension",
            "broadcast_destination_data": "1001",
            "broadcast_phone_numbers": "1001,1002",
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create bc: {b}");
    let id = b
        .get("call_broadcast_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!id.is_empty(), "uuid: {b}");

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_broadcasts/{id}/send"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "send: {b}");
    assert!(b.get("send").is_some(), "send envelope: {b}");

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_broadcasts/{id}/stop"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "stop: {b}");
    assert_eq!(
        b.get("broadcast_enabled").and_then(|v| v.as_bool()),
        Some(false),
        "disabled after stop: {b}"
    );
}

// 3a. active calls get + hangup
#[tokio::test]
async fn batchb_active_calls_hangup() {
    let token = login_token().await;
    // originate a call so there is something live
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/calls/originate",
        serde_json::json!({"from":"1001","to":"1002"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "originate: {b}");
    let call_id = b
        .get("call_id")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!call_id.is_empty());

    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/active_calls/{call_id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get active call: {b}");

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/active_calls/{call_id}/hangup"),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "hangup: {b}");
}

// 3b. conferences_active member control
#[tokio::test]
async fn batchb_active_conferences_member_control() {
    let token = login_token().await;
    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/active_conferences/count",
    )
    .await;
    assert_eq!(s, StatusCode::OK, "count: {b}");
    assert!(
        b.get("count").and_then(|c| c.as_u64()).is_some(),
        "count field: {b}"
    );

    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/active_conferences/demo/members",
    )
    .await;
    assert_eq!(s, StatusCode::OK, "members: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "members data: {b}"
    );

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/active_conferences/demo/members/1/mute",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "mute: {b}");
    assert_eq!(
        b.get("action").and_then(|v| v.as_str()),
        Some("mute"),
        "action echo: {b}"
    );

    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/active_conferences/demo/members/1/bogus",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY, "bad action 422");
}

// 4. conference_controls + profiles CRUD
#[tokio::test]
async fn batchb_conference_controls_profiles_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/conference_controls",
        serde_json::json!({"control_name": "batchb-ctrl"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ctrl: {b}");
    let id = b
        .get("conference_control_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!id.is_empty(), "ctrl uuid: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/conference_controls").await;
    assert_eq!(s, StatusCode::OK, "list ctrls: {b}");
    assert!(
        b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1,
        "total: {b}"
    );

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/conference_profiles",
        serde_json::json!({"profile_name": "batchb-prof"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create prof: {b}");
    let pid = b
        .get("conference_profile_uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!pid.is_empty(), "prof uuid: {b}");
    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/conference_profiles/{pid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get prof: {b}");
}
