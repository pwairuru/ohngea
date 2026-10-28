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

fn other_tenant_token() -> String {
    pbx_core::auth::encode_claims(
        "00000000-0000-0000-0000-000000000099",
        "other-tenant",
        "admin",
        &["admin".to_string()],
    )
}

fn rand_suffix() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.subsec_nanos() % 9000000) + 1000000)
        .unwrap_or(1000001)
}

// ---- spillover replacement RED: per-table entities with real columns ----
// These assert Python-model column names (not generic JSON spillover).
// Pre-migration they FAIL (422 missing legacy `number`/`name` keys, no
// `follow_me_destination` objects, no PATCH route); post-migration GREEN.

// (10) call_block per-table: real columns + legacy `number` alias accepted
#[tokio::test]
async fn call_block_per_table_real_columns() {
    let token = login_token().await;
    let num = format!("+1555{:07}", rand_suffix());
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_block",
        serde_json::json!({"call_block_number": num, "call_block_name": "spam-per-table"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "call_block create: {b}");
    assert!(
        b.get("call_block_uuid").and_then(|v| v.as_str()).is_some(),
        "real PK missing: {b}"
    );
    assert_eq!(
        b.get("call_block_number").and_then(|v| v.as_str()),
        Some(num.as_str()),
        "real column missing: {b}"
    );
    let id = b["call_block_uuid"].as_str().unwrap().to_string();
    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_block/{id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "call_block get one: {b}");
    assert_eq!(
        b.get("call_block_number").and_then(|v| v.as_str()),
        Some(num.as_str()),
        "get one real column: {b}"
    );
}

// (11) bridges per-table: real columns
#[tokio::test]
async fn bridges_per_table_real_columns() {
    let token = login_token().await;
    let name = format!("br-pt-{}", rand_suffix());
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/bridges",
        serde_json::json!({"bridge_name": name, "bridge_destination": "1001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "bridges create: {b}");
    assert!(
        b.get("bridge_uuid").and_then(|v| v.as_str()).is_some(),
        "real PK missing: {b}"
    );
    assert_eq!(
        b.get("bridge_name").and_then(|v| v.as_str()),
        Some(name.as_str()),
        "real column missing: {b}"
    );
}

// (12) destinations per-table: real columns
#[tokio::test]
async fn destinations_per_table_real_columns() {
    let token = login_token().await;
    let num = format!("2{:06}", rand_suffix() % 1000000);
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/destinations",
        serde_json::json!({"destination_number": num, "destination_type": "inbound"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "destinations create: {b}");
    assert!(
        b.get("destination_uuid").and_then(|v| v.as_str()).is_some(),
        "real PK missing: {b}"
    );
    assert_eq!(
        b.get("destination_number").and_then(|v| v.as_str()),
        Some(num.as_str()),
        "real column missing: {b}"
    );
}

// (13) follow_me per-table: destinations roundtrip as objects
#[tokio::test]
async fn follow_me_per_table_destinations_roundtrip() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/follow_me",
        serde_json::json!({"extension": "1001", "follow_me_enabled": true, "destinations": ["1002", "1003"]}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "follow_me create: {b}");
    let id = b
        .get("follow_me_uuid")
        .and_then(|v| v.as_str())
        .expect("follow_me_uuid missing")
        .to_string();
    let dests = b
        .get("destinations")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert_eq!(dests.len(), 2, "destinations roundtrip: {b}");
    assert!(
        dests
            .iter()
            .all(|d| d.get("follow_me_destination").is_some()),
        "destination objects need follow_me_destination: {b}"
    );
    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/follow_me/{id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "follow_me get one: {b}");
    assert_eq!(
        b.get("destinations")
            .and_then(|d| d.as_array())
            .map(|a| a.len()),
        Some(2),
        "get one destinations: {b}"
    );
}

// (14) time_conditions per-table: real columns + PATCH update
#[tokio::test]
async fn time_conditions_per_table_patch() {
    let token = login_token().await;
    let name = format!("office-hours-{}", rand_suffix());
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/time_conditions",
        serde_json::json!({"name": name, "extension": "7001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "time_conditions create: {b}");
    assert_eq!(
        b.get("name").and_then(|v| v.as_str()),
        Some(name.as_str()),
        "real column name: {b}"
    );
    assert_eq!(
        b.get("extension").and_then(|v| v.as_str()),
        Some("7001"),
        "real column extension: {b}"
    );
    let id = b
        .get("time_condition_uuid")
        .and_then(|v| v.as_str())
        .expect("time_condition_uuid missing")
        .to_string();
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::patch(format!("/api/v1/time_conditions/{id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"description": "9-5 weekdays"}).to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK, "PATCH time_condition");
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        v.get("description").and_then(|x| x.as_str()),
        Some("9-5 weekdays"),
        "patched column: {v}"
    );
}

// (15) cross-tenant invisibility: other domain sees none of our call_block rows
#[tokio::test]
async fn call_block_cross_tenant_invisible() {
    let token = login_token().await;
    let other = other_tenant_token();
    let num = format!("+1555{:07}", rand_suffix());
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_block",
        serde_json::json!({"call_block_number": num, "call_block_name": "tenant-a-only"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create tenant A: {b}");
    let id = b["call_block_uuid"].as_str().unwrap_or("").to_string();
    assert!(!id.is_empty(), "uuid: {b}");
    let (s, b) = get_json(pbx_api::test_app(), &other, "/api/v1/call_block").await;
    assert_eq!(s, StatusCode::OK, "other tenant list: {b}");
    let data = b
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        data.iter()
            .all(|r| r.get("call_block_uuid").and_then(|v| v.as_str()) != Some(id.as_str())),
        "tenant B must not see tenant A row: {b}"
    );
    let (s, _) = get_json(
        pbx_api::test_app(),
        &other,
        &format!("/api/v1/call_block/{id}"),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "tenant B get one must 404");
}
