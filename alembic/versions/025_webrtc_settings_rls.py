"""add RLS to webrtc_settings

Revision ID: 025_webrtc_settings_rls
Revises: 024_sip_profiles_wss
Create Date: 2026-08-23
"""
from typing import Sequence, Union

from alembic import op

revision: str = "025_webrtc_settings_rls"
down_revision: Union[str, None] = "024_sip_profiles_wss"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Pattern per alembic/versions/001_initial.py:264 — enable FORCE RLS + policy using domain_id = current_setting
    op.execute("ALTER TABLE webrtc_settings ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE webrtc_settings FORCE ROW LEVEL SECURITY;")
    op.execute(
        """
        CREATE POLICY webrtc_settings_tenant ON webrtc_settings
        USING (domain_id = current_setting('app.domain_uuid')::uuid)
        WITH CHECK (domain_id = current_setting('app.domain_uuid')::uuid);
        """
    )


def downgrade() -> None:
    op.execute("DROP POLICY IF EXISTS webrtc_settings_tenant ON webrtc_settings;")
    op.execute("ALTER TABLE webrtc_settings DISABLE ROW LEVEL SECURITY;")
