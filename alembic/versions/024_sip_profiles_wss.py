"""add sip_profiles wss fields

Revision ID: 024_sip_profiles_wss
Revises: 023_groups_users_parity
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "024_sip_profiles_wss"
down_revision: Union[str, None] = "023_groups_users_parity"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column("sip_profiles", sa.Column("wss_port", sa.Integer(), nullable=True))
    op.add_column("sip_profiles", sa.Column("wss_binding", sa.String(length=128), nullable=True))
    op.add_column("sip_profiles", sa.Column("tls_cert_path", sa.String(length=256), nullable=True))
    op.add_column("sip_profiles", sa.Column("tls_key_path", sa.String(length=256), nullable=True))


def downgrade() -> None:
    for c in ["tls_key_path", "tls_cert_path", "wss_binding", "wss_port"]:
        try:
            op.drop_column("sip_profiles", c)
        except Exception:
            pass
