//! Batch 1 settings data layer — TDD RED.
//! Ground truth: app/seeds/vars.py (68 rows), app/seeds/sofia.py (6 rows),
//! app/seeds/default_settings.py (1338 rows), app/seeds/domain_settings.py (14 rows).

use pbx_db::entities::default_settings::{self, Entity as DefaultSettings};
use pbx_db::entities::domain_settings::{self, Entity as DomainSettings};
use pbx_db::entities::sofia_global_settings::{self, Entity as SofiaSettings};
use pbx_db::entities::vars::{self, Entity as Vars};
use pbx_db::seeds::{
    find_default_setting, resolve_setting, seed_default_settings, seed_domain_settings, seed_sofia,
    seed_vars, DEFAULT_SETTINGS_SEED, DOMAIN_DEFAULTS_SEED, SOFIA_SEED, VARS_SEED,
};
use pbx_db::tenant::{set_tenant, set_tenant_sql};
use pbx_db::{connect_sqlite_memory, create_schema, paginate, Page};
use sea_orm::{ColumnTrait, EntityTrait, Iterable, QueryFilter};
use uuid::Uuid;

fn now_ts() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

#[test]
fn vars_seed_has_68_rows_matching_python() {
    assert_eq!(
        VARS_SEED.len(),
        68,
        "must mirror app/seeds/vars.py VARS count"
    );
}

#[test]
fn sofia_seed_has_6_rows_matching_python() {
    assert_eq!(SOFIA_SEED.len(), 6, "must mirror app/seeds/sofia.py count");
}

#[test]
fn domain_defaults_seed_has_14_rows_matching_python() {
    assert_eq!(
        DOMAIN_DEFAULTS_SEED.len(),
        14,
        "must mirror app/seeds/domain_settings.py DOMAIN_DEFAULTS count"
    );
}

#[test]
fn default_settings_seed_is_representative_subset() {
    assert!(
        DEFAULT_SETTINGS_SEED.len() >= 50,
        "representative subset needs >=50 rows, got {}",
        DEFAULT_SETTINGS_SEED.len()
    );
    // provision + theme must dominate like Python (948 + 66 of 1338)
    let cats: Vec<&str> = DEFAULT_SETTINGS_SEED.iter().map(|r| r.category).collect();
    assert!(cats.contains(&"provision"));
    assert!(cats.contains(&"theme"));
}

#[test]
fn paginate_maps_page_to_limit_offset() {
    assert_eq!(paginate(1, 20), (20, 0));
    assert_eq!(paginate(3, 10), (10, 20));
    // page 0 clamps to first page (offset 0, no underflow)
    assert_eq!(paginate(0, 20), (20, 0));
    // per_page capped at 100 like router Query(le=100)
    assert_eq!(paginate(1, 500), (100, 0));
}

#[test]
fn page_envelope_carries_data_and_total() {
    let p = Page::new(vec![1, 2], 68, 1, 20);
    assert_eq!(p.data.len(), 2);
    assert_eq!(p.total, 68);
    assert_eq!(p.page, 1);
    assert_eq!(p.per_page, 20);
}

#[test]
fn set_tenant_sql_emits_set_local_app_domain_id() {
    let id = "11111111-1111-4111-8111-111111111111";
    assert_eq!(
        set_tenant_sql(id),
        format!("SET LOCAL app.domain_id = '{id}'")
    );
}

#[tokio::test]
async fn vars_68_rows_loadable_and_dedupe_key_enforced() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    let domain_id = Uuid::now_v7();

    let n = seed_vars(&db, domain_id).await.unwrap();
    assert_eq!(n, 68);
    assert_eq!(Vars::find().all(&db).await.unwrap().len(), 68);

    // idempotent reseed inserts nothing
    let n2 = seed_vars(&db, domain_id).await.unwrap();
    assert_eq!(n2, 0);

    // same var_name in a DIFFERENT category is allowed (dedupe key has 3 cols)
    let now = now_ts();
    let dup_cat = vars::ActiveModel {
        var_uuid: sea_orm::Set(Uuid::now_v7()),
        domain_id: sea_orm::Set(domain_id),
        var_category: sea_orm::Set("Other".to_owned()),
        var_name: sea_orm::Set(VARS_SEED[0].var_name.to_owned()),
        var_value: sea_orm::Set("x".to_owned()),
        var_enabled: sea_orm::Set(true),
        created_at: sea_orm::Set(now),
        updated_at: sea_orm::Set(now),
        ..Default::default()
    };
    vars::Entity::insert(dup_cat).exec(&db).await.unwrap();

    // exact (domain_id, var_name, var_category) duplicate is rejected
    let dup = vars::ActiveModel {
        var_uuid: sea_orm::Set(Uuid::now_v7()),
        domain_id: sea_orm::Set(domain_id),
        var_category: sea_orm::Set(VARS_SEED[0].var_category.to_owned()),
        var_name: sea_orm::Set(VARS_SEED[0].var_name.to_owned()),
        var_value: sea_orm::Set("x".to_owned()),
        var_enabled: sea_orm::Set(true),
        created_at: sea_orm::Set(now),
        updated_at: sea_orm::Set(now),
        ..Default::default()
    };
    assert!(
        vars::Entity::insert(dup).exec(&db).await.is_err(),
        "dedupe key (domain_id, var_name, var_category) must be enforced"
    );
}

