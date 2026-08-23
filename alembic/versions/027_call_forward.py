"""call_forward standalone table

Revision ID: 027_call_forward
Revises: 026_groups_users_rls
Create Date: 2026-08-23
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
import sqlmodel

revision: str = "027_call_forward"
down_revision: Union[str, None] = "026_groups_users_rls"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    try:
        op.create_table(
            "call_forwards",
            sa.Column("call_forward_uuid", sa.Uuid(), primary_key=True),
            sa.Column("domain_id", sa.Uuid(), sa.ForeignKey("domains.domain_id", ondelete="CASCADE"), nullable=False, index=True),
            sa.Column("extension_uuid", sa.Uuid(), sa.ForeignKey("extensions.extension_uuid", ondelete="CASCADE"), nullable=True, index=True),
            sa.Column("forward_all_destination", sa.String(length=128), nullable=True),
            sa.Column("forward_all_enabled", sa.Boolean(), server_default="false", nullable=False),
            sa.Column("forward_busy_destination", sa.String(length=128), nullable=True),
            sa.Column("forward_busy_enabled", sa.Boolean(), server_default="false", nullable=False),
            sa.Column("forward_no_answer_destination", sa.String(length=128), nullable=True),
            sa.Column("forward_no_answer_enabled", sa.Boolean(), server_default="false", nullable=False),
            sa.Column("forward_user_not_registered_destination", sa.String(length=128), nullable=True),
            sa.Column("forward_user_not_registered_enabled", sa.Boolean(), server_default="false", nullable=False),
            sa.Column("enabled", sa.Boolean(), server_default="true", nullable=False),
            sa.Column("description", sa.String(length=512), nullable=True),
            sa.Column("created_at", sa.DateTime(), nullable=False),
            sa.Column("updated_at", sa.DateTime(), nullable=False),
        )
    except Exception:
        pass
    # RLS
    try:
        op.execute("ALTER TABLE call_forwards ENABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE call_forwards FORCE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_call_forwards ON call_forwards;")
    except Exception:
        pass
    try:
        op.execute("""
            CREATE POLICY tenant_call_forwards ON call_forwards
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
        """)
    except Exception:
        pass
    try:
        op.create_index("ix_call_forwards_domain", "call_forwards", ["domain_id"])
    except Exception:
        pass

def downgrade() -> None:
    try:
        op.execute("DROP POLICY IF EXISTS tenant_call_forwards ON call_forwards;")
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE call_forwards DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("call_forwards")
    except Exception:
        pass
