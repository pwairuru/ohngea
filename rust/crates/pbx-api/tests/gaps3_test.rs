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

async fn get_status(app: axum::Router, token: &str, path: &str) -> StatusCode {
    let res = app
        .oneshot(auth(Request::get(path).body(Body::empty()).unwrap(), token))
        .await
        .unwrap();
    res.status()
}

// RED: these 5 prefixes are 404 today, must become 200 with {data,total} or object
#[tokio::test]
async fn gaps3_missing_prefixes_become_200() {
    let token = login_token().await;
    for p in [
        "/api/v1/domain_limits",
        "/api/v1/extension_settings",
        "/api/v1/sofia_global_settings",
        "/api/v1/call_recordings",
        "/api/v1/call_flows",
    ] {
        let s = get_status(pbx_api::test_app(), &token, p).await;
        assert_eq!(s, StatusCode::OK, "prefix {p} should be 200, got {s}");
    }
}

#[tokio::test]
async fn gaps3_webrtc_hyphen_alias() {
    let token = login_token().await;
    let s = get_status(pbx_api::test_app(), &token, "/api/v1/webrtc-settings").await;
    assert_eq!(s, StatusCode::OK, "hyphen alias should be 200, got {s}");
}
