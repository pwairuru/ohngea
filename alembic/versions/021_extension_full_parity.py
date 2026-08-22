"""extension full parity — add all legacy fields

Revision ID: 021_extension_full_parity
Revises: 020_domain_apps
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "021_extension_full_parity"
down_revision: Union[str, None] = "020_domain_apps"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    # Add all missing legacy columns to extensions; nullable with sensible defaults for existing rows
    # password etc
    op.add_column("extensions", sa.Column("password", sa.String(length=64), nullable=True))
    op.add_column("extensions", sa.Column("accountcode", sa.String(length=64), nullable=True))
    op.add_column("extensions", sa.Column("outbound_caller_id_name", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("outbound_caller_id_number", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("emergency_caller_id_name", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("emergency_caller_id_number", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("directory_first_name", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("directory_last_name", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("directory_visible", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("directory_exten_visible", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("max_registrations", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("limit_max", sa.String(length=32), nullable=True, server_default="5"))
    op.add_column("extensions", sa.Column("limit_destination", sa.String(length=128), nullable=True, server_default="!USER_BUSY"))
    op.add_column("extensions", sa.Column("missed_call_app", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("missed_call_data", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("user_context", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("toll_allow", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("call_timeout", sa.Integer(), nullable=True, server_default="30"))
    op.add_column("extensions", sa.Column("call_screen_enabled", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("user_record", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("hold_music", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("auth_acl", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("cidr", sa.String(length=512), nullable=True))
    op.add_column("extensions", sa.Column("sip_force_contact", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("sip_force_expires", sa.Integer(), nullable=True))
    op.add_column("extensions", sa.Column("nibble_account", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("mwi_account", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("sip_bypass_media", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("absolute_codec_string", sa.String(length=256), nullable=True))
    op.add_column("extensions", sa.Column("force_ping", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("dial_string", sa.String(length=1024), nullable=True))
    op.add_column("extensions", sa.Column("do_not_disturb", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("forward_all_destination", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("forward_all_enabled", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("forward_busy_destination", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("forward_busy_enabled", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("forward_no_answer_destination", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("forward_no_answer_enabled", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("forward_user_not_registered_destination", sa.String(length=128), nullable=True))
    op.add_column("extensions", sa.Column("forward_user_not_registered_enabled", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("follow_me_uuid", sa.Uuid(), nullable=True))
    op.add_column("extensions", sa.Column("follow_me_enabled", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("extensions", sa.Column("extension_language", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("extension_dialect", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("extension_voice", sa.String(length=32), nullable=True))
    op.add_column("extensions", sa.Column("extension_type", sa.String(length=32), nullable=True, server_default="default"))
    op.add_column("extensions", sa.Column("description", sa.String(length=512), nullable=True))
    # FK for follow_me
    try:
        op.create_foreign_key("fk_extensions_follow_me", "extensions", "follow_me", ["follow_me_uuid"], ["follow_me_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    # Compat views for legacy code expecting v_extensions
    op.execute("DROP VIEW IF EXISTS v_extensions;")
    op.execute("CREATE VIEW v_extensions AS SELECT domain_id AS domain_uuid, extension_uuid, extension, number_alias, password, accountcode, effective_caller_id_name, effective_caller_id_number, outbound_caller_id_name, outbound_caller_id_number, emergency_caller_id_name, emergency_caller_id_number, directory_first_name, directory_last_name, directory_visible, directory_exten_visible, max_registrations, limit_max, limit_destination, missed_call_app, missed_call_data, user_context, toll_allow, call_timeout, call_group, call_screen_enabled, user_record, hold_music, auth_acl, cidr, sip_force_contact, sip_force_expires, nibble_account, mwi_account, sip_bypass_media, absolute_codec_string, force_ping, dial_string, do_not_disturb, forward_all_destination, forward_all_enabled, forward_busy_destination, forward_busy_enabled, forward_no_answer_destination, forward_no_answer_enabled, forward_user_not_registered_destination, forward_user_not_registered_enabled, follow_me_uuid, follow_me_enabled, extension_language, extension_dialect, extension_voice, extension_type, enabled, description, created_at AS insert_date, updated_at AS update_date FROM extensions;")
    op.execute("DROP VIEW IF EXISTS v_extension_users;")
    op.execute("CREATE VIEW v_extension_users AS SELECT extension_uuid, domain_id AS domain_uuid, NULL::uuid AS user_uuid FROM extensions WHERE false;")  # placeholder compat

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_extension_users;")
    op.execute("DROP VIEW IF EXISTS v_extensions;")
    try:
        op.drop_constraint("fk_extensions_follow_me", "extensions", type_="foreignkey")
    except Exception:
        pass
    cols = ["password","accountcode","outbound_caller_id_name","outbound_caller_id_number","emergency_caller_id_name","emergency_caller_id_number","directory_first_name","directory_last_name","directory_visible","directory_exten_visible","max_registrations","limit_max","limit_destination","missed_call_app","missed_call_data","user_context","toll_allow","call_timeout","call_screen_enabled","user_record","hold_music","auth_acl","cidr","sip_force_contact","sip_force_expires","nibble_account","mwi_account","sip_bypass_media","absolute_codec_string","force_ping","dial_string","do_not_disturb","forward_all_destination","forward_all_enabled","forward_busy_destination","forward_busy_enabled","forward_no_answer_destination","forward_no_answer_enabled","forward_user_not_registered_destination","forward_user_not_registered_enabled","follow_me_uuid","follow_me_enabled","extension_language","extension_dialect","extension_voice","extension_type","description"]
    for c in cols:
        try:
            op.drop_column("extensions", c)
        except Exception:
            pass
