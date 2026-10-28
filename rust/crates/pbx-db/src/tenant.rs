//! Tenant isolation: `SET LOCAL app.domain_id` per transaction (Postgres RLS).
//! Python equivalent: `app/db.py::set_tenant` (GUC `app.domain_uuid`).

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

/// Build the `SET LOCAL` statement. Pure — unit-testable without a DB.
#[must_use]
pub fn set_tenant_sql(domain_id: &str) -> String {
    format!(
        "SET LOCAL app.domain_id = '{}'",
        domain_id.replace('\'', "''")
    )
}

/// Run `SET LOCAL app.domain_id` on the current transaction.
pub async fn set_tenant(conn: &DatabaseConnection, domain_id: &str) -> Result<(), DbErr> {
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
