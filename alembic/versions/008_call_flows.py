"""call_flows with RLS

Revision ID: 008_call_flows
Revises: 007_fax
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "008_call_flows"
down_revision: Union[str, None] = "007_fax"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "call_flows",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("call_flow_name", sa.String(length=128), nullable=False),
        sa.Column("call_flow_extension", sa.String(length=32), nullable=False),
        sa.Column("call_flow_feature_code", sa.String(length=32), nullable=True),
        sa.Column("call_flow_context", sa.String(length=128), nullable=False),
        sa.Column("call_flow_enabled", sa.Boolean(), nullable=False),
        sa.Column("call_flow_description", sa.Text(), nullable=True),
        sa.Column("call_flow_alternate_label", sa.String(length=128), nullable=True),
        sa.Column("call_flow_alternate_destination", sa.String(length=128), nullable=True),
        sa.Column("call_flow_sound", sa.String(length=128), nullable=True),
        sa.Column("call_flow_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("call_flow_uuid"),
    )
    op.create_index("ix_call_flows_domain_id", "call_flows", ["domain_id"])
    op.create_index("ix_call_flows_domain_extension", "call_flows", ["domain_id", "call_flow_extension"], unique=True)
    op.create_index("ix_call_flows_name", "call_flows", ["call_flow_name"])

    op.execute("ALTER TABLE call_flows ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE call_flows FORCE ROW LEVEL SECURITY;")
    op.execute(
        """
        CREATE POLICY tenant_isolation ON call_flows
        USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
        WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
        """
    )
    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name) VALUES
        (gen_random_uuid(), 'call_flow_view'),
        (gen_random_uuid(), 'call_flow_add'),
        (gen_random_uuid(), 'call_flow_edit'),
        (gen_random_uuid(), 'call_flow_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )
    op.execute(
        """
        INSERT INTO group_permissions (id, group_uuid, permission_uuid)
        SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
        FROM groups g, permissions p
        WHERE g.name='superadmin' AND p.name IN ('call_flow_view','call_flow_add','call_flow_edit','call_flow_delete')
        ON CONFLICT DO NOTHING;
        """
    )


def downgrade() -> None:
    op.execute("DROP TABLE IF EXISTS call_flows CASCADE;")
