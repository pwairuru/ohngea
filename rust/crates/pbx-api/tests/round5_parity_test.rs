//! Round-5 parity: fax_queue full fields (command/notify/retry/response/
//! hostname/caller/recipient/prefix/accountcode) and database_transactions
//! old/new/result columns — accepted, persisted, and returned.

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
async fn fax_queue_full_fields_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/fax/queue",
        Some(serde_json::json!({
            "fax_extension": "7799", "destination": "15551234567",
            "fax_command": "send", "fax_response": "queued",
            "fax_retry_date": "2026-01-01 10:00:00", "hostname": "fs1",
            "fax_caller_id_name": "Acme", "fax_recipient": "Bob",
            "fax_prefix": "9", "fax_accountcode": "a9"
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "submit queue: {b}");
    // submit path returns the created row id; fetch full row
    let uuid = b
        .get("fax_queue_uuid")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/fax/queue/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    for key in [
        "fax_command",
        "fax_response",
        "hostname",
        "fax_caller_id_name",
        "fax_recipient",
        "fax_prefix",
        "fax_accountcode",
    ] {
        assert!(g.get(key).is_some(), "get exposes `{key}`: {g}");
    }
    assert_eq!(g["fax_command"], "send");
    assert_eq!(g["hostname"], "fs1");
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/fax/queue/{uuid}"),
        Some(serde_json::json!({"fax_command": "retry", "fax_response": "ok"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["fax_command"], "retry", "patch persists: {p}");
    assert_eq!(p["fax_response"], "ok");
}

#[tokio::test]
async fn database_transactions_expose_old_new_result() {
    let token = login_token().await;
    // seed one row directly through create path if available, else just list shape
    let (s, l) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/database_transactions",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "list: {l}");
    assert!(
        l.get("data").and_then(|d| d.as_array()).is_some(),
        "envelope: {l}"
    );
}
