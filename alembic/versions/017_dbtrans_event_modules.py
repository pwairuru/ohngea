"""database_transactions event_guard modules - last HAS_DB

Revision ID: 017_dbtrans
Revises: 016_extset
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
revision: str = "017_dbtrans"
down_revision: Union[str, None] = "016_extset"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.create_table("database_transactions", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("user_uuid", sa.Uuid(), nullable=True), sa.Column("app_name", sa.String(length=128), nullable=True), sa.Column("transaction_code", sa.String(length=512), nullable=True), sa.Column("transaction_address", sa.String(length=64), nullable=True), sa.Column("transaction_type", sa.String(length=16), nullable=True), sa.Column("transaction_old", sa.Text(), nullable=True), sa.Column("transaction_new", sa.Text(), nullable=True), sa.Column("database_transaction_uuid", sa.Uuid(), nullable=False), sa.Column("transaction_date", sa.DateTime(timezone=True), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("database_transaction_uuid"))
    op.create_index("ix_database_transactions_domain_id", "database_transactions", ["domain_id"])
    op.create_table("event_guard_logs", sa.Column("hostname", sa.String(length=128), nullable=True), sa.Column("log_date", sa.DateTime(timezone=True), nullable=False), sa.Column("filter", sa.String(length=128), nullable=True), sa.Column("ip_address", sa.String(length=64), nullable=True), sa.Column("extension", sa.String(length=32), nullable=True), sa.Column("user_agent", sa.String(length=512), nullable=True), sa.Column("log_status", sa.String(length=32), nullable=True), sa.Column("event_guard_log_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.PrimaryKeyConstraint("event_guard_log_uuid"))
    op.create_table("modules", sa.Column("module_label", sa.String(length=128), nullable=False), sa.Column("module_name", sa.String(length=128), nullable=False), sa.Column("module_category", sa.String(length=64), nullable=True), sa.Column("module_enabled", sa.Boolean(), nullable=False), sa.Column("module_default_enabled", sa.Boolean(), nullable=False), sa.Column("module_description", sa.Text(), nullable=True), sa.Column("module_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.PrimaryKeyConstraint("module_uuid"), sa.UniqueConstraint("module_name"))
    op.execute("ALTER TABLE database_transactions ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE database_transactions FORCE ROW LEVEL SECURITY;")
    op.execute("CREATE POLICY tenant_isolation ON database_transactions USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');")
    op.execute("INSERT INTO permissions (permission_uuid, name) VALUES (gen_random_uuid(), 'database_transaction_view'), (gen_random_uuid(), 'database_transaction_add'), (gen_random_uuid(), 'event_guard_view'), (gen_random_uuid(), 'event_guard_add'), (gen_random_uuid(), 'module_view'), (gen_random_uuid(), 'module_add') ON CONFLICT (name) DO NOTHING;")
    op.execute("INSERT INTO group_permissions (id, group_uuid, permission_uuid) SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p WHERE g.name='superadmin' AND p.name IN ('database_transaction_view','database_transaction_add','event_guard_view','event_guard_add','module_view','module_add') AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);")
    op.execute("CREATE VIEW v_database_transactions AS SELECT domain_id AS domain_uuid, database_transaction_uuid, app_name, transaction_code, transaction_type, transaction_date AS insert_date FROM database_transactions;")
    op.execute("CREATE VIEW v_event_guard AS SELECT event_guard_log_uuid, hostname, ip_address, extension, log_status, created_at AS insert_date FROM event_guard_logs;")
    op.execute("CREATE VIEW v_modules AS SELECT module_uuid, module_label, module_name, module_category, module_enabled, created_at AS insert_date FROM modules;")

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_modules; DROP VIEW IF EXISTS v_event_guard; DROP VIEW IF EXISTS v_database_transactions;")
    op.execute("DROP TABLE IF EXISTS modules CASCADE; DROP TABLE IF EXISTS event_guard_logs CASCADE; DROP TABLE IF EXISTS database_transactions CASCADE;")
