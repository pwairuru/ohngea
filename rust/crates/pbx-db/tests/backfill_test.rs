//! Spillover backfill — forward-only `remaining_records` drain into the
//! per-table entities. Runs on sqlite memory: seed spillover rows, run the
//! backfill, assert per-table rows appear and spillover rows are drained.

use pbx_db::entities::{bridges, call_block, remaining_record};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

async fn seed_spillover(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
    resource: &str,
    id: &str,
    data: serde_json::Value,
) {
    let am = remaining_record::ActiveModel {
        id: Set(id.to_owned()),
        domain_id: Set(domain),
        resource: Set(resource.to_owned()),
        data: Set(data.to_string()),
        created_at: Set(chrono::Utc::now()),
    };
    am.insert(db).await.expect("seed spillover row");
}

#[tokio::test]
async fn backfill_drains_spillover_into_per_table_entities() {
    let db = pbx_db::connect_sqlite_memory().await.expect("memory db");
    pbx_db::create_schema(&db).await.expect("schema");

    let domain = uuid::Uuid::now_v7();
    let now = chrono::Utc::now();
    // Parent domain row (FK target on pg; harmless on sqlite).
    let _ = pbx_db::entities::domains::ActiveModel {
        domain_id: Set(domain),
        domain_name: Set("backfill-test".to_owned()),
        enabled: Set(true),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&db)
    .await;

    let cb_id = uuid::Uuid::now_v7().to_string();
    seed_spillover(
        &db,
        domain,
        "call_block",
        &cb_id,
        serde_json::json!({"call_block_number": "+15550001111", "call_block_name": "spam"}),
    )
    .await;
    let br_id = uuid::Uuid::now_v7().to_string();
    seed_spillover(
        &db,
        domain,
        "bridges",
        &br_id,
        serde_json::json!({"name": "br-backfill", "destination": "1001"}),
    )
    .await;
    let dst_id = uuid::Uuid::now_v7().to_string();
    seed_spillover(
        &db,
        domain,
        "destinations",
        &dst_id,
        serde_json::json!({"destination": "2001", "action": "bridge"}),
    )
    .await;
    // Unknown resource stays behind.
    let unk_id = uuid::Uuid::now_v7().to_string();
    seed_spillover(
        &db,
        domain,
        "mystery_resource",
        &unk_id,
        serde_json::json!({"a": 1}),
    )
    .await;

    let counts = pbx_db::migration::backfill_remaining_records(&db)
        .await
        .expect("backfill runs");
    assert_eq!(counts.migrated, 3, "three rows drained: {counts:?}");
    assert_eq!(counts.skipped, 1, "unknown resource skipped: {counts:?}");

    let cb = call_block::Entity::find_by_id(cb_id.clone())
        .one(&db)
        .await
        .expect("lookup")
        .expect("call_block row");
    assert_eq!(cb.call_block_number, "+15550001111");
    assert_eq!(cb.call_block_name, "spam");

    let br = bridges::Entity::find_by_id(br_id.clone())
        .one(&db)
        .await
        .expect("lookup")
        .expect("bridges row");
    assert_eq!(br.bridge_name, "br-backfill");

    // Spillover rows for migrated resources are gone; unknown remains.
    let left = remaining_record::Entity::find()
        .all(&db)
        .await
        .expect("list");
    assert_eq!(left.len(), 1, "only unknown remains: {left:?}");
    assert_eq!(left[0].resource, "mystery_resource");

    // Idempotent: second run migrates nothing new.
    let again = pbx_db::migration::backfill_remaining_records(&db)
        .await
        .expect("backfill rerun");
    assert_eq!(again.migrated, 0, "rerun drains nothing: {again:?}");
    assert_eq!(again.skipped, 1, "unknown still skipped: {again:?}");
}

#[tokio::test]
async fn backfill_empty_db_is_noop() {
    let db = pbx_db::connect_sqlite_memory().await.expect("memory db");
    pbx_db::create_schema(&db).await.expect("schema");
    let counts = pbx_db::migration::backfill_remaining_records(&db)
        .await
        .expect("backfill runs");
    assert_eq!(counts.migrated, 0);
    assert_eq!(counts.skipped, 0);
}
