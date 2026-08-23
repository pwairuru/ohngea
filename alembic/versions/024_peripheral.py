"""follow_me voicemail conference fifo peripheral

Revision ID: 024_peripheral
Revises: 023_devices_dialplans_sip_ivr
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "024_peripheral"
down_revision: Union[str, None] = "023_devices_dialplans_sip_ivr"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    # follow_me ignore_busy
    try:
        op.add_column("follow_me", sa.Column("follow_me_ignore_busy", sa.Boolean(), nullable=False, server_default="false"))
    except Exception:
        pass
    # voicemails extra fields (legacy has many, modern has some)
    try:
        op.add_column("voicemails", sa.Column("voicemail_tutorial", sa.Boolean(), nullable=True, server_default="false"))
    except Exception:
        pass
    try:
        op.add_column("voicemails", sa.Column("voicemail_local_after_email", sa.Boolean(), nullable=True, server_default="true"))
    except Exception:
        pass
    # voicemail_enabled already exists as enabled in modern; skip
    try:
        op.add_column("voicemails", sa.Column("voicemail_transcription_enabled", sa.Boolean(), nullable=True, server_default="false"))
    except Exception:
        pass
    # ring_groups extra
    try:
        op.add_column("ring_groups", sa.Column("ring_group_forward_enabled", sa.Boolean(), nullable=True, server_default="false"))
    except Exception:
        pass
    try:
        op.add_column("ring_groups", sa.Column("ring_group_forward_destination", sa.String(length=128), nullable=True))
    except Exception:
        pass
    # conferences already has conference_enabled

def downgrade() -> None:
    for tbl, col in [("follow_me","follow_me_ignore_busy"),("voicemails","voicemail_tutorial"),("voicemails","voicemail_local_after_email"),("voicemails","voicemail_transcription_enabled"),("ring_groups","ring_group_forward_enabled"),("ring_groups","ring_group_forward_destination")]:
        try:
            op.drop_column(tbl, col)
        except Exception:
            pass
