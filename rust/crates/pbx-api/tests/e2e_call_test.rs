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

async fn get_xml(app: axum::Router, token: &str, path: &str) -> (StatusCode, String) {
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

// Full extension-to-extension call flow (mock transport, no live FS):
// seed domain -> extensions 1001+1002 -> directory XML -> dialplan XML ->
// originate -> active_calls -> kill -> CDR NORMAL_CLEARING billsec>=1 -> stats.
#[tokio::test]
async fn e2e_extension_to_extension_call_flow_mock() {
    let token = login_token().await;
    let domain = "e2e-call.example.com";

    // seed extensions 1001 + 1002 via API (Bearer)
    for ext in ["1001", "1002"] {
        let (s, b) = post_json(
            pbx_api::test_app(),
            &token,
            "/api/v1/extensions",
            serde_json::json!({"extension": ext, "password": "1234", "domain": domain}),
        )
        .await;
        assert!(
            s == StatusCode::CREATED || s == StatusCode::OK || s == StatusCode::CONFLICT,
            "create {ext}: {s} {b}"
        );
    }

    // GET directory XML for 1001 contains password
    let (s, xml) = get_xml(
        pbx_api::test_app(),
        &token,
        &format!("/api/internal/freeswitch/xml?section=directory&key_value=1001@{domain}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "directory xml status: {xml}");
    assert!(xml.contains("1001"), "directory must contain user: {xml}");
    assert!(
        xml.contains("1234"),
        "directory must contain password: {xml}"
    );

    // GET dialplan contains bridge
    let (s, xml) = get_xml(
        pbx_api::test_app(),
        &token,
        "/api/internal/freeswitch/xml?section=dialplan&key_value=default",
    )
    .await;
    assert_eq!(s, StatusCode::OK, "dialplan xml status: {xml}");
    assert!(
        xml.contains("bridge"),
        "dialplan must contain bridge: {xml}"
    );

    // POST /api/v1/calls/originate {from:1001,to:1002}
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/calls/originate",
        serde_json::json!({"from": "1001", "to": "1002"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "originate: {b}");
    let call_id = b
        .get("call_id")
        .and_then(|c| c.as_str())
        .expect("originate must return call_id")
        .to_string();
    assert!(!call_id.is_empty());

    // GET /api/v1/active_calls contains channel
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/active_calls").await;
    assert_eq!(s, StatusCode::OK, "active_calls: {b}");
    assert!(
        b.to_string().contains(&call_id),
        "active_calls must contain channel {call_id}: {b}"
    );

    // kill -> channel gone (DELETE canonical; POST alias accepted)
    let app = pbx_api::test_app();
    let res = app
        .oneshot(auth(
            Request::delete(format!("/api/v1/calls/{call_id}"))
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::OK || res.status() == StatusCode::NO_CONTENT,
        "kill status: {}",
        res.status()
    );

    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/active_calls").await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        !b.to_string().contains(&call_id),
        "killed call must be gone: {b}"
    );

    // POST /api/v1/cdr entry NORMAL_CLEARING billsec>=1
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/cdr",
        serde_json::json!({
            "caller_id_number": "1001",
            "destination_number": "1002",
            "hangup_cause": "NORMAL_CLEARING",
            "billsec": 5,
            "duration": 8,
            "direction": "inbound",
        }),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK,
        "cdr create: {s} {b}"
    );
    let cdr_uuid = b
        .get("cdr_uuid")
        .and_then(|v| v.as_str())
        .expect("cdr create must return cdr_uuid")
        .to_string();
    assert!(!cdr_uuid.is_empty());

    // GET /api/v1/cdr/{uuid} shows NORMAL_CLEARING billsec>=1
    let (s, b) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/cdr/{cdr_uuid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "cdr detail: {b}");
    assert_eq!(
        b.get("hangup_cause").and_then(|v| v.as_str()).unwrap_or(""),
        "NORMAL_CLEARING",
        "hangup_cause: {b}"
    );
    assert!(
        b.get("billsec").and_then(|v| v.as_u64()).unwrap_or(0) >= 1,
        "billsec>=1: {b}"
    );

    // GET /api/v1/cdr/stats counts it
    let (s, b) = get_json(pbx_api::test_app(), &token, "/api/v1/cdr/stats").await;
    assert_eq!(s, StatusCode::OK, "cdr stats: {b}");
    assert!(
        b.get("total").and_then(|t| t.as_u64()).unwrap_or(0) >= 1,
        "stats total>=1: {b}"
    );
    let body = b.to_string();
    assert!(
        body.contains("NORMAL_CLEARING"),
        "stats must count NORMAL_CLEARING: {b}"
    );
}
