"""add webrtc_settings per-tenant table

Revision ID: 020_webrtc_settings
Revises: 022_gateways_full
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "020_webrtc_settings"
down_revision: Union[str, None] = "022_gateways_full"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "webrtc_settings",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("auth_mode", sa.String(length=32), nullable=False),
        sa.Column("wss_url", sa.String(length=256), nullable=True),
        sa.Column("sip_domain", sa.String(length=128), nullable=True),
        sa.Column("ice_servers", sa.JSON(), nullable=True),
        sa.Column("created_at", sa.DateTime(), nullable=False),
        sa.Column("updated_at", sa.DateTime(), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("domain_id"),
    )


def downgrade() -> None:
    op.drop_table("webrtc_settings")
