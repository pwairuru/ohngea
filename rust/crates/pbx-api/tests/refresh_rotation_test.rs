//! Refresh-token rotation with reuse detection (blocker 4.1).
//!
//! - login mints `{access_token, refresh_token}`; the refresh token carries
//!   a `jti` bound to a server-side `refresh_tokens` row (family).
//! - POST /auth/refresh with a refresh token rotates: old row revoked +
//!   replaced_by, new pair minted in the same family.
//! - Reusing a rotated (non-latest) token is 401 AND revokes the whole
//!   family (reuse detection); the rotated replacement then also 401s.
//! - Legacy jti-less access tokens keep the old stateless re-mint (200).

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn login_pair(app: axum::Router) -> (String, String) {
    let res = app
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"username":"admin","password":"x","domain":"default"})
                        .to_string(),
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
    let access = v
        .get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token")
        .to_owned();
    let refresh = v
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .expect("missing refresh_token")
        .to_owned();
    assert_ne!(access, refresh, "refresh must differ from access");
    (access, refresh)
}

async fn refresh_with(app: axum::Router, token: &str) -> (StatusCode, serde_json::Value) {
    let res = app
        .oneshot(
            Request::post("/api/v1/auth/refresh")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, v)
}

#[tokio::test]
async fn refresh_rotates_and_reuse_kills_family() {
    let app = pbx_api::test_app();
    let (_access, refresh1) = login_pair(app.clone()).await;

    // Rotate: new pair, different refresh token.
    let (s, v) = refresh_with(app.clone(), &refresh1).await;
    assert_eq!(s, StatusCode::OK, "rotate: {v}");
    let refresh2 = v
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .expect("rotated pair missing refresh_token")
        .to_owned();
    assert_ne!(refresh1, refresh2, "rotation must mint a fresh token");
    assert!(
        v.get("access_token").and_then(|t| t.as_str()).is_some(),
        "rotated pair missing access_token: {v}"
    );

    // Reuse of the rotated-out token: 401 + family revoked.
    let (s, _) = refresh_with(app.clone(), &refresh1).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "reused token must 401");

    // The replacement is now dead too (family revoked).
    let (s, v) = refresh_with(app.clone(), &refresh2).await;
    assert_eq!(s, StatusCode::UNAUTHORIZED, "revoked family must 401: {v}");
}

#[tokio::test]
async fn legacy_access_refresh_still_mints() {
    let app = pbx_api::test_app();
    let (access, _refresh) = login_pair(app.clone()).await;
    let (s, v) = refresh_with(app.clone(), &access).await;
    assert_eq!(s, StatusCode::OK, "legacy access refresh: {v}");
    assert!(
        v.get("access_token").and_then(|t| t.as_str()).is_some(),
        "legacy path missing access_token: {v}"
    );
}