#[tokio::test]
async fn sofia_6_rows_global_no_domain_and_unique_name() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();

    let n = seed_sofia(&db).await.unwrap();
    assert_eq!(n, 6);

    // global table: no domain_id column
    let cols: Vec<String> = <SofiaSettings as EntityTrait>::Column::iter()
        .map(|c| format!("{c:?}"))
        .collect();
    assert!(
        !cols.iter().any(|c| c.contains("domain")),
        "sofia_global_settings must be global (no domain_id), got {cols:?}"
    );

    // UNIQUE name enforced
    let now = now_ts();
    let dup = sofia_global_settings::ActiveModel {
        sofia_global_setting_uuid: sea_orm::Set(Uuid::now_v7()),
        global_setting_name: sea_orm::Set(SOFIA_SEED[0].name.to_owned()),
        global_setting_value: sea_orm::Set("x".to_owned()),
        global_setting_enabled: sea_orm::Set(true),
        created_at: sea_orm::Set(now),
        updated_at: sea_orm::Set(now),
        ..Default::default()
    };
    assert!(
        SofiaSettings::insert(dup).exec(&db).await.is_err(),
        "global_setting_name must be UNIQUE"
    );
}

#[tokio::test]
async fn default_settings_lookup_by_triple() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    seed_default_settings(&db).await.unwrap();

    // DB-level lookup by (category, subcategory, name)
    let row = DefaultSettings::find()
        .filter(default_settings::Column::DefaultSettingCategory.eq("provision"))
        .filter(default_settings::Column::DefaultSettingSubcategory.eq("aastra_gmt_offset"))
        .filter(default_settings::Column::DefaultSettingName.eq("numeric"))
        .one(&db)
        .await
        .unwrap()
        .expect("provision/aastra_gmt_offset/numeric must exist");
    assert_eq!(row.default_setting_value.as_deref(), Some("0"));

    // pure in-memory triple lookup agrees
    let hit = find_default_setting("provision", "aastra_gmt_offset", "numeric").unwrap();
    assert_eq!(hit.value, Some("0"));
    assert!(find_default_setting("nope", "nope", "nope").is_none());
}

#[tokio::test]
async fn domain_override_wins_over_default() {
    let db = connect_sqlite_memory().await.unwrap();
    create_schema(&db).await.unwrap();
    seed_default_settings(&db).await.unwrap();
    let domain_id = Uuid::now_v7();
    seed_domain_settings(&db, domain_id).await.unwrap();

    // domain seed overrides theme primary_color default
    let val = resolve_setting(&db, domain_id, "theme", "", "primary_color")
        .await
        .unwrap()
        .expect("theme primary_color must resolve");
    let domain_val = DomainSettings::find()
        .filter(domain_settings::Column::DomainId.eq(domain_id))
        .filter(domain_settings::Column::Category.eq("theme"))
        .filter(domain_settings::Column::Key.eq("primary_color"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val, domain_val.value);

    // no domain row -> falls back to global default
    let other = Uuid::now_v7();
    let fb = resolve_setting(&db, other, "provision", "aastra_gmt_offset", "numeric")
        .await
        .unwrap()
        .expect("must fall back to default_settings");
    assert_eq!(fb, "0");

    // unknown triple -> None
    assert!(resolve_setting(&db, other, "nope", "nope", "nope")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn set_tenant_executes_set_local_statement() {
    use sea_orm::{DatabaseBackend, MockDatabase};
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();
    let id = "11111111-1111-4111-8111-111111111111";
    set_tenant(&db, id).await.unwrap();
    let log = db.into_transaction_log();
    assert_eq!(log.len(), 1);
    assert!(
        format!("{:?}", log[0]).contains("SET LOCAL app.domain_id"),
        "must emit SET LOCAL app.domain_id, got {log:?}"
    );
}
