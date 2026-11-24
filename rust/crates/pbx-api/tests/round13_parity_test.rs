//! Round-13 parity: contacts core full fields roundtrip.
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
async fn contacts_core_full_fields_roundtrip() {
    let app = pbx_api::test_app();
    let token = login_token().await;
    let res = app
        .clone()
        .oneshot(auth(
            Request::post("/api/v1/contacts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Core Probe",
                        "number": "5571",
                        "contact_type": "customer",
                        "contact_organization": "Acme",
                        "contact_name_given": "Ada",
                        "contact_name_family": "Lovelace",
                        "contact_nickname": "Ada",
                        "contact_title": "Eng",
                        "contact_role": "Dev",
                        "contact_category": "vip",
                        "contact_time_zone": "UTC"
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
    assert_eq!(v["contact_name_given"], "Ada");
    assert_eq!(v["contact_category"], "vip");
    assert_eq!(v["contact_time_zone"], "UTC");
}
