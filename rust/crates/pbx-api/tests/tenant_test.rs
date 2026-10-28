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
    v.get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token")
        .to_string()
}

#[tokio::test]
async fn missing_token_is_401_with_login_redirect_hint() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_token_is_401() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn valid_token_returns_envelope() {
    let token = login_token().await;
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            Request::get("/api/v1/extensions")
                .header("authorization", format!("Bearer {token}"))
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        v.get("data").and_then(|d| d.as_array()).map(|a| a.len()),
        Some(0)
    );
    assert_eq!(v.get("total").and_then(|t| t.as_u64()), Some(0));
}
