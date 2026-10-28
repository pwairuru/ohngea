//! Batch 1 settings data layer.
//!
//! Entities mirror Python ground truth column-for-column:
//! - `vars` — app/vars/models.py (+ var_command/var_hostname from alembic 029)
//! - `sofia_global_settings` — app/sofia_global_settings/models.py (global, no domain_id)
//! - `default_settings` — app/admin_settings/models.py (035_provision_defaults)
//! - `domain_settings` — app/domains/models.py
//!
//! Runtime uses Postgres (`DATABASE_URL`); tests use shared in-memory sqlite.

pub mod entities;
pub mod pagination;
pub mod seeds;
pub mod tenant;

pub use pagination::{paginate, Page};
pub use tenant::{connect_sqlite_memory, set_tenant, set_tenant_sql};

use sea_orm::{ConnectionTrait, DbErr};

const DDL_VARS: &str = "CREATE TABLE IF NOT EXISTS vars (
  var_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  var_category VARCHAR(64) NOT NULL,
  var_name VARCHAR(128) NOT NULL,
  var_value VARCHAR(512) NOT NULL,
  var_enabled BOOLEAN NOT NULL,
  var_description TEXT,
  var_order INTEGER,
  var_command VARCHAR(512),
  var_hostname VARCHAR(255),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (domain_id, var_name, var_category)
)";
const DDL_VARS_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_vars_domain_id ON vars (domain_id)";

const DDL_SOFIA: &str = "CREATE TABLE IF NOT EXISTS sofia_global_settings (
  sofia_global_setting_uuid TEXT PRIMARY KEY,
  global_setting_name VARCHAR(128) NOT NULL UNIQUE,
  global_setting_value VARCHAR(512) NOT NULL,
  global_setting_enabled BOOLEAN NOT NULL,
  global_setting_description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_DEFAULT_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS default_settings (
  default_setting_uuid TEXT PRIMARY KEY,
  default_setting_category VARCHAR(64) NOT NULL,
  default_setting_subcategory VARCHAR(64) NOT NULL,
  default_setting_name VARCHAR(64) NOT NULL,
  default_setting_value TEXT,
  default_setting_enabled BOOLEAN NOT NULL,
  default_setting_description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_DEFAULT_SETTINGS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_default_settings_category_sub \
     ON default_settings (default_setting_category, default_setting_subcategory)";

const DDL_DOMAIN_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS domain_settings (
  id TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  category VARCHAR(64) NOT NULL,
  subcategory VARCHAR(64) NOT NULL,
  key VARCHAR(64) NOT NULL,
  value VARCHAR(1024) NOT NULL,
  enabled BOOLEAN NOT NULL,
  created_at TEXT NOT NULL
)";
const DDL_DOMAIN_SETTINGS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_domain_settings_domain ON domain_settings (domain_id)";

/// Create the four settings tables (sqlite bootstrap for tests / local dev).
/// Production schema comes from alembic migrations.
pub async fn create_schema(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    for ddl in [
        DDL_VARS,
        DDL_VARS_IDX,
        DDL_SOFIA,
        DDL_DEFAULT_SETTINGS,
        DDL_DEFAULT_SETTINGS_IDX,
        DDL_DOMAIN_SETTINGS,
        DDL_DOMAIN_SETTINGS_IDX,
    ] {
        db.execute_unprepared(ddl).await?;
    }
    Ok(())
}
