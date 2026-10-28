use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn login(app: axum::Router, body: serde_json::Value) -> axum::response::Response {
    app.oneshot(
        Request::post("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn legacy_login_and_me_roundtrip() {
    let app = pbx_api::test_app();
    let login_body = serde_json::json!({"username":"admin","password":"x","domain":"default"});
    let res = login(app.clone(), login_body).await;
    assert_eq!(res.status(), StatusCode::OK);
    // Cookie parity with Python router (httpOnly access_token, Path=/, SameSite=Lax).
    let set_cookie = res
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("missing Set-Cookie")
        .to_string();
    assert!(
        set_cookie.starts_with("access_token="),
        "cookie: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly"), "cookie: {set_cookie}");
    assert!(set_cookie.contains("Path=/"), "cookie: {set_cookie}");
    assert!(set_cookie.contains("SameSite=Lax"), "cookie: {set_cookie}");
    // Accept-compat middleware: Vary: Accept on every response, no 406.
    assert_eq!(
        res.headers().get("vary").and_then(|v| v.to_str().ok()),
        Some("Accept")
    );
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = v
        .get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token");
    assert_eq!(v.get("token_type").and_then(|t| t.as_str()), Some("Bearer"));
    assert_eq!(v.get("expires_in").and_then(|t| t.as_u64()), Some(3600));

    let res = app
        .oneshot(
            Request::get("/api/v1/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn legacy_auth_paths_and_unauthorized_me() {
    let app = pbx_api::test_app();
    let login_body = serde_json::json!({"username":"admin","password":"x","domain":"default"});
    let res = app
        .clone()
        .oneshot(
            Request::post("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        v.get("access_token").and_then(|t| t.as_str()).is_some(),
        "missing access_token"
    );
    assert_eq!(v.get("token_type").and_then(|t| t.as_str()), Some("Bearer"));
    assert_eq!(v.get("expires_in").and_then(|t| t.as_u64()), Some(3600));

    let res = app
        .oneshot(Request::get("/auth/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_missing_or_empty_credentials_is_401() {
    // Missing password field -> 401 (not 422).
    let res = login(pbx_api::test_app(), serde_json::json!({"username":"admin"})).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Empty username -> 401.
    let res = login(
        pbx_api::test_app(),
        serde_json::json!({"username":"","password":"x"}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // Empty password -> 401.
    let res = login(
        pbx_api::test_app(),
        serde_json::json!({"username":"admin","password":"  "}),
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn accept_header_compat_no_406() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/health")
                .header("accept", "application/vnd.pbx.v1+json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("vary").and_then(|v| v.to_str().ok()),
        Some("Accept")
    );
}
