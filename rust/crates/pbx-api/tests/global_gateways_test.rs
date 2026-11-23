use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

// PHP ground truth (app/gateways):
// - list: `(domain_uuid = :d OR domain_uuid IS NULL)` only with
//   `gateway_domain`; otherwise strictly own domain.
// - ?show=all + `gateway_all` → every row (covered in show_all_test.rs).
// - create/edit: without `gateway_domain` the domain is forced to the
//   caller's tenant; with it the posted domain stands — including NULL
//   (global/shared gateway) or another tenant's UUID.
// - detail by id has no domain gate in PHP's edit page; Rust keeps
//   tenant-locking for tenant rows and opens global rows to
//   `gateway_domain` holders only.
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 128)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
}

#[tokio::test]
async fn global_gateway_visibility_and_lifecycle() {
    // tenant A admin token via crafted claims (add+view+domain scope).
    let a = token("gw-tenant-a", &["gateway_add", "gateway_view", "gateway_edit"]);

    // A creates a tenant-scoped gateway.
    let (s, v) = req(
        "POST",
        "/api/v1/gateways",
        Some(&a),
        serde_json::json!({"gateway": uniq("scoped"), "proxy": "10.0.0.1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create scoped: {v}");

    // A creates a GLOBAL gateway (explicit null domain + gateway_domain).
    let a_dom = token(
        "gw-tenant-a",
        &["gateway_add", "gateway_view", "gateway_edit", "gateway_domain"],
    );
    let gname = uniq("global");
    let (s, v) = req(
        "POST",
        "/api/v1/gateways",
        Some(&a_dom),
        serde_json::json!({"gateway": gname, "proxy": "10.0.0.2", "domain_id": ""}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create global: {v}");
    assert!(
        v.get("domain_id").map_or(true, |d| d.is_null()),
        "global row has null domain: {v}"
    );
    let gid = v
        .get("gateway_uuid")
        .and_then(|i| i.as_str())
        .expect("gateway_uuid")
        .to_owned();

    // Posting null WITHOUT the perm forces the caller's own tenant.
    let (s, v) = req(
        "POST",
        "/api/v1/gateways",
        Some(&a),
        serde_json::json!({"gateway": uniq("forced"), "proxy": "10.0.0.3", "domain_id": ""}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "forced-tenant create: {v}");
    assert!(
        v.get("domain_id").and_then(|d| d.as_str()).is_some(),
        "without perm the domain is forced, never null: {v}"
    );

    // Tenant B without gateway_domain: sees neither A's row nor global.
    let b = token("gw-tenant-b", &["gateway_view"]);
    let (s, v) = req("GET", "/api/v1/gateways", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let names: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("gateway")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        !names.iter().any(|n| n == &gname),
        "B must not see the global row without gateway_domain: {names:?}"
    );

    // Tenant B WITH gateway_domain: sees global, not A's tenant row.
    let b_dom = token("gw-tenant-b", &["gateway_view", "gateway_domain"]);
    let (s, v) = req("GET", "/api/v1/gateways", Some(&b_dom), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "B list with domain perm: {v}");
    let names: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("gateway")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        names.iter().any(|n| n == &gname),
        "B must see the global row with gateway_domain: {names:?}"
    );

    // Global detail: 404 without perm, 200 with.
    let (s, _) = req(
        "GET",
        &format!("/api/v1/gateways/{gid}"),
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "global detail w/o perm 404");
    let (s, v) = req(
        "GET",
        &format!("/api/v1/gateways/{gid}"),
        Some(&b_dom),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "global detail with perm: {v}");

    // Global patch/delete require the domain perm too.
    let b_edit = token("gw-tenant-b", &["gateway_edit", "gateway_domain"]);
    let (s, v) = req(
        "PATCH",
        &format!("/api/v1/gateways/{gid}"),
        Some(&b_edit),
        serde_json::json!({"description": "shared edit"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "global patch with perm: {v}");
    let b_nodelete = token("gw-tenant-b", &["gateway_delete"]);
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/gateways/{gid}"),
        Some(&b_nodelete),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "global delete w/o domain perm 404");

    // Owner tenant can still delete its own global row.
    let a_del = token("gw-tenant-a", &["gateway_delete", "gateway_domain"]);
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/gateways/{gid}"),
        Some(&a_del),
        serde_json::json!({}),
    )
    .await;
    assert!(
        s == StatusCode::NO_CONTENT || s == StatusCode::OK,
        "owner deletes global row: {s}"
    );
}
