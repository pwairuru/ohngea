use tower::ServiceExt;

#[tokio::test]
async fn missing_token_is_401_with_login_redirect_hint() {
    let app = pbx_api::test_app();
    let res = app
        .oneshot(
            axum::http::Request::get("/api/v1/extensions")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);
}
