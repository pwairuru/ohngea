use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use tower::ServiceExt;

// PHP parity for the assigned-only fallback tier:
// - fax: without fax_extension_view_domain/_all, a fax_view holder sees
//   only fax_users-assigned rows (app/fax/fax.php:110+).
// - ring_groups: without ring_group_domain/_all, only ring_group_users
//   rows (app/ring_groups/ring_groups.php:128+).
// - voicemails: without voicemail_domain, only the user's own mailboxes
//   (extension numbers via extension_users; app/voicemails/voicemails.php).
fn token(domain: &str, perms: &[&str]) -> String {
    pbx_core::auth::encode_claims(
        &format!("tester-{domain}"),
        domain,
        "tester",
        &perms.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    )
}

fn sub_of(domain: &str) -> String {
    format!("tester-{domain}")
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

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
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
async fn fax_assigned_only_without_domain_perm() {
    // Same tenant for creator and viewer (PHP tier 3 is same-domain +
    // assigned). ensure_tenant materializes the domains row.
    ensure_tenant("asg-tenant", &["extension_add"]).await;
    let a = token("asg-tenant", &["fax_add", "fax_view"]);
    let fname = uniq("asfax");
    let (s, v) = req(
        "POST",
        "/api/v1/fax",
        Some(&a),
        serde_json::json!({"fax_extension": "8101", "fax_name": fname}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create fax: {v}");
    let fid = v
        .get("fax_uuid")
        .and_then(|i| i.as_str())
        .unwrap()
        .to_owned();

    // Plain fax_view holder sees nothing (not assigned).
    let b = token("asg-tenant", &["fax_view"]);
    let (s, v) = req("GET", "/api/v1/fax", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        v.get("total").and_then(|t| t.as_u64()).unwrap_or(999),
        0,
        "unassigned fax must be hidden: {v}"
    );

    // Assign B via fax_users; now visible.
    let db = pbx_api::routing_db::conn().await;
    pbx_db::entities::fax_users::Entity::insert(pbx_db::entities::fax_users::ActiveModel {
        fax_user_uuid: Set(uuid::Uuid::now_v7().to_string()),
        domain_id: Set(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
        fax_uuid: Set(fid.clone()),
        user_uuid: Set(Some(sub_of("asg-tenant"))),
        created_at: Set(chrono::Utc::now()),
    })
    .exec(db)
    .await
    .unwrap();
    let (s, v) = req("GET", "/api/v1/fax", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let ids: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("fax_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(ids.contains(&fid), "assigned fax visible: {v}");
}

#[tokio::test]
async fn ring_group_assigned_only_without_domain_perm() {
    ensure_tenant("asr-tenant", &["extension_add"]).await;
    let a = token("asr-tenant", &["ring_group_add", "ring_group_view"]);
    let (s, v) = req(
        "POST",
        "/api/v1/ring_groups",
        Some(&a),
        serde_json::json!({"name": uniq("asrg"), "extension": "8201"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "create rg: {v}");
    let rid = v
        .get("ring_group_uuid")
        .and_then(|i| i.as_str())
        .expect("rg id")
        .to_owned();

    let b = token("asr-tenant", &["ring_group_view"]);
    let (s, v) = req(
        "GET",
        "/api/v1/ring_groups",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        v.get("total").and_then(|t| t.as_u64()).unwrap_or(999),
        0,
        "unassigned rg hidden: {v}"
    );

    let db = pbx_api::routing_db::conn().await;
    pbx_db::entities::ring_group_users::Entity::insert(
        pbx_db::entities::ring_group_users::ActiveModel {
            ring_group_user_uuid: Set(uuid::Uuid::now_v7().to_string()),
            domain_id: Set(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
            ring_group_uuid: Set(rid.clone()),
            user_uuid: Set(sub_of("asr-tenant")),
        },
    )
    .exec(db)
    .await
    .unwrap();
    let (s, v) = req(
        "GET",
        "/api/v1/ring_groups",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let ids: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("ring_group_uuid")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(ids.contains(&rid), "assigned rg visible: {v}");
}

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
    assert_eq!(s, StatusCode::CREATED, "bootstrap tenant {domain}: {v}");
}

#[tokio::test]
async fn voicemail_own_mailboxes_without_domain_perm() {
    ensure_tenant("asv-tenant", &["extension_add"]).await;
    let a = token(
        "asv-tenant",
        &[
            "extension_add",
            "extension_view",
            "voicemail_add",
            "voicemail_view",
        ],
    );
    // B owns extension 8301.
    let b = token("asv-tenant", &["voicemail_view"]);
    let (s, v) = req(
        "POST",
        "/api/v1/extensions",
        Some(&a),
        serde_json::json!({"extension": "8301", "password": "pw123456"}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "B extension: {v}");
    let b_ext = v
        .get("extension_uuid")
        .and_then(|i| i.as_str())
        .unwrap()
        .to_owned();
    // Link the extension to B (PHP resolves own mailboxes via
    // extension_users; Rust create does not auto-link).
    let db = pbx_api::routing_db::conn().await;
    pbx_db::entities::extension_users::Entity::insert(
        pbx_db::entities::extension_users::ActiveModel {
            extension_user_uuid: Set(uuid::Uuid::now_v7().to_string()),
            domain_id: Set(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()),
            extension_uuid: Set(b_ext),
            user_uuid: Set(sub_of("asv-tenant")),
        },
    )
    .exec(db)
    .await
    .unwrap();

    // A mailbox for B's own number is visible via assignment;
    // someone else's number stays hidden.
    for (box_id, who) in [("8301", "B"), ("8302", "other")] {
        let (s, _) = req(
            "POST",
            "/api/v1/voicemails",
            Some(&a),
            serde_json::json!({"voicemail_id": box_id, "voicemail_password": "1234"}),
        )
        .await;
        assert_eq!(s, StatusCode::CREATED, "{who} mailbox");
    }

    let (s, v) = req("GET", "/api/v1/voicemails", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let ids: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("voicemail_id")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        ids.contains(&"8301".to_string()),
        "own mailbox visible: {ids:?}"
    );
    assert!(
        !ids.contains(&"8302".to_string()),
        "other mailbox hidden: {ids:?}"
    );
}
