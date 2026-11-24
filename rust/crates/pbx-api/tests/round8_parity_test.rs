//! Round-8 parity: user_logs full fields roundtrip (PHP v_user_logs).
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

#[tokio::test]
async fn user_logs_full_fields_roundtrip() {
    let app = pbx_api::test_app();
    let token = login_token().await;
    // list (empty ok, must be 200 with envelope)
    let res = app
        .clone()
        .oneshot(auth(
            Request::get("/api/v1/user_logs")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    // create with PHP fields
    let res = app
        .clone()
        .oneshot(auth(
            Request::post("/api/v1/user_logs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "username": "probe",
                        "type": "login",
                        "result": "success",
                        "remote_address": "10.0.0.1",
                        "user_agent": "parity-test"
                    })
                    .to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["username"], "probe");
    assert_eq!(v["type"], "login");
    assert_eq!(v["result"], "success");
    assert_eq!(v["remote_address"], "10.0.0.1");
    assert_eq!(v["user_agent"], "parity-test");
}
