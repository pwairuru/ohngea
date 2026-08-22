"""fax with RLS

Revision ID: 007_fax
Revises: 006_conferences
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "007_fax"
down_revision: Union[str, None] = "006_conferences"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "fax",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("fax_extension", sa.String(length=32), nullable=False),
        sa.Column("fax_name", sa.String(length=128), nullable=False),
        sa.Column("fax_email", sa.String(length=255), nullable=True),
        sa.Column("fax_pin_number", sa.String(length=32), nullable=True),
        sa.Column("fax_caller_id_name", sa.String(length=128), nullable=True),
        sa.Column("fax_caller_id_number", sa.String(length=32), nullable=True),
        sa.Column("fax_forward_number", sa.String(length=32), nullable=True),
        sa.Column("fax_toll_allow", sa.String(length=32), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("fax_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("fax_uuid"),
    )
    op.create_index("ix_fax_domain_id", "fax", ["domain_id"])
    op.create_index("ix_fax_domain_extension", "fax", ["domain_id", "fax_extension"], unique=True)
    op.create_index("ix_fax_name", "fax", ["fax_name"])

    op.execute("ALTER TABLE fax ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE fax FORCE ROW LEVEL SECURITY;")
    op.execute(
        """
        CREATE POLICY tenant_isolation ON fax
        USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
        WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
        """
    )
    # seed perms
    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name) VALUES
        (gen_random_uuid(), 'fax_view'),
        (gen_random_uuid(), 'fax_add'),
        (gen_random_uuid(), 'fax_edit'),
        (gen_random_uuid(), 'fax_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )
    # assign to superadmin
    op.execute(
        """
        INSERT INTO group_permissions (id, group_uuid, permission_uuid)
        SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
        FROM groups g, permissions p
        WHERE g.name='superadmin' AND p.name IN ('fax_view','fax_add','fax_edit','fax_delete')
        ON CONFLICT DO NOTHING;
        """
    )


def downgrade() -> None:
    op.execute("DROP TABLE IF EXISTS fax CASCADE;")
