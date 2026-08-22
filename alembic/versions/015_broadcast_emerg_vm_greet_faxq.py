"""call_broadcast emergency_logs voicemail_greetings fax_queue

Revision ID: 015_broad
Revises: 014_conf_acl
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
revision: str = "015_broad"
down_revision: Union[str, None] = "014_conf_acl"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.create_table("call_broadcasts", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("broadcast_name", sa.String(length=128), nullable=False), sa.Column("broadcast_description", sa.Text(), nullable=True), sa.Column("broadcast_timeout", sa.Integer(), nullable=True), sa.Column("broadcast_concurrent_limit", sa.Integer(), nullable=True), sa.Column("broadcast_caller_id_name", sa.String(length=128), nullable=True), sa.Column("broadcast_caller_id_number", sa.String(length=32), nullable=True), sa.Column("broadcast_destination_type", sa.String(length=32), nullable=True), sa.Column("broadcast_phone_numbers", sa.Text(), nullable=True), sa.Column("broadcast_enabled", sa.Boolean(), nullable=False), sa.Column("call_broadcast_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("call_broadcast_uuid"))
    op.create_index("ix_call_broadcasts_domain_id", "call_broadcasts", ["domain_id"])
    op.create_index("ix_call_broadcasts_domain_name", "call_broadcasts", ["domain_id", "broadcast_name"], unique=True)
    op.create_table("emergency_logs", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("extension", sa.String(length=32), nullable=True), sa.Column("event", sa.Text(), nullable=True), sa.Column("emergency_log_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("emergency_log_uuid"))
    op.create_index("ix_emergency_logs_domain_id", "emergency_logs", ["domain_id"])
    op.create_table("voicemail_greetings", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("voicemail_id", sa.String(length=32), nullable=False), sa.Column("greeting_id", sa.Integer(), nullable=False), sa.Column("greeting_name", sa.String(length=128), nullable=True), sa.Column("greeting_filename", sa.String(length=512), nullable=True), sa.Column("greeting_description", sa.Text(), nullable=True), sa.Column("greeting_base64", sa.Text(), nullable=True), sa.Column("voicemail_greeting_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("voicemail_greeting_uuid"))
    op.create_index("ix_voicemail_greetings_domain_id", "voicemail_greetings", ["domain_id"])
    op.create_table("fax_queue", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("fax_uuid", sa.Uuid(), nullable=True), sa.Column("fax_queue_name", sa.String(length=128), nullable=True), sa.Column("fax_number", sa.String(length=32), nullable=True), sa.Column("fax_status", sa.String(length=32), nullable=False), sa.Column("fax_file", sa.String(length=512), nullable=True), sa.Column("fax_queue_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.ForeignKeyConstraint(["fax_uuid"], ["fax.fax_uuid"], ondelete="SET NULL"), sa.PrimaryKeyConstraint("fax_queue_uuid"))
    op.create_index("ix_fax_queue_domain_id", "fax_queue", ["domain_id"])
    for tbl in ["call_broadcasts","emergency_logs","voicemail_greetings","fax_queue"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(f"CREATE POLICY tenant_isolation ON {tbl} USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');")
    op.execute("INSERT INTO permissions (permission_uuid, name) VALUES (gen_random_uuid(), 'call_broadcast_view'), (gen_random_uuid(), 'call_broadcast_add'), (gen_random_uuid(), 'call_broadcast_edit'), (gen_random_uuid(), 'call_broadcast_delete'), (gen_random_uuid(), 'emergency_logs_view'), (gen_random_uuid(), 'voicemail_greeting_view'), (gen_random_uuid(), 'voicemail_greeting_add'), (gen_random_uuid(), 'fax_queue_view'), (gen_random_uuid(), 'fax_queue_add') ON CONFLICT (name) DO NOTHING;")
    op.execute("INSERT INTO group_permissions (id, group_uuid, permission_uuid) SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p WHERE g.name='superadmin' AND p.name IN ('call_broadcast_view','call_broadcast_add','call_broadcast_edit','call_broadcast_delete','emergency_logs_view','voicemail_greeting_view','voicemail_greeting_add','fax_queue_view','fax_queue_add') AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);")
    op.execute("CREATE VIEW v_call_broadcasts AS SELECT domain_id AS domain_uuid, call_broadcast_uuid, broadcast_name, broadcast_description, broadcast_enabled, created_at AS insert_date FROM call_broadcasts;")
    op.execute("CREATE VIEW v_emergency_logs AS SELECT domain_id AS domain_uuid, emergency_log_uuid, extension, event, created_at AS insert_date FROM emergency_logs;")
    op.execute("CREATE VIEW v_voicemail_greetings AS SELECT domain_id AS domain_uuid, voicemail_greeting_uuid, voicemail_id, greeting_id, greeting_name, created_at AS insert_date FROM voicemail_greetings;")
    op.execute("CREATE VIEW v_fax_queue AS SELECT domain_id AS domain_uuid, fax_queue_uuid, fax_number, fax_status, created_at AS insert_date FROM fax_queue;")

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_fax_queue; DROP VIEW IF EXISTS v_voicemail_greetings; DROP VIEW IF EXISTS v_emergency_logs; DROP VIEW IF EXISTS v_call_broadcasts;")
    op.execute("DROP TABLE IF EXISTS fax_queue CASCADE; DROP TABLE IF EXISTS voicemail_greetings CASCADE; DROP TABLE IF EXISTS emergency_logs CASCADE; DROP TABLE IF EXISTS call_broadcasts CASCADE;")
