//! Round-14 parity: dashboards + widgets full fields roundtrip.
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
async fn dashboards_widgets_full_fields_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/dashboards",
        Some(serde_json::json!({
            "dashboard_name": "Probe",
            "dashboard_enabled": true,
            "dashboard_description": "desc"
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "dashboard: {b}");
    assert_eq!(b["dashboard_name"], "Probe");
    let did = b["dashboard_uuid"].as_str().unwrap().to_string();
    let (s, w) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        &format!("/api/v1/dashboards/{did}/widgets"),
        Some(serde_json::json!({
            "widget_name": "W1",
            "widget_path": "/p",
            "widget_icon": "i",
            "widget_enabled": true,
            "widget_order": 1
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "widget: {w}");
    assert_eq!(w["widget_name"], "W1");
    assert_eq!(w["widget_path"], "/p");
}
