"""extension_settings email_queue sofia_global_settings feature_codes

Revision ID: 016_extset
Revises: 015_broad
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
revision: str = "016_extset"
down_revision: Union[str, None] = "015_broad"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.create_table("extension_settings", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("extension_uuid", sa.Uuid(), nullable=False), sa.Column("extension_setting_type", sa.String(length=64), nullable=True), sa.Column("extension_setting_name", sa.String(length=128), nullable=False), sa.Column("extension_setting_value", sa.String(length=512), nullable=False), sa.Column("extension_setting_enabled", sa.Boolean(), nullable=False), sa.Column("extension_setting_description", sa.Text(), nullable=True), sa.Column("extension_setting_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.ForeignKeyConstraint(["extension_uuid"], ["extensions.extension_uuid"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("extension_setting_uuid"))
    op.create_index("ix_extension_settings_domain_id", "extension_settings", ["domain_id"])
    op.create_index("ix_extension_settings_ext", "extension_settings", ["extension_uuid"])
    op.create_table("email_queue", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("email_date", sa.DateTime(timezone=True), nullable=False), sa.Column("email_from", sa.String(length=512), nullable=False), sa.Column("email_to", sa.String(length=512), nullable=False), sa.Column("email_subject", sa.String(length=512), nullable=True), sa.Column("email_body", sa.Text(), nullable=True), sa.Column("email_status", sa.String(length=32), nullable=False), sa.Column("email_retry_count", sa.Integer(), nullable=False), sa.Column("email_queue_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("email_queue_uuid"))
    op.create_index("ix_email_queue_domain_id", "email_queue", ["domain_id"])
    op.create_table("sofia_global_settings", sa.Column("global_setting_name", sa.String(length=128), nullable=False), sa.Column("global_setting_value", sa.String(length=512), nullable=False), sa.Column("global_setting_enabled", sa.Boolean(), nullable=False), sa.Column("global_setting_description", sa.Text(), nullable=True), sa.Column("sofia_global_setting_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.PrimaryKeyConstraint("sofia_global_setting_uuid"), sa.UniqueConstraint("global_setting_name"))
    op.create_table("feature_codes", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("feature_code_name", sa.String(length=128), nullable=False), sa.Column("feature_code_value", sa.String(length=32), nullable=False), sa.Column("feature_code_enabled", sa.Boolean(), nullable=False), sa.Column("feature_code_description", sa.Text(), nullable=True), sa.Column("feature_code_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("feature_code_uuid"))
    op.create_index("ix_feature_codes_domain_id", "feature_codes", ["domain_id"])
    op.create_index("ix_feature_codes_domain_name", "feature_codes", ["domain_id", "feature_code_name"], unique=True)
    for tbl in ["extension_settings","email_queue","feature_codes"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(f"CREATE POLICY tenant_isolation ON {tbl} USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');")
    op.execute("INSERT INTO permissions (permission_uuid, name) VALUES (gen_random_uuid(), 'extension_setting_view'), (gen_random_uuid(), 'extension_setting_add'), (gen_random_uuid(), 'email_queue_view'), (gen_random_uuid(), 'email_queue_add'), (gen_random_uuid(), 'sofia_global_setting_view'), (gen_random_uuid(), 'sofia_global_setting_add'), (gen_random_uuid(), 'feature_code_view'), (gen_random_uuid(), 'feature_code_add') ON CONFLICT (name) DO NOTHING;")
    op.execute("INSERT INTO group_permissions (id, group_uuid, permission_uuid) SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p WHERE g.name='superadmin' AND p.name IN ('extension_setting_view','extension_setting_add','email_queue_view','email_queue_add','sofia_global_setting_view','sofia_global_setting_add','feature_code_view','feature_code_add') AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);")
    op.execute("CREATE VIEW v_extension_settings AS SELECT domain_id AS domain_uuid, extension_setting_uuid, extension_uuid, extension_setting_name, extension_setting_value, extension_setting_enabled, created_at AS insert_date FROM extension_settings;")
    op.execute("CREATE VIEW v_email_queue AS SELECT domain_id AS domain_uuid, email_queue_uuid, email_from, email_to, email_subject, email_status, created_at AS insert_date FROM email_queue;")
    op.execute("CREATE VIEW v_sofia_global_settings AS SELECT sofia_global_setting_uuid, global_setting_name, global_setting_value, global_setting_enabled, created_at AS insert_date FROM sofia_global_settings;")
    op.execute("CREATE VIEW v_feature_codes AS SELECT domain_id AS domain_uuid, feature_code_uuid, feature_code_name, feature_code_value, feature_code_enabled, created_at AS insert_date FROM feature_codes;")

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_feature_codes; DROP VIEW IF EXISTS v_sofia_global_settings; DROP VIEW IF EXISTS v_email_queue; DROP VIEW IF EXISTS v_extension_settings;")
    op.execute("DROP TABLE IF EXISTS feature_codes CASCADE; DROP TABLE IF EXISTS sofia_global_settings CASCADE; DROP TABLE IF EXISTS email_queue CASCADE; DROP TABLE IF EXISTS extension_settings CASCADE;")
