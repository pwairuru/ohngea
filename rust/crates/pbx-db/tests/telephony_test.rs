//! Batch 2a telephony DB entities + seeds — TDD RED.
//! Ground truth:
//! - app/extensions/models.py:12-81 (extension 2-7 digit regex, 60 cols)
//! - app/extensions/router.py + service.py (password auto-gen 20 chars,
//!   user_context default domain_name)
//! - app/dialplans/models.py:10-121 + app/seeds/dialplans.py (4 dialplans x 2 details)
//! - app/gateways/models.py:11-62 (36 fields, `register` column alias,
//!   defaults expire 800 / retry 30 / context public / profile external)
//! - app/sip_profiles/models.py:10-74 + app/seeds/sip_profiles.py
//!   (4 profiles, 4 domains, 335 settings)
//! - app/freeswitch/sofia_config.py (145 lines, `_build_sofia_xml` verbatim)
//! - app/devices/models.py (provisioning models)

use pbx_db::entities::dialplan_details::{self, Entity as DialplanDetail};
use pbx_db::entities::dialplans::{self, Entity as Dialplan};
use pbx_db::entities::extensions;
use pbx_db::entities::gateways;
use pbx_db::entities::sip_profile_domains::{self, Entity as SipDomain};
use pbx_db::entities::sip_profile_settings::{self, Entity as SipSetting};
use pbx_db::entities::sip_profiles::{self, Entity as SipProfile};
use pbx_db::extensions::{ensure_password, generate_password, validate_extension};
use pbx_db::seeds::{seed_dialplans, seed_sip_profiles, DIALPLAN_SEED, SIP_PROFILE_SEED};
use pbx_db::{connect_sqlite_memory, create_schema};
use pbx_fs::sofia::{build_sofia_xml, SofiaProfileInput};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

// --- extension validation: app/extensions/models.py:15-20 ---

#[test]
fn extension_rejects_non_2_to_7_digits() {
    for bad in ["", "1", "12345678", "abc", "10a1", " 100", "1001 "] {
        assert!(
            validate_extension(bad).is_err(),
            "{bad:?} must be rejected (2-7 digit regex ^\\d{{2,7}}$)"
        );
    }
    for good in ["10", "1001", "1234567"] {
        assert!(
            validate_extension(good).is_ok(),
            "{good:?} must be accepted"
        );
    }
}

// --- password auto-gen: app/extensions/service.py generate_password(20) ---

#[test]
fn password_autogen_is_20_chars_when_blank() {
    let pwd = generate_password(20);
    assert_eq!(pwd.len(), 20, "auto-gen password must be 20 chars");
    assert_eq!(ensure_password(None).len(), 20);
    assert_eq!(ensure_password(Some("")).len(), 20);
    assert_eq!(ensure_password(Some("keepme")), "keepme");
}

// --- gateway defaults: app/gateways/models.py ---

#[test]
fn gateway_defaults_match_python() {
    let am = gateways::ActiveModel::default();
    // SeaORM ActiveModel defaults are NotSet; check entity-level helper instead
    assert_eq!(pbx_db::gateways::default_expire_seconds(), 800);
    assert_eq!(pbx_db::gateways::default_retry_seconds(), 30);
    assert_eq!(pbx_db::gateways::default_context(), "public");
    assert_eq!(pbx_db::gateways::default_profile(), "external");
    let _ = am;
}

#[tokio::test]
async fn gateway_register_column_is_named_register() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    use sea_orm::{ConnectionTrait, Statement};
    let row = db
        .query_one_raw(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT sql FROM sqlite_master WHERE name='gateways'".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    let ddl: String = row.try_get_by_index(0).unwrap();
    assert!(
        ddl.contains("\"register\"") || ddl.contains("register BOOLEAN"),
        "DB column must stay `register` (property alias gateway_register), got {ddl}"
    );
}

// --- extension entity columns mirror Python ---

