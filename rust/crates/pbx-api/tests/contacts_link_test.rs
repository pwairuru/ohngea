//! Contacts link-table parity: attachments/groups/relations/settings/times/
//! users (PHP `core/contacts/*`) must support create/list/delete under a
//! contact, like the existing addresses/phones/emails/urls/notes children.

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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn contacts_link_children_crud() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/contacts",
        Some(serde_json::json!({"name": "Link Probe", "number": "5551"})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create contact: {b}");
    let cid = b
        .get("contact_uuid")
        .or_else(|| b.get("uuid"))
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    let cases: &[(&str, serde_json::Value, &str)] = &[
        (
            "attachments",
            serde_json::json!({"attachment_filename": "a.txt"}),
            "contact_attachment_uuid",
        ),
        ("groups", serde_json::json!({}), "contact_group_uuid"),
        (
            "relations",
            serde_json::json!({"relation_label": "spouse"}),
            "contact_relation_uuid",
        ),
        (
            "settings",
            serde_json::json!({"contact_setting_name": "tz", "contact_setting_value": "UTC"}),
            "contact_setting_uuid",
        ),
        (
            "times",
            serde_json::json!({"time_start": "09:00", "time_stop": "17:00"}),
            "contact_time_uuid",
        ),
        ("users", serde_json::json!({}), "contact_user_uuid"),
    ];
    for (child, payload, idkey) in cases {
        let base = format!("/api/v1/contacts/{cid}/{child}");
        let (s, b) = call(
            pbx_api::test_app(),
            &token,
            "POST",
            &base,
            Some(payload.clone()),
        )
        .await;
        assert_eq!(s, StatusCode::CREATED, "create {child}: {b}");
        let kid = b.get(*idkey).and_then(|v| v.as_str()).unwrap().to_string();
        let (s, l) = call(pbx_api::test_app(), &token, "GET", &base, None).await;
        assert_eq!(s, StatusCode::OK, "list {child}: {l}");
        assert_eq!(l["total"], 1, "one {child} row: {l}");
        let (s, _) = call(
            pbx_api::test_app(),
            &token,
            "DELETE",
            &format!("{base}/{kid}"),
            None,
        )
        .await;
        assert_eq!(s, StatusCode::NO_CONTENT, "delete {child}");
        let (s, l2) = call(pbx_api::test_app(), &token, "GET", &base, None).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(l2["total"], 0, "{child} gone: {l2}");
    }
}
