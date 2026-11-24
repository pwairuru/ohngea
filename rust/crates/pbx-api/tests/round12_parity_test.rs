//! Round-12 parity: contacts urls + addresses + relations full fields.
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
async fn contacts_urls_addresses_relations_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/contacts",
        Some(serde_json::json!({"name": "Link Probe 12", "number": "5561"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "contact: {b}");
    let cid = b["contact_uuid"].as_str().unwrap().to_string();
    // urls with description + primary
    let (s, u) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        &format!("/api/v1/contacts/{cid}/urls"),
        Some(serde_json::json!({
            "url_address": "https://example.com",
            "url_label": "site",
            "url_label_custom": "My Site",
            "url_description": "desc",
            "url_primary": true
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "url: {u}");
    assert_eq!(u["url_description"], "desc");
    assert_eq!(u["url_primary"], true);
    // addresses with lat/long
    let (s, a) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        &format!("/api/v1/contacts/{cid}/addresses"),
        Some(serde_json::json!({
            "address_street": "Main St",
            "address_latitude": "1.23",
            "address_longitude": "4.56"
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "address: {a}");
    assert_eq!(a["address_latitude"], "1.23");
}
