//! Round-9 parity: email_templates full fields roundtrip (PHP v_email_templates + Python EmailTemplate superset).
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
async fn email_templates_full_fields_roundtrip() {
    let app = pbx_api::test_app();
    let token = login_token().await;
    let res = app
        .clone()
        .oneshot(auth(
            Request::get("/api/v1/email_templates")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let res = app
        .clone()
        .oneshot(auth(
            Request::post("/api/v1/email_templates")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "template_name": "probe",
                        "template_language": "en",
                        "template_category": "voicemail",
                        "template_subcategory": "default",
                        "template_subject": "hi",
                        "template_body": "body",
                        "template_type": "html",
                        "template_enabled": true,
                        "template_description": "probe desc"
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
    assert_eq!(v["template_name"], "probe");
    assert_eq!(v["template_category"], "voicemail");
    assert_eq!(v["template_subcategory"], "default");
    assert_eq!(v["template_type"], "html");
    assert_eq!(v["template_enabled"], true);
}
