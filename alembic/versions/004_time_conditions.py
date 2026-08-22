"""time_conditions with RLS and indexes

Revision ID: 004_time_conditions
Revises: 003_ring_groups
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

# revision identifiers, used by Alembic.
revision: str = "004_time_conditions"
down_revision: Union[str, None] = "003_ring_groups"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "time_conditions",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("name", sa.String(length=128), nullable=False),
        sa.Column("extension", sa.String(length=32), nullable=False),
        sa.Column("context", sa.String(length=128), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=True),
        sa.Column("time_hour", sa.String(length=64), nullable=True),
        sa.Column("time_minute", sa.String(length=64), nullable=True),
        sa.Column("time_wday", sa.String(length=64), nullable=True),
        sa.Column("time_mday", sa.String(length=64), nullable=True),
        sa.Column("time_mon", sa.String(length=64), nullable=True),
        sa.Column("time_year", sa.String(length=32), nullable=True),
        sa.Column("time_yday", sa.String(length=64), nullable=True),
        sa.Column("time_week", sa.String(length=32), nullable=True),
        sa.Column("time_mweek", sa.String(length=32), nullable=True),
        sa.Column("minute_of_day", sa.String(length=64), nullable=True),
        sa.Column("time_of_day", sa.String(length=64), nullable=True),
        sa.Column("date_time", sa.String(length=128), nullable=True),
        sa.Column("condition", sa.String(length=512), nullable=True),
        sa.Column("true_destination", sa.String(length=128), nullable=True),
        sa.Column("false_destination", sa.String(length=128), nullable=True),
        sa.Column("timezone", sa.String(length=64), nullable=True),
        sa.Column("details", sa.JSON(), nullable=True),
        sa.Column("time_condition_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["dialplan_uuid"], ["dialplans.dialplan_uuid"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("time_condition_uuid"),
    )
    op.create_index("ix_time_conditions_domain_id", "time_conditions", ["domain_id"])
    op.create_index("ix_time_conditions_domain_extension", "time_conditions", ["domain_id", "extension"], unique=True)
    op.create_index("ix_time_conditions_name", "time_conditions", ["name"])
    op.create_index("ix_time_conditions_dialplan_uuid", "time_conditions", ["dialplan_uuid"])
    op.create_index("ix_time_conditions_enabled", "time_conditions", ["enabled"])

    # RLS
    op.execute("ALTER TABLE time_conditions ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE time_conditions FORCE ROW LEVEL SECURITY;")
    op.execute(
        """
        CREATE POLICY tenant_isolation ON time_conditions
        USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
        WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
        """
    )

    # seed permissions for time_conditions if not exist
    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name)
        VALUES
            (gen_random_uuid(), 'time_condition_view'),
            (gen_random_uuid(), 'time_condition_add'),
            (gen_random_uuid(), 'time_condition_edit'),
            (gen_random_uuid(), 'time_condition_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )


def downgrade() -> None:
    op.execute("DROP TABLE IF EXISTS time_conditions CASCADE;")
