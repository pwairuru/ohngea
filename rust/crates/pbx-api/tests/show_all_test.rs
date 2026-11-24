use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

// PHP ground truth: `?show=all` + `<base>_all` perm drops the domain filter
// (extensions.php/extension_all, gateways.php/gateway_all,
// contacts.php/contact_all, users.php/user_all, xml_cdr.php/xml_cdr_all,
// emergency.php/emergency_logs_view_all, user_logs.php/user_log_all).
// Default (no show param, other values, or perm missing) stays tenant-scoped,
// and detail/update/delete by id stay tenant-locked even for `_all` holders.

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
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

fn data_len(v: &serde_json::Value) -> usize {
    v.get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
}

static EXT_N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Extensions: unique per test-process via atomic counter (numeric is
/// conventional; PHP also accepts non-numeric values).
fn uniq_ext() -> String {
    let n = EXT_N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("9{:04}", 1000 + (n % 9000))
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
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v.get("access_token")
        .and_then(|t| t.as_str())
        .expect("missing access_token")
        .to_string()
}

#[tokio::test]
async fn show_all_extensions_cross_tenant() {
    let admin = login_token().await;
    let ext = uniq_ext();

    // admin (default tenant) creates an extension
    let (s, v) = req(
        "POST",
        "/api/v1/extensions",
        Some(&admin),
        serde_json::json!({"extension": ext, "password": "pw123456"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create ext: {v}");
    let id = v
        .get("extension_uuid")
        .or_else(|| v.get("id"))
        .and_then(|i| i.as_str())
        .expect("ext id")
        .to_owned();

    // tenant B without extension_all: scoped, show=all ignored
    let b = token("tenant-b", &["extension_view", "extension_add"]);
    let (s, v) = req("GET", "/api/v1/extensions", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(data_len(&v), 0, "B must not see default-tenant rows: {v}");
    let (s, v) = req(
        "GET",
        "/api/v1/extensions?show=all",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        data_len(&v),
        0,
        "show=all without perm must stay scoped: {v}"
    );

    // tenant B WITH extension_all: ?show=all sees both tenants
    let b_all = token("tenant-b", &["extension_view", "extension_all"]);
    let (s, v) = req(
        "GET",
        "/api/v1/extensions?show=all",
        Some(&b_all),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        data_len(&v) >= 1,
        "show=all with perm must see other-tenant rows: {v}"
    );

    // ...but default (no show param) stays scoped even with the perm
    let (s, v) = req(
        "GET",
        "/api/v1/extensions",
        Some(&b_all),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(data_len(&v), 0, "default list stays scoped with perm: {v}");

    // detail by id stays tenant-locked even for _all holders
    let (s, _) = req(
        "GET",
        &format!("/api/v1/extensions/{id}"),
        Some(&b_all),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "cross-tenant detail must 404");
    let b_del = token("tenant-b", &["extension_add", "extension_all"]);
    let (s, _) = req(
        "DELETE",
        &format!("/api/v1/extensions/{id}"),
        Some(&b_del),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::NOT_FOUND, "cross-tenant delete must 404");
}

#[tokio::test]
async fn show_all_gateways_users_contacts() {
    let admin = login_token().await;
    let gw = uniq("gw");
    let (s, v) = req(
        "POST",
        "/api/v1/gateways",
        Some(&admin),
        serde_json::json!({"gateway_name": gw, "proxy": "10.0.0.1"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create gw: {v}");

    for (path, all_perm, view_perm) in [
        ("/api/v1/gateways", "gateway_all", "gateway_view"),
        ("/api/v1/users", "user_all", "user_view"),
        ("/api/v1/contacts", "contact_all", "contact_view"),
    ] {
        let b = token("tenant-c", &[view_perm]);
        let (s, v) = req(
            "GET",
            &format!("{path}?show=all"),
            Some(&b),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{path} scoped ok");
        let scoped = data_len(&v);
        let b_all = token("tenant-c", &[view_perm, all_perm]);
        let (s, v) = req(
            "GET",
            &format!("{path}?show=all"),
            Some(&b_all),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{path} show=all ok: {v}");
        assert!(
            data_len(&v) >= scoped,
            "{path} show=all must be a superset: {v}"
        );
        // gateways: other tenant has rows (created above); users/contacts may
        // be empty cross-tenant, so only gateways asserts strictly.
        if path == "/api/v1/gateways" {
            assert!(data_len(&v) >= 1, "must see default-tenant gateway: {v}");
            assert_eq!(scoped, 0, "scoped gateway list hides others: {v}");
        }
    }
}

#[tokio::test]
async fn show_all_cdr_emergency_user_logs() {
    let admin = login_token().await;
    for (path, all_perm, view_perm) in [
        ("/api/v1/cdr", "xml_cdr_all", "cdr_view"),
        (
            "/api/v1/emergency_logs",
            "emergency_logs_view_all",
            "emergency_logs_view",
        ),
        ("/api/v1/user_logs", "user_log_all", "user_log_view"),
    ] {
        let b = token("tenant-d", &[view_perm]);
        let (s, _) = req("GET", path, Some(&b), serde_json::json!({})).await;
        assert_eq!(s, StatusCode::OK, "{path} scoped ok");
        let (s, _) = req(
            "GET",
            &format!("{path}?show=all"),
            Some(&b),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(
            s,
            StatusCode::OK,
            "{path} show=all without perm ok (scoped)"
        );
        let b_all = token("tenant-d", &[view_perm, all_perm]);
        let (s, v) = req(
            "GET",
            &format!("{path}?show=all"),
            Some(&b_all),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{path} show=all with perm ok: {v}");
        let _ = admin;
    }
}

#[tokio::test]
async fn show_all_broad_resource_sweep() {
    // Every PHP-verified `show=all` list: scoped by default, 200 with and
    // without the perm, superset with the perm, never an error.
    let cases = [
        ("/api/v1/dialplans", "dialplan_all", "dialplan_view"),
        ("/api/v1/devices", "device_all", "device_view"),
        (
            "/api/v1/device_profiles",
            "device_profile_all",
            "device_view",
        ),
        ("/api/v1/ring_groups", "ring_group_all", "ring_group_view"),
        ("/api/v1/ivr_menus", "ivr_menu_all", "ivr_menu_view"),
        ("/api/v1/voicemails", "voicemail_all", "voicemail_view"),
        ("/api/v1/recordings", "recording_all", "recording_view"),
        ("/api/v1/conferences", "conference_all", "conference_view"),
        (
            "/api/v1/conference_centers",
            "conference_center_all",
            "conference_center_view",
        ),
        (
            "/api/v1/call_centers",
            "call_center_all",
            "call_center_view",
        ),
        ("/api/v1/fax", "fax_extension_view_all", "fax_view"),
        ("/api/v1/fax/queue", "fax_queue_all", "fax_queue_view"),
        ("/api/v1/groups", "group_all", "group_view"),
        ("/api/v1/dashboards", "dashboard_all", "dashboard_view"),
        ("/api/v1/email_queue", "email_queue_all", "email_queue_view"),
        ("/api/v1/call_flows", "call_flow_all", "call_flow_view"),
        (
            "/api/v1/extension_settings",
            "extension_setting_all",
            "extension_setting_view",
        ),
        (
            "/api/v1/call_recordings",
            "call_recording_all",
            "call_recording_view",
        ),
        (
            "/api/v1/domain_settings",
            "domain_setting_all",
            "domain_setting_view",
        ),
    ];
    for (path, all_perm, view_perm) in cases {
        let b = token("tenant-e", &[view_perm]);
        let (s, v) = req("GET", path, Some(&b), serde_json::json!({})).await;
        assert_eq!(s, StatusCode::OK, "{path} scoped list: {v}");
        let scoped = data_len(&v);
        let (s, v) = req(
            "GET",
            &format!("{path}?show=all"),
            Some(&b),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{path} show=all w/o perm: {v}");
        assert_eq!(
            data_len(&v),
            scoped,
            "{path} show=all w/o perm stays scoped"
        );
        let b_all = token("tenant-e", &[view_perm, all_perm]);
        let (s, v) = req(
            "GET",
            &format!("{path}?show=all"),
            Some(&b_all),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{path} show=all with perm: {v}");
        assert!(data_len(&v) >= scoped, "{path} show=all is a superset: {v}");
        // unknown show values behave like scoped
        let (s, v) = req(
            "GET",
            &format!("{path}?show=everything"),
            Some(&b_all),
            serde_json::json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{path} show=bogus: {v}");
        assert_eq!(data_len(&v), scoped, "{path} show=bogus stays scoped");
    }
}
