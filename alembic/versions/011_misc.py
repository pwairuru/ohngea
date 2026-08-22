"""follow_me, call_block, destinations with RLS

Revision ID: 011_follow_me_call_block_destinations
Revises: 010_music_on_hold
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "011_misc"
down_revision: Union[str, None] = "010_music_on_hold"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # follow_me
    op.create_table(
        "follow_me",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("follow_me_enabled", sa.Boolean(), nullable=False),
        sa.Column("cid_name_prefix", sa.String(length=128), nullable=True),
        sa.Column("cid_number_prefix", sa.String(length=32), nullable=True),
        sa.Column("call_prompt", sa.String(length=32), nullable=True),
        sa.Column("dial_string", sa.String(length=512), nullable=True),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("follow_me_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("follow_me_uuid"),
    )
    op.create_index("ix_follow_me_domain_id", "follow_me", ["domain_id"])
    op.create_table(
        "follow_me_destinations",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("follow_me_uuid", sa.Uuid(), nullable=False),
        sa.Column("follow_me_destination", sa.String(length=128), nullable=False),
        sa.Column("follow_me_delay", sa.Integer(), nullable=False),
        sa.Column("follow_me_timeout", sa.Integer(), nullable=False),
        sa.Column("follow_me_prompt", sa.String(length=32), nullable=True),
        sa.Column("follow_me_order", sa.Integer(), nullable=False),
        sa.Column("follow_me_destination_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["follow_me_uuid"], ["follow_me.follow_me_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("follow_me_destination_uuid"),
    )
    op.create_index("ix_follow_me_destinations_domain_id", "follow_me_destinations", ["domain_id"])
    op.create_index("ix_follow_me_destinations_follow_me_uuid", "follow_me_destinations", ["follow_me_uuid"])

    # call_block
    op.create_table(
        "call_block",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("call_block_name", sa.String(length=128), nullable=False),
        sa.Column("call_block_number", sa.String(length=32), nullable=False),
        sa.Column("call_block_enabled", sa.Boolean(), nullable=False),
        sa.Column("call_block_description", sa.Text(), nullable=True),
        sa.Column("call_block_action", sa.String(length=32), nullable=True),
        sa.Column("call_block_count", sa.Integer(), nullable=True),
        sa.Column("call_block_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("call_block_uuid"),
    )
    op.create_index("ix_call_block_domain_id", "call_block", ["domain_id"])
    op.create_index("ix_call_block_domain_number", "call_block", ["domain_id", "call_block_number"], unique=True)

    # destinations
    op.create_table(
        "destinations",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("destination_type", sa.String(length=32), nullable=False),
        sa.Column("destination_number", sa.String(length=32), nullable=False),
        sa.Column("destination_prefix", sa.String(length=32), nullable=True),
        sa.Column("destination_enabled", sa.Boolean(), nullable=False),
        sa.Column("destination_description", sa.Text(), nullable=True),
        sa.Column("destination_context", sa.String(length=128), nullable=True),
        sa.Column("destination_app", sa.String(length=64), nullable=True),
        sa.Column("destination_data", sa.String(length=512), nullable=True),
        sa.Column("destination_caller_id_name", sa.String(length=128), nullable=True),
        sa.Column("destination_caller_id_number", sa.String(length=32), nullable=True),
        sa.Column("destination_hold_music", sa.String(length=128), nullable=True),
        sa.Column("destination_record", sa.Boolean(), nullable=True),
        sa.Column("destination_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("destination_uuid"),
    )
    op.create_index("ix_destinations_domain_id", "destinations", ["domain_id"])
    op.create_index("ix_destinations_domain_number", "destinations", ["domain_id", "destination_number"], unique=True)
    op.create_index("ix_destinations_type", "destinations", ["destination_type"])

    for tbl in ["follow_me", "follow_me_destinations", "call_block", "destinations"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(
            f"""
            CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
            """
        )

    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name) VALUES
        (gen_random_uuid(), 'follow_me_view'), (gen_random_uuid(), 'follow_me_add'), (gen_random_uuid(), 'follow_me_edit'), (gen_random_uuid(), 'follow_me_delete'),
        (gen_random_uuid(), 'call_block_view'), (gen_random_uuid(), 'call_block_add'), (gen_random_uuid(), 'call_block_edit'), (gen_random_uuid(), 'call_block_delete'),
        (gen_random_uuid(), 'destination_view'), (gen_random_uuid(), 'destination_add'), (gen_random_uuid(), 'destination_edit'), (gen_random_uuid(), 'destination_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )
    op.execute(
        """
        INSERT INTO group_permissions (id, group_uuid, permission_uuid)
        SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
        FROM groups g, permissions p
        WHERE g.name='superadmin' AND p.name IN ('follow_me_view','follow_me_add','follow_me_edit','follow_me_delete','call_block_view','call_block_add','call_block_edit','call_block_delete','destination_view','destination_add','destination_edit','destination_delete')
        AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """
    )

    op.execute("CREATE VIEW v_follow_me AS SELECT domain_id AS domain_uuid, follow_me_uuid, follow_me_enabled, cid_name_prefix, cid_number_prefix, call_prompt, dial_string, description, created_at AS insert_date, updated_at AS update_date FROM follow_me;")
    op.execute("CREATE VIEW v_follow_me_destinations AS SELECT domain_id AS domain_uuid, follow_me_destination_uuid, follow_me_uuid, follow_me_destination, follow_me_delay, follow_me_timeout, created_at AS insert_date FROM follow_me_destinations;")
    op.execute("CREATE VIEW v_call_block AS SELECT domain_id AS domain_uuid, call_block_uuid, call_block_name, call_block_number, call_block_enabled, call_block_count, call_block_description, created_at AS insert_date FROM call_block;")
    op.execute("CREATE VIEW v_destinations AS SELECT domain_id AS domain_uuid, destination_uuid, destination_type, destination_number, destination_prefix, destination_enabled, destination_description, destination_context, destination_app, destination_data, created_at AS insert_date FROM destinations;")


def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_destinations;")
    op.execute("DROP VIEW IF EXISTS v_call_block;")
    op.execute("DROP VIEW IF EXISTS v_follow_me_destinations;")
    op.execute("DROP VIEW IF EXISTS v_follow_me;")
    op.execute("DROP TABLE IF EXISTS destinations CASCADE;")
    op.execute("DROP TABLE IF EXISTS call_block CASCADE;")
    op.execute("DROP TABLE IF EXISTS follow_me_destinations CASCADE;")
    op.execute("DROP TABLE IF EXISTS follow_me CASCADE;")
