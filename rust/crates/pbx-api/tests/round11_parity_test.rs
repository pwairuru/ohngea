//! Round-11 parity: contacts phones link-table full fields.
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
async fn contacts_phones_full_fields_roundtrip() {
    let app = pbx_api::test_app();
    let token = login_token().await;
    // create parent contact
    let res = app
        .clone()
        .oneshot(auth(
            Request::post("/api/v1/contacts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"name": "Phone Probe", "number": "5559"}).to_string(),
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
    let cid = v["contact_uuid"].as_str().unwrap().to_string();
    // create phone with full PHP fields
    let res = app
        .clone()
        .oneshot(auth(
            Request::post(&format!("/api/v1/contacts/{cid}/phones"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "phone_number": "5551234",
                        "phone_label": "mobile",
                        "phone_label_custom": "My Mobile",
                        "phone_country_code": "1",
                        "phone_extension": "101",
                        "phone_speed_dial": "5",
                        "phone_description": "desc",
                        "phone_primary": true,
                        "phone_type_voice": true,
                        "phone_type_fax": false,
                        "phone_type_video": true,
                        "phone_type_text": true
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
    let p: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(p["phone_label_custom"], "My Mobile");
    assert_eq!(p["phone_country_code"], "1");
    assert_eq!(p["phone_speed_dial"], "5");
    assert_eq!(p["phone_type_voice"], true);
}
