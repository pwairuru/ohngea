use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

// PHP parity for cross-tenant assign + patch-time domain moves:
// - extension_domain: create honors posted domain_uuid (else tenant).
// - user_domain: create honors posted domain_uuid (else tenant).
// - Gated tables (dialplans/dialplan_domain, call_block/call_block_domain,
//   music_on_hold/music_on_hold_domain, ivr/ivr_menu_domain,
//   time_conditions/time_condition_domain): PATCH honors posted
//   domain_uuid (UUID or empty = global); without the perm the row
//   keeps its domain.
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
    let bytes = axum::body::to_bytes(res.into_body(), 1024 * 256)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

fn tenant_uuid(name: &str) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, name.as_bytes()).to_string()
}

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
}

#[tokio::test]
async fn extension_cross_tenant_assign() {
    // Tenant B's UUID (opaque name mapping matches routing_db::domain).
    let b_id = tenant_uuid("assign-tenant-b");
    let a = token(
        "assign-tenant-a",
        &["extension_add", "extension_view", "extension_domain"],
    );
    let ext = format!("9{:04}", rand4());

    ensure_tenant("assign-tenant-b", &["extension_add"]).await;
    // With the perm: row lands in tenant B.
    let (s, v) = req(
        "POST",
        "/api/v1/extensions",
        Some(&a),
        serde_json::json!({"extension": ext, "password": "pw123456", "domain_id": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "cross-tenant create: {v}");
    let id = v
        .get("extension_uuid")
        .and_then(|i| i.as_str())
        .expect("id")
        .to_owned();

    // A-scoped list must NOT show it; B-scoped list must.
    let (s, v) = req("GET", "/api/v1/extensions", Some(&a), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let mine: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("extension_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(!mine.contains(&id), "A must not see B's row");

    let b = token("assign-tenant-b", &["extension_view"]);
    let (s, v) = req("GET", "/api/v1/extensions", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let theirs: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("extension_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(theirs.contains(&id), "B must see its row: {v}");

    // Without the perm the posted domain is ignored (forced to own tenant).
    let a_noperm = token("assign-tenant-a", &["extension_add", "extension_view"]);
    let ext2 = format!("8{:04}", rand4());
    let (s, v) = req(
        "POST",
        "/api/v1/extensions",
        Some(&a_noperm),
        serde_json::json!({"extension": ext2, "password": "pw123456", "domain_id": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "forced-tenant create: {v}");
    let id2 = v.get("extension_uuid").and_then(|i| i.as_str()).unwrap().to_owned();
    let (s, v) = req("GET", "/api/v1/extensions", Some(&a_noperm), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let mine: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("extension_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(mine.contains(&id2), "row forced to own tenant: {v}");

    // PATCH re-home with the perm moves the row; without it, untouched.
    let a_dom = token("assign-tenant-a", &["extension_add", "extension_view", "extension_domain"]);
    let (s, _) = req(
        "PATCH",
        &format!("/api/v1/extensions/{id2}"),
        Some(&a_dom),
        serde_json::json!({"domain_id": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch re-home");
    let (s, _) = req(
        "GET",
        &format!("/api/v1/extensions/{id2}"),
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "B sees re-homed row");
}


/// Materialize the tenant's `domains` row (write paths auto-insert the
/// caller's parent domain; cross-tenant targets need an explicit write
/// first — PHP likewise requires the domain to exist).
async fn ensure_tenant(domain: &str, perms: &[&str]) {
    let t = token(domain, perms);
    let ext = format!("7{:04}", rand4());
    let (s, v) = req(
        "POST",
        "/api/v1/extensions",
        Some(&t),
        serde_json::json!({"extension": ext, "password": "pw123456"}),
    )
    .await;
    assert!(
        s == StatusCode::CREATED,
        "bootstrap tenant {domain}: {v}"
    );
}

fn rand4() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    1000 + (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42)
        % 9000) as u16
}

#[tokio::test]
async fn user_cross_tenant_assign() {
    ensure_tenant("userassign-tenant-b", &["extension_add"]).await;
    let b_id = tenant_uuid("userassign-tenant-b");
    let a = token("userassign-tenant-a", &["user_add", "user_view", "user_domain"]);
    let uname = uniq("crossuser");

    let (s, v) = req(
        "POST",
        "/api/v1/users",
        Some(&a),
        serde_json::json!({"username": uname, "password": "pw123456", "domain_uuid": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "cross-tenant user: {v}");
    let id = v
        .get("user_uuid")
        .or_else(|| v.get("id"))
        .and_then(|i| i.as_str())
        .expect("user id")
        .to_owned();

    let b = token("userassign-tenant-b", &["user_view"]);
    let (s, v) = req(
        "GET",
        &format!("/api/v1/users/{id}"),
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "B sees its user: {v}");

    // Without the perm the row stays in the caller's tenant.
    let uname2 = uniq("ownuser");
    let a_noperm = token("userassign-tenant-a", &["user_add", "user_view"]);
    let (s, v) = req(
        "POST",
        "/api/v1/users",
        Some(&a_noperm),
        serde_json::json!({"username": uname2, "password": "pw123456", "domain_uuid": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "forced-tenant user: {v}");
    let id2 = v
        .get("user_uuid")
        .or_else(|| v.get("id"))
        .and_then(|i| i.as_str())
        .unwrap()
        .to_owned();
    let (s, _) = req(
        "GET",
        &format!("/api/v1/users/{id2}"),
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "B must not see A's user");
}

#[tokio::test]
async fn patch_domain_moves_and_globalizes() {
    // dialplan: move tenant row to another tenant with dialplan_domain.
    let a = token(
        "mv-tenant-a",
        &["dialplan_add", "dialplan_view", "dialplan_edit", "dialplan_domain"],
    );
    let (s, v) = req(
        "POST",
        "/api/v1/dialplans",
        Some(&a),
        serde_json::json!({"name": uniq("mvdp")}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create dialplan: {v}");
    let id = v.get("dialplan_uuid").and_then(|i| i.as_str()).unwrap().to_owned();
    ensure_tenant("mv-tenant-b", &["extension_add"]).await;
    let b_id = tenant_uuid("mv-tenant-b");
    let (s, v) = req(
        "PATCH",
        &format!("/api/v1/dialplans/{id}"),
        Some(&a),
        serde_json::json!({"domain_uuid": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch move: {v}");
    let b = token("mv-tenant-b", &["dialplan_view"]);
    let (s, _) = req(
        "GET",
        &format!("/api/v1/dialplans/{id}"),
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "B sees moved dialplan");

    // Without the perm the domain is untouched.
    let a_noperm = token("mv2-tenant-a", &["dialplan_add", "dialplan_view", "dialplan_edit"]);
    let (s, v) = req(
        "POST",
        "/api/v1/dialplans",
        Some(&a_noperm),
        serde_json::json!({"name": uniq("mvdp2")}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let id2 = v.get("dialplan_uuid").and_then(|i| i.as_str()).unwrap().to_owned();
    let (s, _) = req(
        "PATCH",
        &format!("/api/v1/dialplans/{id2}"),
        Some(&a_noperm),
        serde_json::json!({"domain_uuid": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = req(
        "GET",
        &format!("/api/v1/dialplans/{id2}"),
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "domain unchanged without perm");

    // call_block: empty string globalizes with the perm.
    let a = token(
        "mvb-tenant-a",
        &["call_block_add", "call_block_view", "call_block_edit", "call_block_domain"],
    );
    let blk = uniq("777");
    let (s, v) = req(
        "POST",
        "/api/v1/call_block",
        Some(&a),
        serde_json::json!({"call_block_number": blk}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create block: {v}");
    let bid = v.get("call_block_uuid").and_then(|i| i.as_str()).unwrap().to_owned();
    ensure_tenant("mvb-tenant-b", &["extension_add"]).await;
    let (s, _) = req(
        "PATCH",
        &format!("/api/v1/call_block/{bid}"),
        Some(&a),
        serde_json::json!({"domain_uuid": ""}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch globalize: {v}");
    let b_g = token("mvb-tenant-b", &["call_block_view", "call_block_domain"]);
    let (s, v) = req("GET", "/api/v1/call_block", Some(&b_g), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let names: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("call_block_number")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(names.contains(&blk), "globalized row visible: {v}");
}

#[tokio::test]
async fn user_patch_moves_tenant_with_perm() {
    let a = token("usermove-a", &["user_add", "user_view", "user_edit", "user_domain"]);
    let uname = uniq("moveuser");
    let (s, v) = req(
        "POST",
        "/api/v1/users",
        Some(&a),
        serde_json::json!({"username": uname, "password": "pw123456"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create user: {v}");
    let id = v.get("user_uuid").and_then(|i| i.as_str()).unwrap().to_owned();

    ensure_tenant("usermove-b", &["extension_add"]).await;
    let b_id = tenant_uuid("usermove-b");
    let (s, v) = req(
        "PATCH",
        &format!("/api/v1/users/{id}"),
        Some(&a),
        serde_json::json!({"domain_uuid": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "patch move: {v}");

    let b = token("usermove-b", &["user_view"]);
    let (s, _) = req("GET", &format!("/api/v1/users/{id}"), Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "B sees moved user");

    // Without the perm the domain is untouched.
    let a2 = token("usermove2-a", &["user_add", "user_view", "user_edit"]);
    let uname2 = uniq("stayuser");
    let (s, v) = req(
        "POST",
        "/api/v1/users",
        Some(&a2),
        serde_json::json!({"username": uname2, "password": "pw123456"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let id2 = v.get("user_uuid").and_then(|i| i.as_str()).unwrap().to_owned();
    let (s, _) = req(
        "PATCH",
        &format!("/api/v1/users/{id2}"),
        Some(&a2),
        serde_json::json!({"domain_uuid": b_id}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = req("GET", &format!("/api/v1/users/{id2}"), Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "domain unchanged without perm");
}
