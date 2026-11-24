//! Round-10 parity: users full fields roundtrip (PHP v_users).
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
async fn users_full_fields_roundtrip() {
    let app = pbx_api::test_app();
    let token = login_token().await;
    let uname = format!("parity_{}", &uuid::Uuid::now_v7().to_string()[..8]);
    let res = app
        .clone()
        .oneshot(auth(
            Request::post("/api/v1/users")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "username": uname,
                        "password": "Secret123!",
                        "password_confirm": "Secret123!",
                        "user_email": "a@example.com",
                        "user_language": "en-us",
                        "user_status": "Active",
                        "user_time_zone": "UTC",
                        "user_type": "default",
                        "contact_name_given": "Ada",
                        "contact_name_family": "Lovelace",
                        "contact_organization": "Acme"
                    })
                    .to_string(),
                ))
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 32)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["user_language"], "en-us");
    assert_eq!(v["user_status"], "Active");
    assert_eq!(v["contact_name_given"], "Ada");
}
