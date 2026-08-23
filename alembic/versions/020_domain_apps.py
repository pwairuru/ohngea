"""add domain_apps for per-tenant toggles

Revision ID: 020_domain_apps
Revises: 019_contacts_recordings
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "020_domain_apps"
down_revision: Union[str, None] = "018_logs_perm"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table("domain_apps",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("app_name", sa.String(length=64), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("domain_id", "app_name", name="uq_domain_apps_domain_app")
    )
    op.create_index("ix_domain_apps_domain_id", "domain_apps", ["domain_id"])
    op.create_index("ix_domain_apps_app_name", "domain_apps", ["app_name"])
    op.execute("ALTER TABLE domain_apps ENABLE ROW LEVEL SECURITY")
    op.execute("ALTER TABLE domain_apps FORCE ROW LEVEL SECURITY")
    op.execute("CREATE POLICY tenant_isolation ON domain_apps USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')")


def downgrade() -> None:
    op.execute("DROP POLICY IF EXISTS tenant_isolation ON domain_apps")
    op.drop_table("domain_apps")
