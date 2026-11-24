//! Time-conditions dialplan parity: PHP builds a real v_dialplans row +
//! details (destination condition, timezone, transfers). Creating a time
//! condition must maintain that backing dialplan so FS routing works, and
//! the FS dialplan XML must render it.

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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn time_condition_maintains_backing_dialplan() {
    let token = login_token().await;
    let (s, b) = call(
        pbx_api::test_app(),
        &token,
        "POST",
        "/api/v1/time_conditions",
        Some(serde_json::json!({
            "name": "tc-backing", "dialplan_number": "8899", "dialplan_order": 210,
            "context": "public", "timezone": "Africa/Nairobi",
            "true_destination": "1000", "false_destination": "1001"
        })),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create tc: {b}");
    // PHP aliases accepted
    assert_eq!(b["dialplan_number"], "8899");
    assert_eq!(b["extension"], "8899");
    let uuid = b["time_condition_uuid"].as_str().unwrap().to_string();
    // backing dialplan exists with order + destination condition
    let (s, dps) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/dialplans",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let dp = dps["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["dialplan_number"] == "8899")
        .cloned();
    assert!(dp.is_some(), "backing dialplan created: {dps}");
    let dp = dp.unwrap();
    assert_eq!(dp["dialplan_order"], 210);
    let det = dp["details"].as_array().cloned().unwrap_or_default();
    assert!(
        det.iter().any(|d| d["data"] == "^8899$"),
        "destination condition: {det:?}"
    );
    assert!(
        det.iter().any(|d| d["data"] == "timezone=Africa/Nairobi"),
        "timezone action: {det:?}"
    );
    // FS dialplan XML renders it (DB-backed render)
    let res = pbx_api::test_app()
        .oneshot({
            let (mut parts, body) =
                Request::get("/api/internal/freeswitch/xml?section=dialplan&key_value=public")
                    .body(Body::empty())
                    .unwrap()
                    .into_parts();
            parts
                .headers
                .insert("authorization", format!("Bearer {token}").parse().unwrap());
            Request::from_parts(parts, body)
        })
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        xml.contains("8899"),
        "dialplan XML renders condition: {xml}"
    );
    // delete cascades to the backing dialplan
    let (s, _) = call(
        pbx_api::test_app(),
        &token,
        "DELETE",
        &format!("/api/v1/time_conditions/{uuid}"),
        None,
    )
    .await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, dps2) = call(
        pbx_api::test_app(),
        &token,
        "GET",
        "/api/v1/dialplans",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        dps2["data"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["dialplan_number"] != "8899"),
        "backing dialplan removed: {dps2}"
    );
}
