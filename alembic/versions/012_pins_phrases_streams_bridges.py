"""pins phrases streams bridges

Revision ID: 012_pins
Revises: 011_misc
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
revision: str = "012_pins"
down_revision: Union[str, None] = "011_misc"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.create_table("pin_numbers", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("pin_number", sa.String(length=32), nullable=False), sa.Column("accountcode", sa.String(length=64), nullable=True), sa.Column("enabled", sa.Boolean(), nullable=False), sa.Column("description", sa.Text(), nullable=True), sa.Column("pin_number_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("pin_number_uuid"))
    op.create_index("ix_pin_numbers_domain_id", "pin_numbers", ["domain_id"])
    op.create_index("ix_pin_numbers_domain_pin", "pin_numbers", ["domain_id", "pin_number"], unique=True)
    op.create_table("phrases", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("phrase_name", sa.String(length=128), nullable=False), sa.Column("phrase_language", sa.String(length=16), nullable=True), sa.Column("phrase_enabled", sa.Boolean(), nullable=False), sa.Column("phrase_description", sa.Text(), nullable=True), sa.Column("phrase_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("phrase_uuid"))
    op.create_index("ix_phrases_domain_id", "phrases", ["domain_id"])
    op.create_table("phrase_details", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("phrase_uuid", sa.Uuid(), nullable=False), sa.Column("phrase_detail_function", sa.String(length=64), nullable=True), sa.Column("phrase_detail_data", sa.String(length=512), nullable=True), sa.Column("phrase_detail_order", sa.Integer(), nullable=False), sa.Column("phrase_detail_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.ForeignKeyConstraint(["phrase_uuid"], ["phrases.phrase_uuid"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("phrase_detail_uuid"))
    op.create_index("ix_phrase_details_phrase_uuid", "phrase_details", ["phrase_uuid"])
    op.create_table("streams", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("stream_name", sa.String(length=128), nullable=False), sa.Column("stream_location", sa.String(length=512), nullable=True), sa.Column("stream_enabled", sa.Boolean(), nullable=False), sa.Column("stream_description", sa.Text(), nullable=True), sa.Column("stream_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("stream_uuid"))
    op.create_index("ix_streams_domain_id", "streams", ["domain_id"])
    op.create_index("ix_streams_domain_name", "streams", ["domain_id", "stream_name"], unique=True)
    op.create_table("bridges", sa.Column("domain_id", sa.Uuid(), nullable=False), sa.Column("bridge_name", sa.String(length=128), nullable=False), sa.Column("bridge_destination", sa.String(length=128), nullable=True), sa.Column("bridge_enabled", sa.Boolean(), nullable=False), sa.Column("bridge_description", sa.Text(), nullable=True), sa.Column("bridge_uuid", sa.Uuid(), nullable=False), sa.Column("created_at", sa.DateTime(timezone=True), nullable=False), sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False), sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"), sa.PrimaryKeyConstraint("bridge_uuid"))
    op.create_index("ix_bridges_domain_id", "bridges", ["domain_id"])
    op.create_index("ix_bridges_domain_name", "bridges", ["domain_id", "bridge_name"], unique=True)
    for tbl in ["pin_numbers", "phrases", "phrase_details", "streams", "bridges"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(f"CREATE POLICY tenant_isolation ON {tbl} USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');")
    op.execute("INSERT INTO permissions (permission_uuid, name) VALUES (gen_random_uuid(), 'pin_number_view'), (gen_random_uuid(), 'pin_number_add'), (gen_random_uuid(), 'pin_number_edit'), (gen_random_uuid(), 'pin_number_delete'), (gen_random_uuid(), 'phrase_view'), (gen_random_uuid(), 'phrase_add'), (gen_random_uuid(), 'phrase_edit'), (gen_random_uuid(), 'phrase_delete'), (gen_random_uuid(), 'stream_view'), (gen_random_uuid(), 'stream_add'), (gen_random_uuid(), 'stream_edit'), (gen_random_uuid(), 'stream_delete'), (gen_random_uuid(), 'bridge_view'), (gen_random_uuid(), 'bridge_add'), (gen_random_uuid(), 'bridge_edit'), (gen_random_uuid(), 'bridge_delete') ON CONFLICT (name) DO NOTHING;")
    op.execute("INSERT INTO group_permissions (id, group_uuid, permission_uuid) SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p WHERE g.name='superadmin' AND p.name IN ('pin_number_view','pin_number_add','pin_number_edit','pin_number_delete','phrase_view','phrase_add','phrase_edit','phrase_delete','stream_view','stream_add','stream_edit','stream_delete','bridge_view','bridge_add','bridge_edit','bridge_delete') AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);")
    op.execute("CREATE VIEW v_pin_numbers AS SELECT domain_id AS domain_uuid, pin_number_uuid, pin_number, accountcode, enabled, description, created_at AS insert_date FROM pin_numbers;")
    op.execute("CREATE VIEW v_phrases AS SELECT domain_id AS domain_uuid, phrase_uuid, phrase_name, phrase_language, phrase_enabled, phrase_description, created_at AS insert_date FROM phrases;")
    op.execute("CREATE VIEW v_streams AS SELECT domain_id AS domain_uuid, stream_uuid, stream_name, stream_location, stream_enabled, stream_description, created_at AS insert_date FROM streams;")
    op.execute("CREATE VIEW v_bridges AS SELECT domain_id AS domain_uuid, bridge_uuid, bridge_name, bridge_destination, bridge_enabled, bridge_description, created_at AS insert_date FROM bridges;")

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_bridges; DROP VIEW IF EXISTS v_streams; DROP VIEW IF EXISTS v_phrases; DROP VIEW IF EXISTS v_pin_numbers;")
    op.execute("DROP TABLE IF EXISTS bridges CASCADE; DROP TABLE IF EXISTS streams CASCADE; DROP TABLE IF EXISTS phrase_details CASCADE; DROP TABLE IF EXISTS phrases CASCADE; DROP TABLE IF EXISTS pin_numbers CASCADE;")
