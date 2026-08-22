"""gateways full parity

Revision ID: 022_gateways_full
Revises: 021_extension_full_parity
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "022_gateways_full"
down_revision: Union[str, None] = "021_extension_full_parity"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.add_column("gateways", sa.Column("distinct_to", sa.Boolean(), nullable=True))
    op.add_column("gateways", sa.Column("auth_username", sa.String(length=64), nullable=True))
    op.add_column("gateways", sa.Column("realm", sa.String(length=128), nullable=True))
    op.add_column("gateways", sa.Column("from_user", sa.String(length=64), nullable=True))
    op.add_column("gateways", sa.Column("from_domain", sa.String(length=128), nullable=True))
    op.add_column("gateways", sa.Column("register_proxy", sa.String(length=128), nullable=True))
    op.add_column("gateways", sa.Column("outbound_proxy", sa.String(length=128), nullable=True))
    op.add_column("gateways", sa.Column("expire_seconds", sa.Integer(), nullable=True))
    op.add_column("gateways", sa.Column("register_transport", sa.String(length=32), nullable=True))
    op.add_column("gateways", sa.Column("contact_params", sa.String(length=256), nullable=True))
    op.add_column("gateways", sa.Column("retry_seconds", sa.Integer(), nullable=True))
    op.add_column("gateways", sa.Column("extension", sa.String(length=64), nullable=True))
    op.add_column("gateways", sa.Column("ping", sa.String(length=32), nullable=True))
    op.add_column("gateways", sa.Column("ping_min", sa.Integer(), nullable=True))
    op.add_column("gateways", sa.Column("ping_max", sa.Integer(), nullable=True))
    op.add_column("gateways", sa.Column("contact_in_ping", sa.Boolean(), nullable=True))
    op.add_column("gateways", sa.Column("caller_id_in_from", sa.Boolean(), nullable=True))
    op.add_column("gateways", sa.Column("supress_cng", sa.Boolean(), nullable=True))
    op.add_column("gateways", sa.Column("sip_cid_type", sa.String(length=32), nullable=True))
    op.add_column("gateways", sa.Column("codec_prefs", sa.String(length=256), nullable=True))
    op.add_column("gateways", sa.Column("channels", sa.Integer(), nullable=True))
    op.add_column("gateways", sa.Column("extension_in_contact", sa.String(length=64), nullable=True))
    op.add_column("gateways", sa.Column("context", sa.String(length=64), nullable=True, server_default="public"))
    op.add_column("gateways", sa.Column("profile", sa.String(length=64), nullable=True, server_default="external"))
    op.add_column("gateways", sa.Column("hostname", sa.String(length=255), nullable=True))
    op.add_column("gateways", sa.Column("description", sa.String(length=512), nullable=True))
    op.add_column("gateways", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    # set updated_at for existing rows
    op.execute("UPDATE gateways SET updated_at = created_at WHERE updated_at IS NULL")
    op.alter_column("gateways", "updated_at", nullable=False)

def downgrade() -> None:
    cols = ["distinct_to","auth_username","realm","from_user","from_domain","register_proxy","outbound_proxy","expire_seconds","register_transport","contact_params","retry_seconds","extension","ping","ping_min","ping_max","contact_in_ping","caller_id_in_from","supress_cng","sip_cid_type","codec_prefs","channels","extension_in_contact","context","profile","hostname","description","updated_at"]
    for c in cols:
        try:
            op.drop_column("gateways", c)
        except Exception:
            pass
