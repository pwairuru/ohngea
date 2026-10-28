//! Shared SeaORM connection for call-routing routers.
//!
//! sqlite `:memory:` for tests (single connection so the DB is shared),
//! postgres via `DATABASE_URL` at runtime. One global connection per
//! process — `test_app()` builds a fresh router per request, so per-router
//! state would lose rows between POST and GET in tests.

use sea_orm::DatabaseConnection;
use tokio::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

async fn init_db() -> DatabaseConnection {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let mut opt = sea_orm::ConnectOptions::new(url);
        opt.max_connections(5);
        let db = sea_orm::Database::connect(opt)
            .await
            .expect("DATABASE_URL connect");
        pbx_db::create_schema(&db).await.expect("schema");
        return db;
    }
    // File-backed sqlite (NOT :memory:): each #[tokio::test] gets its own
    // runtime, and runtimes shut down between tests — a :memory: DB dies
    // with its creating runtime/connection ("no such table" on next test).
    // File persists across connections; pid in name isolates processes.
    let path = std::env::temp_dir().join(format!("pbx-routing-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let mut opt = sea_orm::ConnectOptions::new(url);
    opt.max_connections(1);
    let db = sea_orm::Database::connect(opt).await.expect("sqlite file");
    pbx_db::create_schema(&db).await.expect("schema");
    db
}

/// Shared connection (initialized once, schema bootstrapped).
pub async fn conn() -> &'static DatabaseConnection {
    DB.get_or_init(init_db).await
}

/// Tenant domain from verified JWT claims as Uuid (RULING: domain_id unified
/// to Uuid across batch1/2a/3a for RLS join parity).
/// Test tokens carry opaque strings ("default"); those map deterministically
/// via UUIDv5 (DNS namespace) so tenant isolation still holds in tests.
/// Prod tokens carry real UUID strings and parse directly.
pub fn domain(claims: &pbx_core::auth::Claims) -> uuid::Uuid {
    if let Ok(id) = claims.domain_id.parse::<uuid::Uuid>() {
        return id;
    }
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, claims.domain_id.as_bytes())
}

/// Tenant domain from verified JWT claims as Uuid, ensuring the parent
/// `domains` row exists (FK targets in batch1/2a/3a reference it).
/// Name: real UUID claims get `tenant-{uuid}`; opaque test strings ("default")
/// are kept as the domain_name for readability.
pub async fn tenant_domain(
    db: &sea_orm::DatabaseConnection,
    claims: &pbx_core::auth::Claims,
) -> uuid::Uuid {
    let d = domain(claims);
    let name = if claims.domain_id.parse::<uuid::Uuid>().is_ok() {
        format!("tenant-{d}")
    } else {
        claims.domain_id.clone()
    };
    let now = now();
    let am = pbx_db::entities::domains::ActiveModel {
        domain_id: sea_orm::Set(d),
        domain_name: sea_orm::Set(name),
        enabled: sea_orm::Set(true),
        description: sea_orm::Set(None),
        created_at: sea_orm::Set(now),
        updated_at: sea_orm::Set(now),
    };
    use sea_orm::EntityTrait;
    // Already-bootstrapped / raced insert → ignore conflict.
    let _ = pbx_db::entities::domains::Entity::insert(am).exec(db).await;
    d
}

pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

pub fn new_uuid() -> String {
    uuid::Uuid::now_v7().to_string()
}
