//! Shared SeaORM connection for call-routing routers.
//!
//! Postgres via `DATABASE_URL` at runtime (pool parity with `app/db.py`:
//! max 5, timeout 30, pre-ping via `test_before_acquire`), sqlite file for
//! unit tests only. One global connection per process — `test_app()` builds
//! a fresh router per request, so per-router state would lose rows between
//! POST and GET in tests.
//!
//! No sqlite fallback when `DATABASE_URL` is postgres: postgres URL always
//! uses `pbx_db::connect_postgres` + `Migrator::up`, never `create_schema`.

use sea_orm::DatabaseConnection;
use tokio::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

/// Normalize Python `postgresql+asyncpg://` / `postgresql://` to SeaORM
/// `postgres://`. Pure — unit-testable.
#[must_use]
pub fn normalize_database_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("postgresql+asyncpg://") {
        return format!("postgres://{rest}");
    }
    if let Some(rest) = url.strip_prefix("postgresql://") {
        return format!("postgres://{rest}");
    }
    url.to_owned()
}

fn is_postgres_url(url: &str) -> bool {
    let u = normalize_database_url(url);
    u.starts_with("postgres://") || u.starts_with("postgresql://")
}

async fn init_db() -> DatabaseConnection {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let url = normalize_database_url(url.trim());
        if is_postgres_url(&url) {
            let db = pbx_db::connect_postgres(&url)
                .await
                .expect("DATABASE_URL postgres connect");
            use sea_orm_migration::MigratorTrait;
            pbx_db::migration::Migrator::up(&db, None)
                .await
                .expect("postgres migrate on boot");
            return db;
        }
        if url.starts_with("sqlite") {
            let mut opt = sea_orm::ConnectOptions::new(url);
            opt.max_connections(1);
            let db = sea_orm::Database::connect(opt)
                .await
                .expect("sqlite connect");
            pbx_db::create_schema(&db).await.expect("schema");
            return db;
        }
        // Unknown scheme with DATABASE_URL set: fail fast, never silently
        // fall back to sqlite when caller asked for a real DB.
        panic!("unsupported DATABASE_URL scheme (expected postgres:// or sqlite:)");
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
///
/// NOTE (Task 2): pure derivation — this NEVER inserts. Read paths must not
/// conjure tenant rows (orphan domains, cross-test leakage). Use
/// [`tenant_domain_required`] when the caller needs the row to exist.
pub async fn tenant_domain(
    _db: &sea_orm::DatabaseConnection,
    claims: &pbx_core::auth::Claims,
) -> uuid::Uuid {
    domain(claims)
}

/// Tenant domain that must already exist. Returns `RecordNotFound` when the
/// claims' tenant has no `domains` row — callers answer 404/empty instead of
/// auto-inserting (Task 2 read-path rule).
pub async fn tenant_domain_required(
    db: &sea_orm::DatabaseConnection,
    claims: &pbx_core::auth::Claims,
) -> Result<uuid::Uuid, sea_orm::DbErr> {
    use sea_orm::EntityTrait;
    let d = domain(claims);
    pbx_db::entities::domains::Entity::find_by_id(d)
        .one(db)
        .await?
        .map(|r| r.domain_id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("domain not found".to_owned()))
}

/// Display name for a tenant UUID when its row must be materialized:
/// real UUID claims get `tenant-{uuid}`; opaque test strings ("default")
/// are kept verbatim for readability.
pub fn tenant_display_name(claims: &pbx_core::auth::Claims) -> String {
    let d = domain(claims);
    if claims.domain_id.parse::<uuid::Uuid>().is_ok() {
        format!("tenant-{d}")
    } else {
        claims.domain_id.clone()
    }
}

/// Insert the claims' `domains` row when absent (insert-or-ignore), returning
/// the tenant UUID. WRITE-PATH ONLY: FK targets reference `domains`, so a
/// create/update/delete handler needs its parent row to exist. Never call
/// from read paths — unknown tenants must resolve to empty/404, not conjured
/// rows (Task 2). Idempotent; raced inserts collapse to the existing row.
pub async fn ensure_tenant_domain(
    db: &sea_orm::DatabaseConnection,
    claims: &pbx_core::auth::Claims,
) -> uuid::Uuid {
    use sea_orm::EntityTrait;
    let d = domain(claims);
    let name = tenant_display_name(claims);
    let now = now();
    let am = pbx_db::entities::domains::ActiveModel {
        domain_id: sea_orm::Set(d),
        domain_name: sea_orm::Set(name),
        enabled: sea_orm::Set(true),
        description: sea_orm::Set(None),
        created_at: sea_orm::Set(now),
        updated_at: sea_orm::Set(now),
    };
    // Pre-existing / raced insert → ignore conflict.
    let _ = pbx_db::entities::domains::Entity::insert(am).exec(db).await;
    d
}

pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

pub fn new_uuid() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// Per-request tenant scoping: `SET LOCAL app.domain_uuid` (Postgres RLS,
/// `app/db.py::set_tenant` parity). No-op on sqlite unit tests.
pub async fn set_tenant(db: &DatabaseConnection, claims: &pbx_core::auth::Claims) {
    let id = domain(claims);
    let _ = pbx_db::set_tenant(db, &id.to_string()).await;
}

/// Transaction scoped to tenant: `BEGIN` + `SET LOCAL app.domain_uuid`.
/// Required for FORCE RLS (each pooled connection needs GUC inside txn).
/// Falls back to plain txn on sqlite (set_tenant no-op there).
pub async fn tenant_txn(
    db: &DatabaseConnection,
    claims: &pbx_core::auth::Claims,
) -> Result<sea_orm::DatabaseTransaction, sea_orm::DbErr> {
    use sea_orm::TransactionTrait;
    let txn = db.begin().await?;
    let id = domain(claims);
    pbx_db::set_tenant(&txn, &id.to_string()).await?;
    Ok(txn)
}
