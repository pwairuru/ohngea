//! Round-4 parity: modules table (was stub + unrouted), email_response
//! column, bridge destination composition from action/number/profile.

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
async fn modules_full_crud() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/modules",
        Some(serde_json::json!({"module_name": "mod_soup", "module_label": "Soup", "module_category": "Endpoints", "module_enabled": true, "module_default_enabled": false, "module_description": "test mod"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create module: {b}");
    for key in [
        "module_label",
        "module_category",
        "module_enabled",
        "module_default_enabled",
        "module_description",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    assert_eq!(b["module_enabled"], true);
    let uuid = b["module_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/modules/{uuid}"),
        Some(serde_json::json!({"module_enabled": false})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["module_enabled"], false);
    let (s, _) = call(
        pbx_api::test_app(),
        &token,
        "DELETE",
        &format!("/api/v1/modules/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn email_response_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/email_queue",
        Some(
            serde_json::json!({"email_from": "a@x.y", "email_to": "b@x.y", "email_subject": "hi"}),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create email: {b}");
    let uuid = b["email_queue_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/email_queue/{uuid}"),
        Some(serde_json::json!({"email_response": "250 OK", "email_retry_count": 2})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/email_queue/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(g["email_response"], "250 OK", "persisted: {g}");
    assert_eq!(g["email_retry_count"], 2);
}

#[tokio::test]
async fn bridge_compose_from_action_parts() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(), &token, "POST", "/api/v1/bridges",
        Some(serde_json::json!({"bridge_name": "br-comp", "bridge_action": "user", "destination_number": "1001"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create bridge: {b}");
    assert_eq!(b["bridge_destination"], "user/1001", "composed: {b}");
    let uuid = b["bridge_uuid"].as_str().unwrap().to_string();
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/bridges/{uuid}"),
        Some(serde_json::json!({"bridge_action": "loopback", "destination_number": "2002"})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["bridge_destination"], "loopback/2002");
}