#[test]
fn extension_entity_has_exact_python_columns() {
    use sea_orm::{EntityTrait, Iterable};
    let cols: Vec<String> = <extensions::Entity as EntityTrait>::Column::iter()
        .map(|c| format!("{c:?}"))
        .collect();
    for want in [
        "ExtensionUuid",
        "DomainId",
        "Extension",
        "NumberAlias",
        "Password",
        "UserContext",
        "CallTimeout",
        "Enabled",
    ] {
        assert!(
            cols.iter().any(|c| c == want),
            "missing {want}, got {cols:?}"
        );
    }
}

// --- dialplan seeds: 4 dialplans x 2 details ---

#[test]
fn dialplan_seed_has_4_with_2_details_each() {
    assert_eq!(DIALPLAN_SEED.len(), 4);
    for d in DIALPLAN_SEED {
        assert_eq!(d.details.len(), 2, "{} must have 2 details", d.name);
    }
}

#[tokio::test]
async fn dialplan_parent_plus_details_cascade_loads_4_plus_8() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    let domain_id = Uuid::now_v7();
    pbx_db::seeds::seed_domains(&db, domain_id, "tenant.example.com")
        .await
        .unwrap();

    let n = seed_dialplans(&db, domain_id, "tenant.example.com")
        .await
        .unwrap();
    assert_eq!(n, 12, "4 dialplans + 8 details = 12 rows, got {n}");
    assert_eq!(Dialplan::find().all(&db).await.unwrap().len(), 4);
    assert_eq!(DialplanDetail::find().all(&db).await.unwrap().len(), 8);

    // idempotent reseed inserts nothing
    assert_eq!(
        seed_dialplans(&db, domain_id, "tenant.example.com")
            .await
            .unwrap(),
        0
    );

    // parent detail rows point at a real dialplan_uuid
    let dps = Dialplan::find().all(&db).await.unwrap();
    let local = dps.iter().find(|d| d.name == "local_extension").unwrap();
    let dets = DialplanDetail::find()
        .filter(dialplan_details::Column::DialplanUuid.eq(local.dialplan_uuid))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(dets.len(), 2);
    assert_eq!(
        seed_dialplans(&db, domain_id, "tenant.example.com")
            .await
            .unwrap(),
        0
    );
    let _ = dialplans::Column::Name;
}

// --- sip seeds: 4 profiles, 4 domains, 335 settings ---

#[test]
fn sip_seed_counts_match_python() {
    assert_eq!(SIP_PROFILE_SEED.len(), 4, "must mirror SIP_PROFILES count");
    let domains: usize = SIP_PROFILE_SEED.iter().map(|p| p.domains.len()).sum();
    let settings: usize = SIP_PROFILE_SEED.iter().map(|p| p.settings.len()).sum();
    assert_eq!(domains, 4);
    assert_eq!(settings, 335);
}

#[tokio::test]
async fn sip_seed_loads_4_4_335_and_dedupes() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    let domain_id = Uuid::now_v7();
    pbx_db::seeds::seed_domains(&db, domain_id, "tenant.example.com")
        .await
        .unwrap();

    let n = seed_sip_profiles(&db, domain_id).await.unwrap();
    assert_eq!(
        n,
        4 + 4 + 335,
        "4 profiles + 4 domains + 335 settings, got {n}"
    );
    assert_eq!(SipProfile::find().all(&db).await.unwrap().len(), 4);
    assert_eq!(SipDomain::find().all(&db).await.unwrap().len(), 4);
    assert_eq!(SipSetting::find().all(&db).await.unwrap().len(), 335);

    // reseed is idempotent
    assert_eq!(seed_sip_profiles(&db, domain_id).await.unwrap(), 0);

    // child rows reference a real parent profile
    let profiles = SipProfile::find().all(&db).await.unwrap();
    let internal = profiles.iter().find(|p| p.name == "internal").unwrap();
    assert_eq!(internal.sip_port, 5060);
    let kids = SipSetting::find()
        .filter(sip_profile_settings::Column::SipProfileUuid.eq(internal.sip_profile_uuid))
        .all(&db)
        .await
        .unwrap();
    assert!(!kids.is_empty());
    let doms = SipDomain::find()
        .filter(sip_profile_domains::Column::SipProfileUuid.eq(internal.sip_profile_uuid))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(doms.len(), 1);
    let _ = sip_profiles::Column::Name;
}

