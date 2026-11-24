"""devices + sip_profiles parity — missing child tables

Revision ID: 032_devices_sip_parity
Revises: 031_ring_ivr_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "032_devices_sip_parity"
down_revision: Union[str, None] = "031_ring_ivr_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # --- device_profiles (must be first for FK) ---
    try:
        op.create_table(
            "device_profiles",
            sa.Column("device_profile_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("device_profile_name", sa.String(length=128), nullable=False),
            sa.Column("device_profile_enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("device_profile_description", sa.String(length=512), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("device_profile_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_device_profiles_domain_id", "device_profiles", ["domain_id"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE device_profiles ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE device_profiles FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON device_profiles
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # add device_profile_uuid column to devices if missing
    try:
        op.add_column("devices", sa.Column("device_profile_uuid", sa.Uuid(), nullable=True))
    except Exception:
        pass
    try:
        op.create_foreign_key("fk_devices_device_profile", "devices", "device_profiles", ["device_profile_uuid"], ["device_profile_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_devices_device_profile_uuid", "devices", ["device_profile_uuid"])
    except Exception:
        pass

    # --- device_settings ---
    try:
        op.create_table(
            "device_settings",
            sa.Column("device_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("device_uuid", sa.Uuid(), nullable=False),
            sa.Column("device_profile_uuid", sa.Uuid(), nullable=True),
            sa.Column("device_setting_category", sa.String(length=64), nullable=True),
            sa.Column("device_setting_subcategory", sa.String(length=64), nullable=True),
            sa.Column("device_setting_name", sa.String(length=128), nullable=False),
            sa.Column("device_setting_value", sa.String(length=512), nullable=True),
            sa.Column("device_setting_enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("device_setting_description", sa.String(length=512), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["device_uuid"], ["devices.device_uuid"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["device_profile_uuid"], ["device_profiles.device_profile_uuid"], ondelete="SET NULL"),
            sa.PrimaryKeyConstraint("device_setting_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_device_settings_domain_id", "device_settings", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_device_settings_device_uuid", "device_settings", ["device_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE device_settings ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE device_settings FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON device_settings
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # --- device_keys ---
    try:
        op.create_table(
            "device_keys",
            sa.Column("device_key_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("device_uuid", sa.Uuid(), nullable=False),
            sa.Column("device_profile_uuid", sa.Uuid(), nullable=True),
            sa.Column("device_key_id", sa.Integer(), nullable=True),
            sa.Column("device_key_category", sa.String(length=64), nullable=True),
            sa.Column("device_key_vendor", sa.String(length=64), nullable=True),
            sa.Column("device_key_type", sa.String(length=64), nullable=True),
            sa.Column("device_key_subtype", sa.String(length=64), nullable=True),
            sa.Column("device_key_line", sa.Integer(), nullable=True),
            sa.Column("device_key_value", sa.String(length=128), nullable=True),
            sa.Column("device_key_extension", sa.String(length=64), nullable=True),
            sa.Column("device_key_protected", sa.String(length=16), nullable=True),
            sa.Column("device_key_label", sa.String(length=128), nullable=True),
            sa.Column("device_key_icon", sa.String(length=64), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["device_uuid"], ["devices.device_uuid"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["device_profile_uuid"], ["device_profiles.device_profile_uuid"], ondelete="SET NULL"),
            sa.PrimaryKeyConstraint("device_key_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_device_keys_domain_id", "device_keys", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_device_keys_device_uuid", "device_keys", ["device_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE device_keys ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE device_keys FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON device_keys
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # --- device_profile_keys ---
    try:
        op.create_table(
            "device_profile_keys",
            sa.Column("device_profile_key_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("device_profile_uuid", sa.Uuid(), nullable=False),
            sa.Column("profile_key_id", sa.Integer(), nullable=True),
            sa.Column("profile_key_category", sa.String(length=64), nullable=True),
            sa.Column("profile_key_vendor", sa.String(length=64), nullable=True),
            sa.Column("profile_key_type", sa.String(length=64), nullable=True),
            sa.Column("profile_key_subtype", sa.String(length=64), nullable=True),
            sa.Column("profile_key_line", sa.Integer(), nullable=True),
            sa.Column("profile_key_value", sa.String(length=128), nullable=True),
            sa.Column("profile_key_extension", sa.String(length=64), nullable=True),
            sa.Column("profile_key_protected", sa.Boolean(), nullable=False, server_default="false"),
            sa.Column("profile_key_label", sa.String(length=128), nullable=True),
            sa.Column("profile_key_icon", sa.String(length=64), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["device_profile_uuid"], ["device_profiles.device_profile_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("device_profile_key_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_device_profile_keys_domain_id", "device_profile_keys", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_device_profile_keys_device_profile_uuid", "device_profile_keys", ["device_profile_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE device_profile_keys ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE device_profile_keys FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON device_profile_keys
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # --- device_profile_settings ---
    try:
        op.create_table(
            "device_profile_settings",
            sa.Column("device_profile_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("device_profile_uuid", sa.Uuid(), nullable=False),
            sa.Column("profile_setting_name", sa.String(length=128), nullable=False),
            sa.Column("profile_setting_value", sa.String(length=512), nullable=True),
            sa.Column("profile_setting_enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("profile_setting_description", sa.String(length=512), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["device_profile_uuid"], ["device_profiles.device_profile_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("device_profile_setting_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_device_profile_settings_domain_id", "device_profile_settings", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_device_profile_settings_device_profile_uuid", "device_profile_settings", ["device_profile_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE device_profile_settings ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE device_profile_settings FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON device_profile_settings
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # --- sip_profile_domains ---
    try:
        op.create_table(
            "sip_profile_domains",
            sa.Column("sip_profile_domain_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("sip_profile_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_name", sa.String(length=128), nullable=False),
            sa.Column("domain_alias", sa.String(length=16), nullable=True, server_default="false"),
            sa.Column("domain_parse", sa.String(length=16), nullable=True, server_default="true"),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["sip_profile_uuid"], ["sip_profiles.sip_profile_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("sip_profile_domain_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_sip_profile_domains_domain_id", "sip_profile_domains", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_sip_profile_domains_sip_profile_uuid", "sip_profile_domains", ["sip_profile_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE sip_profile_domains ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE sip_profile_domains FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON sip_profile_domains
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # --- sip_profile_settings ---
    try:
        op.create_table(
            "sip_profile_settings",
            sa.Column("sip_profile_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("sip_profile_uuid", sa.Uuid(), nullable=False),
            sa.Column("setting_name", sa.String(length=128), nullable=False),
            sa.Column("setting_value", sa.String(length=512), nullable=True),
            sa.Column("enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("description", sa.String(length=512), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["sip_profile_uuid"], ["sip_profiles.sip_profile_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("sip_profile_setting_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_sip_profile_settings_domain_id", "sip_profile_settings", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_sip_profile_settings_sip_profile_uuid", "sip_profile_settings", ["sip_profile_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE sip_profile_settings ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE sip_profile_settings FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON sip_profile_settings
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # legacy compat views
    try:
        op.execute("DROP VIEW IF EXISTS v_devices;")
        op.execute(
            """CREATE VIEW v_devices AS SELECT
            device_uuid, domain_id AS domain_uuid, device_profile_uuid, device_address, mac AS device_mac_address,
            label AS device_label, vendor AS device_vendor, model AS device_model, device_firmware_version, device_template, device_location, device_serial_number, device_description, device_username, device_password, device_provisioned_date, device_provisioned_ip, device_provisioned_method,
            enabled AS device_enabled, extension_uuid AS device_extension_uuid,
            created_at AS insert_date, updated_at AS update_date FROM devices;"""
        )
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_lines;")
        op.execute("CREATE VIEW v_device_lines AS SELECT domain_id AS domain_uuid, device_line_uuid, device_uuid, line_number, server_address, server_address_primary, server_address_secondary, outbound_proxy_primary, outbound_proxy_secondary, label, display_name, user_id, auth_id, password, sip_port, sip_transport, register_expires, shared_line, enabled, created_at AS insert_date FROM device_lines;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_settings;")
        op.execute("CREATE VIEW v_device_settings AS SELECT domain_id AS domain_uuid, device_setting_uuid, device_uuid, device_profile_uuid, device_setting_category, device_setting_subcategory, device_setting_name, device_setting_value, device_setting_enabled, device_setting_description, created_at AS insert_date, updated_at AS update_date FROM device_settings;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_keys;")
        op.execute("CREATE VIEW v_device_keys AS SELECT domain_id AS domain_uuid, device_key_uuid, device_uuid, device_profile_uuid, device_key_id, device_key_category, device_key_vendor, device_key_type, device_key_subtype, device_key_line, device_key_value, device_key_extension, device_key_protected, device_key_label, device_key_icon, created_at AS insert_date, updated_at AS update_date FROM device_keys;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_profiles;")
        op.execute("CREATE VIEW v_device_profiles AS SELECT domain_id AS domain_uuid, device_profile_uuid, device_profile_name, device_profile_enabled, device_profile_description, created_at AS insert_date, updated_at AS update_date FROM device_profiles;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_profile_keys;")
        op.execute("CREATE VIEW v_device_profile_keys AS SELECT domain_id AS domain_uuid, device_profile_key_uuid, device_profile_uuid, profile_key_id, profile_key_category, profile_key_vendor, profile_key_type, profile_key_subtype, profile_key_line, profile_key_value, profile_key_extension, profile_key_protected, profile_key_label, profile_key_icon, created_at AS insert_date, updated_at AS update_date FROM device_profile_keys;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_profile_settings;")
        op.execute("CREATE VIEW v_device_profile_settings AS SELECT domain_id AS domain_uuid, device_profile_setting_uuid, device_profile_uuid, profile_setting_name, profile_setting_value, profile_setting_enabled, profile_setting_description, created_at AS insert_date, updated_at AS update_date FROM device_profile_settings;")
    except Exception:
        pass
    # hostname parity column (model has it, table missed it)
    try:
        op.add_column("sip_profiles", sa.Column("hostname", sa.String(length=128), nullable=True))
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_sip_profiles;")
        op.execute("CREATE VIEW v_sip_profiles AS SELECT sip_profile_uuid, domain_id AS domain_uuid, name AS sip_profile_name, hostname AS sip_profile_hostname, enabled AS sip_profile_enabled, description AS sip_profile_description, created_at AS insert_date, updated_at AS update_date FROM sip_profiles;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_sip_profile_domains;")
        op.execute("CREATE VIEW v_sip_profile_domains AS SELECT sip_profile_domain_uuid, domain_id AS domain_uuid, sip_profile_uuid, domain_name AS sip_profile_domain_name, domain_alias AS sip_profile_domain_alias, domain_parse AS sip_profile_domain_parse, created_at AS insert_date, updated_at AS update_date FROM sip_profile_domains;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_sip_profile_settings;")
        op.execute("CREATE VIEW v_sip_profile_settings AS SELECT sip_profile_setting_uuid, domain_id AS domain_uuid, sip_profile_uuid, setting_name AS sip_profile_setting_name, setting_value AS sip_profile_setting_value, enabled AS sip_profile_setting_enabled, description AS sip_profile_setting_description, created_at AS insert_date, updated_at AS update_date FROM sip_profile_settings;")
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_sip_profile_settings;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_sip_profile_domains;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_sip_profiles;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_profile_settings;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_profile_keys;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_profiles;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_keys;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_settings;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_device_lines;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_devices;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON sip_profile_settings;")
        op.execute("ALTER TABLE sip_profile_settings DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON sip_profile_domains;")
        op.execute("ALTER TABLE sip_profile_domains DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON device_profile_settings;")
        op.execute("ALTER TABLE device_profile_settings DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON device_profile_keys;")
        op.execute("ALTER TABLE device_profile_keys DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON device_keys;")
        op.execute("ALTER TABLE device_keys DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON device_settings;")
        op.execute("ALTER TABLE device_settings DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON device_profiles;")
        op.execute("ALTER TABLE device_profiles DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    for tbl in ["sip_profile_settings", "sip_profile_domains", "device_profile_settings", "device_profile_keys", "device_keys", "device_settings", "device_profiles"]:
        try:
            op.drop_table(tbl)
        except Exception:
            pass
    try:
        op.drop_constraint("fk_devices_device_profile", "devices", type_="foreignkey")
    except Exception:
        pass
    try:
        op.drop_column("devices", "device_profile_uuid")
    except Exception:
        pass
