"""final parity — ACL/vars/music/streams/phrases already complete, add indexes

Revision ID: 025_final_parity
Revises: 024_peripheral
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "025_final_parity"
down_revision: Union[str, None] = "024_peripheral"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    # vars, acl, music already have needed columns; ensure indexes for FS xml lookups
    try:
        op.create_index("ix_vars_domain_category", "vars", ["domain_id", "var_category"])
    except Exception:
        pass
    try:
        op.create_index("ix_access_controls_domain", "access_controls", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_music_on_hold_domain", "music_on_hold", ["domain_id"])
    except Exception:
        pass
    # ensure v_extensions view exists for legacy compat (already in 021, reconfirm)
    try:
        op.execute("CREATE OR REPLACE VIEW v_extensions AS SELECT domain_id AS domain_uuid, extension_uuid, extension, number_alias, password, accountcode, effective_caller_id_name, effective_caller_id_number, outbound_caller_id_name, outbound_caller_id_number, emergency_caller_id_name, emergency_caller_id_number, directory_first_name, directory_last_name, directory_visible, directory_exten_visible, max_registrations, limit_max, limit_destination, missed_call_app, missed_call_data, user_context, toll_allow, call_timeout, call_group, call_screen_enabled, user_record, hold_music, auth_acl, cidr, sip_force_contact, sip_force_expires, nibble_account, mwi_account, sip_bypass_media, absolute_codec_string, force_ping, dial_string, do_not_disturb, forward_all_destination, forward_all_enabled, forward_busy_destination, forward_busy_enabled, forward_no_answer_destination, forward_no_answer_enabled, forward_user_not_registered_destination, forward_user_not_registered_enabled, follow_me_uuid, follow_me_enabled, extension_language, extension_dialect, extension_voice, extension_type, enabled, description, created_at AS insert_date FROM extensions;")
    except Exception:
        pass

def downgrade() -> None:
    try:
        op.drop_index("ix_vars_domain_category", table_name="vars")
    except Exception:
        pass
    try:
        op.drop_index("ix_access_controls_domain", table_name="access_controls")
    except Exception:
        pass
    try:
        op.drop_index("ix_music_on_hold_domain", table_name="music_on_hold")
    except Exception:
        pass
