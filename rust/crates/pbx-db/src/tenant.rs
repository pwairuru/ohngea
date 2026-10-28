//! Tenant isolation: `SET LOCAL app.domain_uuid` per transaction (Postgres RLS).
//! Python equivalent: `app/db.py::set_tenant` (GUC `app.domain_uuid`).

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

/// Build the `SET LOCAL` statement. Pure — unit-testable without a DB.
#[must_use]
pub fn set_tenant_sql(domain_id: &str) -> String {
    format!(
        "SET LOCAL app.domain_uuid = '{}'",
        domain_id.replace('\'', "''")
    )
}

/// Run `SET LOCAL app.domain_uuid` on the current transaction.
/// Postgres RLS (`app/db.py::set_tenant` parity). No-op on sqlite/memory
/// tests (no `SET LOCAL` there) — never silent on postgres.
/// Generic over any connection (DatabaseConnection or Transaction) so
/// callers can scope RLS inside an explicit txn (required for FORCE RLS).
pub async fn set_tenant<C>(conn: &C, domain_id: &str) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    use sea_orm::DatabaseBackend;
    if matches!(
        conn.get_database_backend(),
        DatabaseBackend::Sqlite | DatabaseBackend::MySql
    ) {
        return Ok(());
    }
    conn.execute_unprepared(&set_tenant_sql(domain_id))
        .await
        .map(|_| ())
}

/// Shared in-memory sqlite for tests (single connection so `:memory:` is shared).
pub async fn connect_sqlite_memory() -> Result<DatabaseConnection, DbErr> {
    let mut opt = sea_orm::ConnectOptions::new("sqlite::memory:".to_owned());
    opt.max_connections(1);
    sea_orm::Database::connect(opt).await
}
