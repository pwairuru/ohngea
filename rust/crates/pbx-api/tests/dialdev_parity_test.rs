//! Dialplan details + device full-field parity: Python includes `details[]`
//! (condition/action rules) on list/get/create; PHP device_edit has
//! template/location/serial/firmware/credentials that the Rust API drops.

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
        "DELETE" => Request::delete(path),
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
async fn dialplan_details_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/dialplans",
        Some(serde_json::json!({
            "name": "dp-det",
            "context": "public",
            "dialplan_number": "888",
            "details": [
                {"tag": "condition", "type": "destination_number", "data": "^888$"},
                {"tag": "action", "type": "transfer", "data": "1000 XML default"}
            ]
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create dp: {b}");
    let details = b
        .get("details")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert_eq!(details.len(), 2, "create echoes details: {b}");
    let uuid = b["dialplan_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/dialplans/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    let gd = g
        .get("details")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    assert_eq!(gd.len(), 2, "get persists details: {g}");
    assert_eq!(gd[0]["data"], "^888$");
}

#[tokio::test]
async fn device_full_fields_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/devices",
        Some(serde_json::json!({
            "mac": "00:11:22:33:44:99",
            "vendor": "yealink",
            "model": "T46U",
            "label": "Desk",
            "device_template": "yealink/T46U",
            "device_location": "HQ",
            "device_serial_number": "SN123",
            "device_firmware_version": "108.86.0.5",
            "device_description": "front desk",
            "device_username": "admin",
            "device_password": "secret"
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create dev: {b}");
    for key in [
        "device_template",
        "device_location",
        "device_serial_number",
        "device_description",
        "device_username",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["device_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/devices/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(g["device_location"], "HQ", "persisted: {g}");
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/devices/{uuid}"),
        Some(serde_json::json!({"device_location": "Branch", "device_template": "yealink/T48U"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["device_location"], "Branch", "patch echoes: {p}");
}
