use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sea_orm::ConnectionTrait;
use tower::ServiceExt;

// PHP ground truth: v_* list pages include NULL-domain (global/shared) rows —
// unconditionally for most tables, gated behind `*_domain`/`*_global` perms
// for gateways/call_block/dialplans/music_on_hold. Rust entities carry
// nullable `domain_id`; scoped lists OR NULL; detail stays tenant-locked
// except globals (visible per the same rule).

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

fn names_of(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get(key)?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

async fn insert_global(table: &str, pk: &str, id: &str, cols: &str, vals: &str) {
    let db = pbx_api::routing_db::conn().await;
    let sql = format!(
        "INSERT INTO {table} ({pk}, domain_id, created_at, updated_at{cols}) \
         VALUES ('{id}', NULL, '2026-01-01 00:00:00', '2026-01-01 00:00:00'{vals})"
    );
    db.execute_unprepared(&sql).await.expect("insert global");
}

fn uniq(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::now_v7().simple())
}

#[tokio::test]
async fn global_rows_visible_unconditionally() {
    // Seed one legacy global row per unconditionally-global table.
    let g_dest = uniq("1001");
    insert_global(
        "destinations",
        "destination_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", destination_type, destination_number",
        &format!(", 'inbound', '{g_dest}'"),
    )
    .await;
    let g_ivr = uniq("ivrglobal");
    insert_global(
        "ivr_menus",
        "ivr_menu_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", name",
        &format!(", '{g_ivr}'"),
    )
    .await;
    insert_global(
        "devices",
        "device_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", mac, vendor",
        &format!(",'00:11:22:33:44:{:02x}', 'yealink'", rand_byte()),
    )
    .await;
    let g_vm = uniq("2001");
    insert_global(
        "voicemails",
        "voicemail_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", voicemail_id, voicemail_password",
        &format!(", '{g_vm}', '1234'"),
    )
    .await;
    let g_conf = uniq("confglobal");
    insert_global(
        "conferences",
        "conference_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", conference_name, conference_extension",
        &format!(", '{g_conf}', '3001'"),
    )
    .await;
    let g_fax = uniq("faxglobal");
    insert_global(
        "fax",
        "fax_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", fax_extension, fax_name",
        &format!(", '4001', '{g_fax}'"),
    )
    .await;
    let g_tc = uniq("tcglobal");
    insert_global(
        "time_conditions",
        "time_condition_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", name, extension",
        &format!(", '{g_tc}', '5001'"),
    )
    .await;
    let g_cf = uniq("cfglobal");
    insert_global(
        "call_flows",
        "call_flow_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", call_flow_name, call_flow_extension",
        &format!(", '{g_cf}', '6001'"),
    )
    .await;
    // music_on_hold is created via API in the gated test below (needs path).
    let g_ph = uniq("phglobal");
    insert_global(
        "phrases",
        "phrase_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", phrase_name",
        &format!(", '{g_ph}'"),
    )
    .await;
    let g_st = uniq("stglobal");
    insert_global(
        "streams",
        "stream_uuid",
        &uuid::Uuid::now_v7().to_string(),
        ", stream_name",
        &format!(", '{g_st}'"),
    )
    .await;

    // A foreign tenant with only base view perms sees every global row.
    let b = token("global-reader", &["destination_view"]);
    let (s, v) = req(
        "GET",
        "/api/v1/destinations",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        names_of(&v, "destination_number").contains(&g_dest),
        "global destination visible: {v}"
    );

    let b = token("global-reader", &["ivr_menu_view"]);
    let (s, v) = req("GET", "/api/v1/ivr_menus", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("name")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(got.contains(&g_ivr), "global ivr visible: {v}");

    let b = token("global-reader", &["device_view"]);
    let (s, v) = req("GET", "/api/v1/devices", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "devices list: {v}");

    let b = token("global-reader", &["voicemail_view", "voicemail_domain"]);
    let (s, v) = req("GET", "/api/v1/voicemails", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        names_of(&v, "voicemail_id").contains(&g_vm),
        "global voicemail visible with domain perm: {v}"
    );

    let b = token("global-reader", &["conference_view"]);
    let (s, v) = req(
        "GET",
        "/api/v1/conferences",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "conferences list: {v}");

    let b = token("global-reader", &["fax_view"]);
    let (s, v) = req("GET", "/api/v1/fax", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "fax list: {v}");

    let b = token("global-reader", &["time_condition_view"]);
    let (s, v) = req(
        "GET",
        "/api/v1/time_conditions",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        names_of(&v, "name").contains(&g_tc),
        "global time_condition visible: {v}"
    );

    let b = token("global-reader", &["call_flow_view"]);
    let (s, v) = req("GET", "/api/v1/call_flows", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "call_flows list: {v}");

    let b = token("global-reader", &["phrase_view"]);
    let (s, v) = req("GET", "/api/v1/phrases", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "phrases list: {v}");

    let b = token("global-reader", &["stream_view"]);
    let (s, v) = req("GET", "/api/v1/streams", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK, "streams list: {v}");
}

fn rand_byte() -> u8 {
    use std::time::{SystemTime, UNIX_EPOCH};
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42)
        % 250) as u8
        + 5
}