#[tokio::test]
async fn devices_provisioning_tables_exist_and_link() {
    use pbx_db::entities::{device_lines, devices};
    use sea_orm::Set;
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    let domain_id = Uuid::now_v7();
    pbx_db::seeds::seed_domains(&db, domain_id, "tenant.example.com")
        .await
        .unwrap();
    let now = chrono::Utc::now();
    let dev_id = Uuid::now_v7();
    devices::Entity::insert(devices::ActiveModel {
        device_uuid: Set(dev_id),
        domain_id: Set(domain_id),
        mac: Set("00:11:22:33:44:55".to_owned()),
        vendor: Set("yealink".to_owned()),
        enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    })
    .exec(&db)
    .await
    .unwrap();
    device_lines::Entity::insert(device_lines::ActiveModel {
        device_line_uuid: Set(Uuid::now_v7()),
        domain_id: Set(domain_id),
        device_uuid: Set(dev_id),
        enabled: Set(true),
        created_at: Set(now),
        ..Default::default()
    })
    .exec(&db)
    .await
    .unwrap();
    assert_eq!(devices::Entity::find().all(&db).await.unwrap().len(), 1);
    assert_eq!(
        device_lines::Entity::find()
            .filter(device_lines::Column::DeviceUuid.eq(dev_id))
            .all(&db)
            .await
            .unwrap()
            .len(),
        1
    );
}

// --- sip save verbatim: app/freeswitch/sofia_config.py _build_sofia_xml ---

#[test]
fn sofia_xml_fallback_matches_python_defaults() {
    let xml = build_sofia_xml(&[]);
    // fallback renders internal + internal-test like Python when profiles == []
    assert!(
        xml.contains("profile name=\"internal\""),
        "fallback internal:\n{xml}"
    );
    assert!(xml.contains("profile name=\"internal-test\""));
    for param in [
        "param name=\"sip-ip\" value=\"0.0.0.0\"",
        "param name=\"dialplan\" value=\"XML\"",
        "param name=\"challenge-realm\" value=\"auto_from\"",
        "param name=\"accept-blind-reg\" value=\"false\"",
        "param name=\"inbound-late-negotiation\" value=\"true\"",
        "param name=\"apply-inbound-acl\" value=\"e2e-all\"",
        "param name=\"local-network-acl\" value=\"localnet.auto\"",
    ] {
        assert!(xml.contains(param), "missing {param}:\n{xml}");
    }
}

#[test]
fn sofia_xml_db_profile_mirrors_python_field_mapping() {
    let p = SofiaProfileInput {
        name: "internal".to_owned(),
        sip_port: 5060,
        sip_ip: None, // -> 0.0.0.0
        ext_sip_ip: None,
        ext_rtp_ip: None,
        rtp_ip: None,
        realm: Some("tenant.example.com".to_owned()),
        context: None,           // -> realm
        challenge_realm: None,   // -> auto_from
        apply_inbound_acl: None, // -> e2e-all
        local_network_acl: None, // -> localnet.auto
        hold_music: Some("$${hold_music}".to_owned()),
        record_path: None,
    };
    let xml = build_sofia_xml(std::slice::from_ref(&p));
    for param in [
        "param name=\"sip-ip\" value=\"0.0.0.0\"",
        "param name=\"context\" value=\"tenant.example.com\"",
        "param name=\"realm\" value=\"tenant.example.com\"",
        "param name=\"challenge-realm\" value=\"auto_from\"",
        "param name=\"hold-music\" value=\"$${hold_music}\"",
    ] {
        assert!(xml.contains(param), "missing {param}:\n{xml}");
    }
    assert!(
        !xml.contains("record-path"),
        "record-path must be omitted when None:\n{xml}"
    );
}
