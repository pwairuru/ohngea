"""fifo call_centers vars number_translations

Revision ID: 013_fifo
Revises: 012_pins
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
revision: str = "013_fifo"
down_revision: Union[str, None] = "012_pins"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.create_table("fifo", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("fifo_name", sa.String(length=128), nullable=False), sa.Column("fifo_extension", sa.String(length=32), nullable=True), sa.Column("fifo_enabled", sa.Boolean(), nullable=False), sa.Column("fifo_description", sa.Text(), nullable=True), sa.Column("fifo_strategy", sa.String(length=32), nullable=True), sa.Column("fifo_caller_id_name", sa.String(length=128), nullable=True), sa.Column("fifo_caller_id_number", sa.String(length=32), nullable=True), sa.Column("fifo_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("fifo_uuid"))
    op.create_index("ix_fifo_domain_id", "fifo", ["domain_id"])
    op.create_index("ix_fifo_domain_name", "fifo", ["domain_id", "fifo_name"], unique=True)
    op.create_table("call_centers", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("queue_name", sa.String(length=128), nullable=False), sa.Column("queue_extension", sa.String(length=32), nullable=True), sa.Column("queue_strategy", sa.String(length=32), nullable=True), sa.Column("queue_enabled", sa.Boolean(), nullable=False), sa.Column("queue_description", sa.Text(), nullable=True), sa.Column("queue_greeting", sa.String(length=128), nullable=True), sa.Column("queue_moh_sound", sa.String(length=128), nullable=True), sa.Column("call_center_queue_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("call_center_queue_uuid"))
    op.create_index("ix_call_centers_domain_id", "call_centers", ["domain_id"])
    op.create_index("ix_call_centers_domain_name", "call_centers", ["domain_id", "queue_name"], unique=True)
    op.create_table("call_center_agents", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("call_center_queue_uuid", sa.Uuid(), nullable=False), sa.Column("agent_name", sa.String(length=128), nullable=False), sa.Column("agent_type", sa.String(length=32), nullable=True), sa.Column("agent_contact", sa.String(length=128), nullable=True), sa.Column("agent_status", sa.String(length=32), nullable=True), sa.Column("call_center_agent_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.ForeignKeyConstraint(["call_center_queue_uuid"], ["call_centers.call_center_queue_uuid"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("call_center_agent_uuid"))
    op.create_index("ix_call_center_agents_queue", "call_center_agents", ["call_center_queue_uuid"])
    op.create_table("vars", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("var_category", sa.String(length=64), nullable=False), sa.Column("var_name", sa.String(length=128), nullable=False), sa.Column("var_value", sa.String(length=512), nullable=False), sa.Column("var_enabled", sa.Boolean(), nullable=False), sa.Column("var_description", sa.Text(), nullable=True), sa.Column("var_order", sa.Integer(), nullable=True), sa.Column("var_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("var_uuid"))
    op.create_index("ix_vars_domain_id", "vars", ["domain_id"])
    op.create_index("ix_vars_domain_name", "vars", ["domain_id", "var_name"], unique=True)
    op.create_table("number_translations", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("number_translation_name", sa.String(length=128), nullable=False), sa.Column("number_translation_regex", sa.String(length=512), nullable=False), sa.Column("number_translation_replace", sa.String(length=512), nullable=False), sa.Column("number_translation_enabled", sa.Boolean(), nullable=False), sa.Column("number_translation_description", sa.Text(), nullable=True), sa.Column("number_translation_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("number_translation_uuid"))
    op.create_index("ix_number_translations_domain_id", "number_translations", ["domain_id"])
    op.create_index("ix_number_translations_domain_name", "number_translations", ["domain_id", "number_translation_name"], unique=True)
    for tbl in ["fifo", "call_centers", "call_center_agents", "vars", "number_translations"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(f"CREATE POLICY tenant_isolation ON {tbl} USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');")
    op.execute("INSERT INTO permissions (permission_uuid, name) VALUES (gen_random_uuid(), 'fifo_view'), (gen_random_uuid(), 'fifo_add'), (gen_random_uuid(), 'fifo_edit'), (gen_random_uuid(), 'fifo_delete'), (gen_random_uuid(), 'call_center_view'), (gen_random_uuid(), 'call_center_add'), (gen_random_uuid(), 'call_center_edit'), (gen_random_uuid(), 'call_center_delete'), (gen_random_uuid(), 'var_view'), (gen_random_uuid(), 'var_add'), (gen_random_uuid(), 'var_edit'), (gen_random_uuid(), 'var_delete'), (gen_random_uuid(), 'number_translation_view'), (gen_random_uuid(), 'number_translation_add'), (gen_random_uuid(), 'number_translation_edit'), (gen_random_uuid(), 'number_translation_delete') ON CONFLICT (name) DO NOTHING;")
    op.execute("INSERT INTO group_permissions (id, group_uuid, permission_uuid) SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p WHERE g.name='superadmin' AND p.name IN ('fifo_view','fifo_add','fifo_edit','fifo_delete','call_center_view','call_center_add','call_center_edit','call_center_delete','var_view','var_add','var_edit','var_delete','number_translation_view','number_translation_add','number_translation_edit','number_translation_delete') AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);")
    op.execute("CREATE VIEW v_fifo AS SELECT domain_id AS domain_uuid, fifo_uuid, fifo_name, fifo_extension, fifo_enabled, fifo_description, created_at AS insert_date FROM fifo;")
    op.execute("CREATE VIEW v_call_centers AS SELECT domain_id AS domain_uuid, call_center_queue_uuid, queue_name, queue_extension, queue_strategy, queue_enabled, queue_description, created_at AS insert_date FROM call_centers;")
    op.execute("CREATE VIEW v_vars AS SELECT domain_id AS domain_uuid, var_uuid, var_category, var_name, var_value, var_enabled, var_description, created_at AS insert_date FROM vars;")
    op.execute("CREATE VIEW v_number_translations AS SELECT domain_id AS domain_uuid, number_translation_uuid, number_translation_name, number_translation_regex, number_translation_replace, number_translation_enabled, number_translation_description, created_at AS insert_date FROM number_translations;")

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_number_translations; DROP VIEW IF EXISTS v_vars; DROP VIEW IF EXISTS v_call_centers; DROP VIEW IF EXISTS v_fifo;")
    op.execute("DROP TABLE IF EXISTS number_translations CASCADE; DROP TABLE IF EXISTS vars CASCADE; DROP TABLE IF EXISTS call_center_agents CASCADE; DROP TABLE IF EXISTS call_centers CASCADE; DROP TABLE IF EXISTS fifo CASCADE;")