#[tokio::test]
async fn gated_tables_need_domain_perm_for_globals() {
    // call_block: global via API with the perm; visible only with the perm.
    let a = token(
        "cb-tenant-a",
        &["call_block_add", "call_block_domain", "call_block_view"],
    );
    let blk = uniq("999");
    let (s, v) = req(
        "POST",
        "/api/v1/call_block",
        Some(&a),
        serde_json::json!({"call_block_number": blk, "domain_uuid": ""}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "global call_block: {v}");

    let b = token("cb-tenant-b", &["call_block_view"]);
    let (s, v) = req("GET", "/api/v1/call_block", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        !names_of(&v, "call_block_number").contains(&blk),
        "hidden without perm: {v}"
    );
    let b_dom = token("cb-tenant-b", &["call_block_view", "call_block_domain"]);
    let (s, v) = req(
        "GET",
        "/api/v1/call_block",
        Some(&b_dom),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        names_of(&v, "call_block_number").contains(&blk),
        "visible with perm: {v}"
    );

    // dialplans: same pattern via dialplan_domain / dialplan_global.
    let a = token(
        "dp-tenant-a",
        &["dialplan_add", "dialplan_domain", "dialplan_view"],
    );
    let dpname = uniq("dpglobal");
    let (s, v) = req(
        "POST",
        "/api/v1/dialplans",
        Some(&a),
        serde_json::json!({"name": dpname, "domain_id": ""}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "global dialplan: {v}");
    let b = token("dp-tenant-b", &["dialplan_view"]);
    let (s, v) = req("GET", "/api/v1/dialplans", Some(&b), serde_json::json!({})).await;
    assert_eq!(s, StatusCode::OK);
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("name")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        !got.contains(&dpname),
        "global dialplan hidden w/o perm: {got:?}"
    );
    let b_all = token("dp-tenant-b", &["dialplan_view", "dialplan_global"]);
    let (s, v) = req(
        "GET",
        "/api/v1/dialplans",
        Some(&b_all),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("name")?.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        got.contains(&dpname),
        "global dialplan visible with perm: {got:?}"
    );

    // music_on_hold: same pattern.
    let a = token(
        "moh-tenant-a",
        &[
            "music_on_hold_add",
            "music_on_hold_domain",
            "music_on_hold_view",
        ],
    );
    let mohname = uniq("mohglobal");
    let (s, v) = req(
        "POST",
        "/api/v1/music_on_hold",
        Some(&a),
        serde_json::json!({"name": mohname, "domain_uuid": ""}),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED, "global moh: {v}");
    let b = token("moh-tenant-b", &["music_on_hold_view"]);
    let (s, v) = req(
        "GET",
        "/api/v1/music_on_hold",
        Some(&b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "moh list w/o perm: {v}");
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| {
                    r.get("music_name")
                        .or_else(|| r.get("name"))?
                        .as_str()
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        !got.contains(&mohname),
        "global moh hidden w/o perm: {got:?}"
    );
    let b_dom = token(
        "moh-tenant-b",
        &["music_on_hold_view", "music_on_hold_global"],
    );
    let (s, v) = req(
        "GET",
        "/api/v1/music_on_hold",
        Some(&b_dom),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "moh list with perm: {v}");
    let got: Vec<String> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|r| {
                    r.get("music_name")
                        .or_else(|| r.get("name"))?
                        .as_str()
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        got.contains(&mohname),
        "global moh visible with perm: {got:?}"
    );
}
