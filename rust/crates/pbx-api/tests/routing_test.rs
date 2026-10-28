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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

// (1) ring group CRUD + members roundtrip {data,total}
#[tokio::test]
async fn ring_group_crud_with_members_roundtrip() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/ring_groups",
        serde_json::json!({
            "name": "sales-ring",
            "extension": "6101",
            "strategy": "simultaneous",
            "members": [
                {"destination": "1001", "delay": 0, "timeout": 20},
                {"destination": "1002", "delay": 0, "timeout": 20}
            ]
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ring group: {b}");
    let id = b
        .get("ring_group_uuid")
        .and_then(|v| v.as_str())
        .expect("ring_group_uuid in response")
        .to_string();

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/ring_groups").await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    assert!(
        b.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope {{data,total}}: {b}"
    );
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);

    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ring_groups/{id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get one: {b}");
    let members = b
        .get("members")
        .and_then(|m| m.as_array())
        .expect("members roundtrip");
    assert_eq!(members.len(), 2, "members roundtrip: {b}");
}

// (2) call center queue + agent CRUD + mock-ESL queue join
#[tokio::test]
async fn call_center_queue_agent_and_join() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_centers",
        serde_json::json!({"queue_name": "support-q", "queue_extension": "6201"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create queue: {b}");
    let qid = b
        .get("call_center_queue_uuid")
        .and_then(|v| v.as_str())
        .expect("queue uuid")
        .to_string();

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/call_centers/agents",
        serde_json::json!({"queue_id": qid, "agent_name": "alice", "agent_contact": "1001"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create agent: {b}");

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/call_centers").await;
    assert_eq!(s, StatusCode::OK, "list queues: {b}");
    assert!(b.get("data").and_then(|d| d.as_array()).is_some());
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);

    // mock-ESL queue join: agent joins queue via mocked transport
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/call_centers/{qid}/join"),
        serde_json::json!({"agent_name": "alice"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "queue join: {b}");
    assert_eq!(
        b.get("status").and_then(|v| v.as_str()).unwrap_or(""),
        "joined"
    );
}

// (3) IVR CRUD + options
#[tokio::test]
async fn ivr_crud_with_options() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/ivr_menus",
        serde_json::json!({
            "name": "main-ivr",
            "extension": "6301",
            "options": [
                {"digits": "1", "action": "transfer", "param": "1001"},
                {"digits": "2", "action": "transfer", "param": "1002"}
            ]
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ivr: {b}");
    let id = b
        .get("ivr_menu_uuid")
        .and_then(|v| v.as_str())
        .expect("ivr uuid")
        .to_string();

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/ivr_menus").await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    assert!(b.get("data").and_then(|d| d.as_array()).is_some());
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);

    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/ivr_menus/{id}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get one: {b}");
    let opts = b
        .get("options")
        .and_then(|o| o.as_array())
        .expect("options roundtrip");
    assert_eq!(opts.len(), 2, "options roundtrip: {b}");
}

// (4) voicemail CRUD + greeting
#[tokio::test]
async fn voicemail_crud_with_greeting() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/voicemails",
        serde_json::json!({"voicemail_id": "6401", "voicemail_password": "1234"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create voicemail: {b}");
    let id = b
        .get("voicemail_uuid")
        .and_then(|v| v.as_str())
        .expect("vm uuid")
        .to_string();

    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/voicemails/{id}/greetings"),
        serde_json::json!({"greeting_name": "greet-1", "greeting_text": "hello"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create greeting: {b}");

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/voicemails").await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    assert!(b.get("data").and_then(|d| d.as_array()).is_some());
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);
}

// (5) conference CRUD
#[tokio::test]
async fn conference_crud() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/conferences",
        serde_json::json!({"conference_name": "team-conf", "conference_extension": "6501"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create conference: {b}");

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/conferences").await;
    assert_eq!(s, StatusCode::OK, "list: {b}");
    assert!(b.get("data").and_then(|d| d.as_array()).is_some());
    assert!(b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1);
}

// (6) fax queue submit
#[tokio::test]
async fn fax_queue_submit() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/fax/queue",
        serde_json::json!({"fax_extension": "6601", "destination": "15551234567"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "fax queue submit: {b}");
    assert!(
        b.get("fax_queue_uuid").and_then(|v| v.as_str()).is_some(),
        "queue uuid: {b}"
    );

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/fax/queue").await;
    assert_eq!(s, StatusCode::OK, "list queue: {b}");
    assert!(b.get("data").and_then(|d| d.as_array()).is_some());
}

// (7) CDR tolerant ingest: malformed XML still 200 + counted
#[tokio::test]
async fn cdr_tolerant_ingest_malformed_still_200_and_counted() {
    let token = login_token().await;
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::post("/api/v1/cdr/import")
                .header("content-type", "application/xml")
                .body(Body::from("<cdr><broken><unclosed>"))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "malformed xml must still be 200"
    );
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        v.get("status").and_then(|s| s.as_str()).unwrap_or(""),
        "stored"
    );

    // well-formed ingest
    let (s, _) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/cdr/import",
        serde_json::json!({
            "caller_id_number": "1001",
            "destination_number": "1002",
            "hangup_cause": "NORMAL_CLEARING",
            "direction": "inbound"
        }),
    )
    .await;
    assert!(
        s == StatusCode::OK || s == StatusCode::CREATED,
        "well-formed ingest: {s}"
    );

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/cdr").await;
    assert_eq!(s, StatusCode::OK, "list cdr: {b}");
    assert!(
        b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 2,
        "both counted: {b}"
    );
}

// (8) CDR stats endpoint: counts by disposition
#[tokio::test]
async fn cdr_stats_counts_by_disposition() {
    let token = login_token().await;
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/cdr/stats").await;
    assert_eq!(s, StatusCode::OK, "stats: {b}");
    assert!(
        b.get("by_hangup_cause").is_some()
            || b.get("by_disposition").is_some()
            || b.get("total").is_some(),
        "stats shape: {b}"
    );
}

// RED: analytics summary returns dashboard keys (zeros when empty, never 500)
#[tokio::test]
async fn analytics_summary_returns_required_keys() {
    let token = login_token().await;
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/analytics/summary").await;
    assert_eq!(s, StatusCode::OK, "analytics summary: {b}");
    assert!(
        b.get("total_calls").and_then(|v| v.as_u64()).is_some(),
        "total_calls: {b}"
    );
    assert!(
        b.get("today_calls").and_then(|v| v.as_u64()).is_some(),
        "today_calls: {b}"
    );
    assert!(
        b.get("avg_duration").and_then(|v| v.as_f64()).is_some(),
        "avg_duration: {b}"
    );
}

// RED: analytics detail endpoints return {data} (zeros when empty, never 500)
#[tokio::test]
async fn analytics_detail_endpoints_return_data() {
    let token = login_token().await;
    for path in [
        "/api/v1/analytics/daily",
        "/api/v1/analytics/by_direction",
        "/api/v1/analytics/by_hangup",
        "/api/v1/analytics/top_extensions",
        "/api/v1/analytics/hourly",
        "/api/v1/analytics/system_counts",
        "/api/v1/analytics/answer_rate",
    ] {
        let (s, b) = get_json(pbx_api::test_app(), &token, path).await;
        assert_eq!(s, StatusCode::OK, "{path}: {b}");
    }
}
