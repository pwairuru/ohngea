//! Trunk/dialplan parity: Gateway + Dialplan serializers must expose full
//! entity fields (PHP `gateway_edit.php` ~36 inputs, `dialplan_edit.php`),
//! not 7/5-field trims.

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

#[tokio::test]
async fn gateway_create_returns_full_fields() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/gateways",
        serde_json::json!({
            "gateway": "gw-parity",
            "username": "gwuser",
            "password": "gwpw",
            "proxy": "sip.example.com",
            "register": true,
            "auth_username": "authu",
            "realm": "example.com",
            "from_user": "gwuser",
            "expire_seconds": 3600,
            "retry_seconds": 30,
            "ping": "25",
            "context": "public",
            "profile": "external",
            "description": "parity gw"
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create gw: {b}");
    for key in [
        "proxy",
        "auth_username",
        "realm",
        "from_user",
        "expire_seconds",
        "retry_seconds",
        "ping",
        "profile",
        "description",
    ] {
        assert!(b.get(key).is_some(), "create must echo `{key}`, got {b}");
    }
    assert_eq!(b["expire_seconds"], 3600);
    let uuid = b["gateway_uuid"].as_str().unwrap().to_string();
    let (s, g) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/gateways/{uuid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get gw: {g}");
    assert_eq!(g["realm"], "example.com", "persisted: {g}");
    assert_eq!(g["proxy"], "sip.example.com");
}

#[tokio::test]
async fn dialplan_create_returns_full_fields() {
    let token = login_token().await;
    let (s, b) = post_json(
        pbx_api::test_app(),
        &token,
        "/api/v1/dialplans",
        serde_json::json!({
            "name": "dp-parity",
            "context": "public",
            "enabled": true,
            "description": "parity dp",
            "dialplan_continue": true,
            "dialplan_order": 100,
            "dialplan_number": "999",
            "hostname": "fs1"
        }),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create dp: {b}");
    for key in [
        "description",
        "dialplan_continue",
        "dialplan_order",
        "dialplan_number",
        "hostname",
    ] {
        assert!(b.get(key).is_some(), "create must echo `{key}`, got {b}");
    }
    let uuid = b["dialplan_uuid"].as_str().unwrap().to_string();
    let (s, g) = get_json(
        pbx_api::test_app(),
        &token,
        &format!("/api/v1/dialplans/{uuid}"),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get dp: {g}");
    assert_eq!(g["dialplan_number"], "999", "persisted: {g}");
}
