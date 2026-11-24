"""services PHP-shape parity + global tables RLS-open.

Revision ID: 046_services_parity
Revises: 045_seed_parity
041 created `services` with a Rust-invented shape (name/host/port); the PHP
ground truth (core/services v_services) uses service_name/service_category/
service_file/service_enabled/service_description. Add the PHP columns
(additive only — never drop 041 columns with possible prod data) and keep
global tables without domain_id RLS-open, matching the Rust Migrator open
list (permissions, domains, sofia_global_settings, modules, services,
device_vendors, device_vendor_functions, device_vendor_function_groups).
"""

revision: str = "046_services_parity"
down_revision: Union[str, None] = "045_seed_parity"
branch_labels = None
depends_on = None

from alembic import op


def _exec(sql: str):
    try:
        op.execute(sql)
    except Exception:
        pass


def upgrade() -> None:
    for col in [
        "ADD COLUMN IF NOT EXISTS service_name VARCHAR(128)",
        "ADD COLUMN IF NOT EXISTS service_category VARCHAR(64)",
        "ADD COLUMN IF NOT EXISTS service_file VARCHAR(512)",
        "ADD COLUMN IF NOT EXISTS service_enabled BOOLEAN NOT NULL DEFAULT TRUE",
        "ADD COLUMN IF NOT EXISTS service_description TEXT",
    ]:
        _exec(f"ALTER TABLE services {col};")
    for tbl in [
        "permissions",
        "domains",
        "sofia_global_settings",
        "modules",
        "services",
        "device_vendors",
        "device_vendor_functions",
        "device_vendor_function_groups",
    ]:
        # IF EXISTS: device_vendors*/modules have no alembic CREATE yet;
        # a bare ALTER would abort the whole single-transaction upgrade.
        _exec(f"ALTER TABLE IF EXISTS {tbl} DISABLE ROW LEVEL SECURITY;")


def downgrade() -> None:
    for col in [
        "service_name",
        "service_category",
        "service_file",
        "service_enabled",
        "service_description",
    ]:
        _exec(f"ALTER TABLE services DROP COLUMN IF EXISTS {col};")
