"""ring_groups + ring_group_members with RLS and indexes

Revision ID: 003_ring_groups
Revises: 002_recordings
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

# revision identifiers, used by Alembic.
revision: str = "003_ring_groups"
down_revision: Union[str, None] = "002_recordings"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ring_groups table
    op.create_table(
        "ring_groups",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("name", sa.String(length=128), nullable=False),
        sa.Column("extension", sa.String(length=32), nullable=False),
        sa.Column("strategy", sa.String(length=32), nullable=False),
        sa.Column("timeout", sa.Integer(), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("ring_group_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("ring_group_uuid"),
    )
    op.create_index("ix_ring_groups_domain_id", "ring_groups", ["domain_id"])
    op.create_index("ix_ring_groups_domain_extension", "ring_groups", ["domain_id", "extension"], unique=True)
    op.create_index("ix_ring_groups_name", "ring_groups", ["name"])
    op.create_index("ix_ring_groups_strategy", "ring_groups", ["strategy"])

    # ring_group_members table
    op.create_table(
        "ring_group_members",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("ring_group_uuid", sa.Uuid(), nullable=False),
        sa.Column("destination_number", sa.String(length=32), nullable=False),
        sa.Column("destination_delay", sa.Integer(), nullable=False),
        sa.Column("destination_timeout", sa.Integer(), nullable=False),
        sa.Column("destination_enabled", sa.Boolean(), nullable=False),
        sa.Column("destination_description", sa.String(length=255), nullable=True),
        sa.Column("ring_group_member_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["ring_group_uuid"], ["ring_groups.ring_group_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("ring_group_member_uuid"),
    )
    op.create_index("ix_ring_group_members_domain_id", "ring_group_members", ["domain_id"])
    op.create_index("ix_ring_group_members_ring_group_uuid", "ring_group_members", ["ring_group_uuid"])

    # compatibility view/table for legacy Ohngea: ring_group_destinations
    # Create a VIEW that mirrors ring_group_members with destination-style column names
    # This allows legacy queries SELECT * FROM ring_group_destinations to work.
    # Also create ring_group_destinations as alias table for ORM compatibility if view not enough,
    # but VIEW is sufficient. For SQLModel's RingGroupDestination table=True we create actual table
    # as duplicate to avoid view write issues; instead we create VIEW and also ensure ORM can still query
    # via view. Simplest: create VIEW.

    op.execute("""
        CREATE VIEW ring_group_destinations AS
        SELECT
            ring_group_member_uuid AS ring_group_destination_uuid,
            domain_id,
            ring_group_uuid,
            destination_number,
            destination_delay,
            destination_timeout,
            destination_enabled,
            destination_description,
            created_at
        FROM ring_group_members;
    """)

    # RLS
    for tbl in ["ring_groups", "ring_group_members"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(
            f"""
            CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
            """
        )


def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS ring_group_destinations;")
    op.execute("DROP TABLE IF EXISTS ring_group_members CASCADE;")
    op.execute("DROP TABLE IF EXISTS ring_groups CASCADE;")
