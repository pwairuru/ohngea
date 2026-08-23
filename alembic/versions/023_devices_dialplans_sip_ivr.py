"""devices dialplans sip_profiles ivr full parity

Revision ID: 023_devices_dialplans_sip_ivr
Revises: 022_gateways_full
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "023_devices_dialplans_sip_ivr"
down_revision: Union[str, None] = "023_groups_users_parity"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    # devices add columns
    op.add_column("devices", sa.Column("device_address", sa.String(length=32), nullable=True))
    op.add_column("devices", sa.Column("device_label", sa.String(length=128), nullable=True))
    op.add_column("devices", sa.Column("device_vendor", sa.String(length=32), nullable=True))
    op.add_column("devices", sa.Column("device_model", sa.String(length=64), nullable=True))
    op.add_column("devices", sa.Column("device_firmware_version", sa.String(length=64), nullable=True))
    op.add_column("devices", sa.Column("device_template", sa.String(length=128), nullable=True))
    op.add_column("devices", sa.Column("device_location", sa.String(length=128), nullable=True))
    op.add_column("devices", sa.Column("device_serial_number", sa.String(length=128), nullable=True))
    op.add_column("devices", sa.Column("device_description", sa.String(length=512), nullable=True))
    op.add_column("devices", sa.Column("device_username", sa.String(length=64), nullable=True))
    op.add_column("devices", sa.Column("device_password", sa.String(length=64), nullable=True))
    op.add_column("devices", sa.Column("device_provisioned_date", sa.DateTime(timezone=True), nullable=True))
    op.add_column("devices", sa.Column("device_provisioned_ip", sa.String(length=64), nullable=True))
    op.add_column("devices", sa.Column("device_provisioned_method", sa.String(length=32), nullable=True))
    op.add_column("devices", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    op.execute("UPDATE devices SET updated_at = created_at WHERE updated_at IS NULL")
    try:
        op.alter_column("devices", "updated_at", nullable=False)
    except Exception:
        pass
    # device_lines table (new)
    op.create_table("device_lines",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("device_uuid", sa.Uuid(), nullable=False),
        sa.Column("line_number", sa.String(length=8), nullable=False, server_default="1"),
        sa.Column("server_address", sa.String(length=128), nullable=True),
        sa.Column("server_address_primary", sa.String(length=128), nullable=True),
        sa.Column("server_address_secondary", sa.String(length=128), nullable=True),
        sa.Column("outbound_proxy_primary", sa.String(length=128), nullable=True),
        sa.Column("outbound_proxy_secondary", sa.String(length=128), nullable=True),
        sa.Column("label", sa.String(length=128), nullable=True),
        sa.Column("display_name", sa.String(length=128), nullable=True),
        sa.Column("user_id", sa.String(length=64), nullable=True),
        sa.Column("auth_id", sa.String(length=64), nullable=True),
        sa.Column("password", sa.String(length=64), nullable=True),
        sa.Column("sip_port", sa.Integer(), nullable=True),
        sa.Column("sip_transport", sa.String(length=32), nullable=True),
        sa.Column("register_expires", sa.Integer(), nullable=True),
        sa.Column("shared_line", sa.String(length=32), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False, server_default="true"),
        sa.Column("device_line_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["device_uuid"], ["devices.device_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("device_line_uuid")
    )
    op.create_index("ix_device_lines_device_uuid", "device_lines", ["device_uuid"])
    op.create_index("ix_device_lines_domain_id", "device_lines", ["domain_id"])
    # dialplans
    op.add_column("dialplans", sa.Column("dialplan_continue", sa.Boolean(), nullable=False, server_default="false"))
    op.add_column("dialplans", sa.Column("dialplan_order", sa.Integer(), nullable=True, server_default="100"))
    op.add_column("dialplans", sa.Column("hostname", sa.String(length=128), nullable=True))
    op.add_column("dialplans", sa.Column("app_uuid", sa.String(length=64), nullable=True))
    # sip_profiles
    op.add_column("sip_profiles", sa.Column("sip_secure", sa.Boolean(), nullable=True))
    op.add_column("sip_profiles", sa.Column("rtp_ip", sa.String(length=64), nullable=True))
    op.add_column("sip_profiles", sa.Column("rtp_port_range", sa.String(length=32), nullable=True))
    op.add_column("sip_profiles", sa.Column("hold_music", sa.String(length=128), nullable=True))
    op.add_column("sip_profiles", sa.Column("apply_inbound_acl", sa.String(length=128), nullable=True))
    op.add_column("sip_profiles", sa.Column("local_network_acl", sa.String(length=128), nullable=True))
    op.add_column("sip_profiles", sa.Column("record_path", sa.String(length=256), nullable=True))
    op.add_column("sip_profiles", sa.Column("record_template", sa.String(length=128), nullable=True))
    op.add_column("sip_profiles", sa.Column("description", sa.String(length=512), nullable=True))
    op.add_column("sip_profiles", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    op.execute("UPDATE sip_profiles SET updated_at = created_at WHERE updated_at IS NULL")
    try:
        op.alter_column("sip_profiles", "updated_at", nullable=False)
    except Exception:
        pass
    # ivr_menus
    op.add_column("ivr_menus", sa.Column("greet_long", sa.String(length=256), nullable=True))
    op.add_column("ivr_menus", sa.Column("greet_short", sa.String(length=256), nullable=True))
    op.add_column("ivr_menus", sa.Column("invalid_sound", sa.String(length=256), nullable=True))
    op.add_column("ivr_menus", sa.Column("exit_sound", sa.String(length=256), nullable=True))
    op.add_column("ivr_menus", sa.Column("confirm_macro", sa.String(length=128), nullable=True))
    op.add_column("ivr_menus", sa.Column("confirm_key", sa.String(length=8), nullable=True))
    op.add_column("ivr_menus", sa.Column("tts_engine", sa.String(length=32), nullable=True))
    op.add_column("ivr_menus", sa.Column("tts_voice", sa.String(length=32), nullable=True))
    op.add_column("ivr_menus", sa.Column("confirm_attempts", sa.Integer(), nullable=True, server_default="3"))
    op.add_column("ivr_menus", sa.Column("timeout", sa.Integer(), nullable=True, server_default="10000"))
    op.add_column("ivr_menus", sa.Column("inter_digit_timeout", sa.Integer(), nullable=True, server_default="2000"))
    op.add_column("ivr_menus", sa.Column("max_failures", sa.Integer(), nullable=True, server_default="3"))
    op.add_column("ivr_menus", sa.Column("max_timeouts", sa.Integer(), nullable=True, server_default="3"))
    op.add_column("ivr_menus", sa.Column("digit_len", sa.Integer(), nullable=True, server_default="4"))
    op.add_column("ivr_menus", sa.Column("description", sa.String(length=512), nullable=True))
    op.add_column("ivr_menus", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    op.execute("UPDATE ivr_menus SET updated_at = created_at WHERE updated_at IS NULL")
    try:
        op.alter_column("ivr_menus", "updated_at", nullable=False)
    except Exception:
        pass
    # add RLS for device_lines
    try:
        op.execute("ALTER TABLE device_lines ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE device_lines FORCE ROW LEVEL SECURITY;")
        op.execute("CREATE POLICY tenant_isolation ON device_lines USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');")
    except Exception:
        pass

def downgrade() -> None:
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON device_lines;")
        op.execute("ALTER TABLE device_lines DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    op.drop_table("device_lines")
    for tbl,col in [("devices",["device_address","device_label","device_vendor","device_model","device_firmware_version","device_template","device_location","device_serial_number","device_description","device_username","device_password","device_provisioned_date","device_provisioned_ip","device_provisioned_method","updated_at"]),("dialplans",["dialplan_continue","dialplan_order","hostname","app_uuid"]),("sip_profiles",["sip_secure","rtp_ip","rtp_port_range","hold_music","apply_inbound_acl","local_network_acl","record_path","record_template","description","updated_at"]),("ivr_menus",["greet_long","greet_short","invalid_sound","exit_sound","confirm_macro","confirm_key","tts_engine","tts_voice","confirm_attempts","timeout","inter_digit_timeout","max_failures","max_timeouts","digit_len","description","updated_at"])]:
        for c in col:
            try:
                op.drop_column(tbl, c)
            except Exception:
                pass
