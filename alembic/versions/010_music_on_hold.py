"""music_on_hold with RLS

Revision ID: 010_music_on_hold
Revises: 009_fix_permissions
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "010_music_on_hold"
down_revision: Union[str, None] = "009_fix_permissions"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "music_on_hold",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("music_name", sa.String(length=128), nullable=False),
        sa.Column("music_path", sa.String(length=512), nullable=False),
        sa.Column("music_rate", sa.Integer(), nullable=True),
        sa.Column("music_shuffle", sa.String(length=16), nullable=True),
        sa.Column("music_channels", sa.Integer(), nullable=True),
        sa.Column("music_interval", sa.Integer(), nullable=True),
        sa.Column("music_timer_name", sa.String(length=64), nullable=True),
        sa.Column("music_chime_list", sa.String(length=512), nullable=True),
        sa.Column("music_chime_freq", sa.String(length=64), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("music_on_hold_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("music_on_hold_uuid"),
    )
    op.create_index("ix_music_on_hold_domain_id", "music_on_hold", ["domain_id"])
    op.create_index("ix_music_on_hold_domain_name", "music_on_hold", ["domain_id", "music_name"], unique=True)
    op.create_index("ix_music_on_hold_name", "music_on_hold", ["music_name"])

    op.execute("ALTER TABLE music_on_hold ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE music_on_hold FORCE ROW LEVEL SECURITY;")
    op.execute(
        """
        CREATE POLICY tenant_isolation ON music_on_hold
        USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
        WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
        """
    )
    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name) VALUES
        (gen_random_uuid(), 'music_on_hold_view'),
        (gen_random_uuid(), 'music_on_hold_add'),
        (gen_random_uuid(), 'music_on_hold_edit'),
        (gen_random_uuid(), 'music_on_hold_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )
    op.execute(
        """
        INSERT INTO group_permissions (id, group_uuid, permission_uuid)
        SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
        FROM groups g, permissions p
        WHERE g.name='superadmin' AND p.name IN ('music_on_hold_view','music_on_hold_add','music_on_hold_edit','music_on_hold_delete')
        AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """
    )
    op.execute(
        """
        CREATE VIEW v_music_on_hold AS
        SELECT domain_id AS domain_uuid, music_on_hold_uuid, music_name, music_path, music_rate, music_shuffle, enabled AS music_on_hold_enabled, description AS music_on_hold_description, created_at AS insert_date, updated_at AS update_date FROM music_on_hold;
        """
    )


def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_music_on_hold;")
    op.execute("DROP TABLE IF EXISTS music_on_hold CASCADE;")
