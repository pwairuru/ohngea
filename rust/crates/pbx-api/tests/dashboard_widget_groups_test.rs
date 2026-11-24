use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

// PHP ground truth: core/dashboard v_dashboard_widget_groups (widget ↔
// group assignments managed in dashboard_widget_edit.php, perms
// dashboard_widget_group_view/add/edit/delete). Rust had dashboards +
// widgets but no assignment surface.
fn token(domain: &str, perms: &[&str]) -> String {
    pbx_core::auth::encode_claims(
        &format!("tester-{domain}"),
        domain,
        "tester",
        &perms.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    )
}

async fn req(
    method: &str,
    path: &str,
    token: Option<&str>,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = pbx_api::test_app();
    let mut b = match method {
        "GET" => Request::get(path),
        "POST" => Request::post(path),
        "PATCH" => Request::patch(path),
        "DELETE" => Request::delete(path),
        _ => Request::post(path),
    };
    if let Some(t) = token {
        b = b.header("authorization", format!("Bearer {t}"));
    }
    let res = app
        .oneshot(
            b.header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

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
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .unwrap()
        .get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token")
        .to_string()
}

#[tokio::test]
async fn widget_group_assign_roundtrip() {
    let admin = login_token().await;
    let (s, v) = req(
        "POST",
        "/api/v1/dashboards",
        Some(&admin),
        serde_json::json!({"dashboard_name": "wg-test"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create dashboard: {v}");
    let did = v
        .get("dashboard_uuid")
        .and_then(|i| i.as_str())
        .unwrap()
        .to_owned();
    let (s, v) = req(
        "POST",
        &format!("/api/v1/dashboards/{did}/widgets"),
        Some(&admin),
        serde_json::json!({"widget_name": "wg-widget"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create widget: {v}");
    let wid = v
        .get("dashboard_widget_uuid")
        .and_then(|i| i.as_str())
        .unwrap()
        .to_owned();
    let base = format!("/api/v1/dashboards/{did}/widgets/{wid}/groups");

    // assign
    let gid = uuid::Uuid::now_v7().to_string();
    let (s, v) = req(
        "POST",
        &base,
        Some(&admin),
        serde_json::json!({"group_uuid": gid}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "assign group: {v}");
    let assign_id = v
        .get("dashboard_widget_group_uuid")
        .and_then(|i| i.as_str())
        .expect("assign id")
        .to_owned();

    // list contains it
    let (s, v) = req("GET", &base, Some(&admin), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "list groups: {v}");
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("group_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(got.contains(&gid), "assigned group listed: {v}");

    // missing view perm -> 403
    let limited = token("wg-tenant", &["dashboard_view", "dashboard_widget_view"]);
    let (s, v) = req("GET", &base, Some(&limited), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "missing perm 403: {v}");

    // unassign
    let (s, _) = req(
        "DELETE",
        &format!("{base}/{assign_id}"),
        Some(&admin),
        serde_json::json!({}),
    )
    .await;
    assert!(
        s == StatusCode::NO_CONTENT || s == StatusCode::OK,
        "unassign: {s}"
    );
    let (s, v) = req("GET", &base, Some(&admin), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("group_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(!got.contains(&gid), "unassigned gone: {v}");

    // other tenant's widget -> 404 even with perms
    let other = token(
        "wg-other",
        &[
            "dashboard_view",
            "dashboard_widget_view",
            "dashboard_widget_group_view",
        ],
    );
    let (s, _) = req("GET", &base, Some(&other), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "cross-tenant widget 404");
}
