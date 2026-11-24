//! Final-row parity: queue_record_enabled round-trips through the record
//! template (PHP gating checkbox has no column), time_conditions design
//! divergence is covered by API (dialplan builder writes v_dialplans).

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
async fn queue_record_enabled_roundtrip() {
    let token = login_token().await;
    // enabled with template -> echoed true, template stored
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/call_centers",
        Some(serde_json::json!({"queue_name": "qr1", "queue_extension": "7701", "queue_record_enabled": true, "queue_record_template": "tpl.wav"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create: {b}");
    assert_eq!(b["queue_record_enabled"], true);
    assert_eq!(b["record_template"], "tpl.wav");
    let uuid = b["call_center_queue_uuid"].as_str().unwrap().to_string();
    // disabling clears the template
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/call_centers/{uuid}"),
        Some(serde_json::json!({"queue_record_enabled": false})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["queue_record_enabled"], false, "echoes: {p}");
    assert!(
        p.get("record_template").is_none() || p["record_template"].is_null(),
        "template cleared: {p}"
    );
}
