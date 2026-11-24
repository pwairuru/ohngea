//! Failing-first: user_setting_order roundtrip (PHP v_user_settings column).
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
        "PUT" => Request::put(path),
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}
#[tokio::test]
async fn user_setting_order_roundtrip() {
    let token = login_token().await;
    // need a user to attach settings to: create minimal user via API
    let (s, u) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/users",
        Some(serde_json::json!({"username":"ordtest","password":"Secret123!","user_enabled":true})),
    )
    .await;
    assert!(
        s == StatusCode::CREATED || s == StatusCode::OK || s == StatusCode::CONFLICT,
        "create user: {u}"
    );
    let uid = u
        .get("user_uuid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();
    // list to find uuid if conflict
    let uid = if uid.is_empty() {
        let (s, l) = call(pbx_api::test_app(), &token, "GET", "/api/v1/users", None).await;
        assert_eq!(s, StatusCode::OK, "list users: {l}");
        l["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["username"] == "ordtest")
            .and_then(|r| r["user_uuid"].as_str())
            .unwrap_or("")
            .to_string()
    } else {
        uid
    };
    assert!(!uid.is_empty(), "user uuid resolved");
    let (s, b) = call(pbx_api::test_app(), &token, "PUT", &format!("/api/v1/users/{uid}/settings"), Some(serde_json::json!({"settings":[{"user_setting_category":"c","user_setting_name":"n","user_setting_order":5}] }))).await;
    // accept 200 replace-all
    assert_eq!(s, StatusCode::OK, "put settings: {b}");
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/users/{uid}/settings"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get settings: {g}");
    let row = g["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["user_setting_name"] == "n")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(row["user_setting_order"], 5, "order persisted: {g}");
}
