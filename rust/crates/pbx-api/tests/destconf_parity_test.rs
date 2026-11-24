//! Destinations + conferences full-field parity: PHP `v_destinations` has
//! ~30 columns (fax/group/provider links, regex, ringback, type flags…)
//! and `v_conferences` 14 — the routers must accept, persist and return them.

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
        "PATCH" => Request::patch(path),
        "DELETE" => Request::delete(path),
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn destinations_full_fields_roundtrip() {
    let token = login_token().await;
    let payload = serde_json::json!({
        "destination_type": "inbound",
        "destination_number": "1555000999",
        "destination_context": "public",
        "destination_description": "main DID",
        "destination_caller_id_name": "Acme",
        "destination_caller_id_number": "1555000999",
        "destination_accountcode": "acct-9",
        "destination_trunk_prefix": "9",
        "destination_area_code": "555",
        "destination_number_regex": "^\\+?1555",
        "destination_distinctive_ring": "ring1",
        "destination_ringback": "us-ring",
        "destination_type_voice": true,
        "destination_type_fax": false,
        "destination_type_emergency": false,
        "destination_record": true,
        "destination_hold_music": "default",
        "destination_cid_name_prefix": "ACME"
    });
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/destinations",
        Some(payload),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create dest: {b}");
    for key in [
        "destination_accountcode",
        "destination_trunk_prefix",
        "destination_number_regex",
        "destination_ringback",
        "destination_type_voice",
        "destination_record",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["destination_uuid"].as_str().unwrap().to_string();
    let (s, g) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        &format!("/api/v1/destinations/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK, "get: {g}");
    assert_eq!(g["destination_accountcode"], "acct-9", "persisted: {g}");
    assert_eq!(g["destination_type_voice"], true);
    let (s, p) = call(
        pbx_api::test_app(),
        &token,
        "PATCH",
        &format!("/api/v1/destinations/{uuid}"),
        Some(serde_json::json!({"destination_ringback": "uk-ring", "destination_type_fax": true})),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch: {p}");
    assert_eq!(p["destination_ringback"], "uk-ring", "patch echoes: {p}");
    assert_eq!(p["destination_type_fax"], true);
}

#[tokio::test]
async fn conferences_full_fields_roundtrip() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/conferences",
        Some(serde_json::json!({
            "conference_name": "Board",
            "conference_extension": "3800",
            "conference_profile": "default",
            "conference_enabled": false,
            "pin_number": "1234",
            "email_address": "b@x.y",
            "account_code": "conf-acct",
            "flags": "moh",
            "conference_order": 5,
            "conference_description": "board room",
            "context": "default"
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create conf: {b}");
    for key in [
        "conference_enabled",
        "pin_number",
        "email_address",
        "flags",
        "conference_description",
        "context",
    ] {
        assert!(b.get(key).is_some(), "create echoes `{key}`: {b}");
    }
    let uuid = b["conference_uuid"].as_str().unwrap().to_string();
    let (s, l) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/conferences",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let row = l["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["conference_uuid"] == uuid)
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert_eq!(row["pin_number"], "1234", "list carries full fields: {l}");
}
