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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

// (1) call_block CRUD: POST + GET list {data,total}
#[tokio::test]
async fn call_block_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_block",
        serde_json::json!({"number":"+15551234567","enabled":true}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "call_block create: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/call_block").await;
    assert_eq!(s, StatusCode::OK, "call_block list: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );
    assert!(
        b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1,
        "{b}"
    );
}

// (2) bridges CRUD
#[tokio::test]
async fn bridges_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/bridges",
        serde_json::json!({"name":"br-test","destination":"1001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "bridges create: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/bridges").await;
    assert_eq!(s, StatusCode::OK, "bridges list: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );
}

// (3) destinations CRUD
#[tokio::test]
async fn destinations_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/destinations",
        serde_json::json!({"destination":"1001","action":"bridge"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "destinations create: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/destinations").await;
    assert_eq!(s, StatusCode::OK, "destinations list: {b}");
    assert!(
        b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1,
        "{b}"
    );
}

// (4) follow_me CRUD
#[tokio::test]
async fn follow_me_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/follow_me",
        serde_json::json!({"extension":"1001","destinations":["1002","1003"]}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "follow_me create: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/follow_me").await;
    assert_eq!(s, StatusCode::OK, "follow_me list: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );
}

// (5) active_calls list (mock ESL) {data,total}
#[tokio::test]
async fn active_calls_list_from_mock_esl() {
    let token = login_token().await;
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/active_calls").await;
    assert_eq!(s, StatusCode::OK, "active_calls: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );
    assert!(b.get("total").is_some(), "total required: {b}");
}

// (6) registrations list (mock ESL) {data,total}
#[tokio::test]
async fn registrations_list_from_mock_esl() {
    let token = login_token().await;
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/registrations").await;
    assert_eq!(s, StatusCode::OK, "registrations: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {b}"
    );
    assert!(b.get("total").is_some(), "total required: {b}");
}

// (7) operator_panel summary
#[tokio::test]
async fn operator_panel_summary() {
    let token = login_token().await;
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/operator_panel").await;
    assert_eq!(s, StatusCode::OK, "operator_panel: {b}");
    let body = b.to_string();
    assert!(
        body.contains("active_calls") || body.contains("channels") || body.contains("total"),
        "summary must carry counts: {b}"
    );
}

// (9) required-field validation: empty number rejected 422
#[tokio::test]
async fn call_block_rejects_empty_number() {
    let token = login_token().await;
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_block",
        serde_json::json!({"number":""}),
    )
    .await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
}
#[tokio::test]
async fn click_to_call_originate_shows_in_channels() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/click_to_call",
        serde_json::json!({"extension":"1001","number":"1002"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK,
        "click_to_call: {b}"
    );
    let call_id = b
        .get("call_id")
        .or_else(|| b.get("data").and_then(|d| d.get("call_id")))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    assert!(!call_id.is_empty(), "must return call_id: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/calls/channels").await;
    assert_eq!(s, StatusCode::OK, "channels: {b}");
    assert!(
        b.to_string().contains(&call_id) || b.to_string().contains("1001"),
        "channels must show click_to_call call: {b}"
    );
}
