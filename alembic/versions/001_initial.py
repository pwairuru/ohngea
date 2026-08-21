"""initial — domains, users, extensions, dialplans, gateways, sip_profiles, ivr, devices, cdr partitioned

Revision ID: 001_initial
Revises:
Create Date: 2026-08-21
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel

# revision identifiers, used by Alembic.
revision: str = "001_initial"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto;")
    # domains
    op.create_table(
        "domains",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("domain_name", sa.String(length=255), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("description", sa.String(length=512), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("domain_id"),
        sa.UniqueConstraint("domain_name"),
    )
    op.create_index("ix_domains_domain_name", "domains", ["domain_name"])

    op.create_table(
        "domain_settings",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("category", sa.String(length=64), nullable=False),
        sa.Column("subcategory", sa.String(length=64), nullable=False),
        sa.Column("key", sa.String(length=64), nullable=False),
        sa.Column("value", sa.String(length=1024), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
    )

    op.create_table(
        "domain_plugins",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("plugin_name", sa.String(length=64), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
    )

    # permissions
    op.create_table(
        "permissions",
        sa.Column("permission_uuid", sa.Uuid(), nullable=False),
        sa.Column("name", sa.String(length=64), nullable=False),
        sa.Column("description", sa.String(length=255), nullable=True),
        sa.PrimaryKeyConstraint("permission_uuid"),
        sa.UniqueConstraint("name"),
    )

    # groups
    op.create_table(
        "groups",
        sa.Column("group_uuid", sa.Uuid(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("name", sa.String(length=64), nullable=False),
        sa.Column("description", sa.String(length=255), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("group_uuid"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
    )

    op.create_table(
        "users",
        sa.Column("user_uuid", sa.Uuid(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("username", sa.String(length=64), nullable=False),
        sa.Column("email", sa.String(length=255), nullable=True),
        sa.Column("password_hash", sa.String(length=255), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("user_uuid"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
    )
    op.create_index("ix_users_username", "users", ["username"])

    op.create_table(
        "user_groups",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("user_uuid", sa.Uuid(), nullable=False),
        sa.Column("group_uuid", sa.Uuid(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.PrimaryKeyConstraint("id"),
        sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["group_uuid"], ["groups.group_uuid"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
    )

    op.create_table(
        "group_permissions",
        sa.Column("id", sa.Uuid(), nullable=False),
        sa.Column("group_uuid", sa.Uuid(), nullable=False),
        sa.Column("permission_uuid", sa.Uuid(), nullable=False),
        sa.PrimaryKeyConstraint("id"),
        sa.ForeignKeyConstraint(["group_uuid"], ["groups.group_uuid"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["permission_uuid"], ["permissions.permission_uuid"], ondelete="CASCADE"),
    )

    # extensions
    op.create_table(
        "extensions",
        sa.Column("extension", sa.String(), nullable=False),
        sa.Column("number_alias", sa.String(length=32), nullable=True),
        sa.Column("effective_caller_id_name", sa.String(length=128), nullable=False),
        sa.Column("effective_caller_id_number", sa.String(length=32), nullable=True),
        sa.Column("call_group", sa.String(length=32), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("extension_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("extension_uuid"),
    )
    op.create_index("ix_extensions_domain_id", "extensions", ["domain_id"])
    op.create_index("ix_extensions_domain_extension", "extensions", ["domain_id", "extension"])

    # dialplans
    op.create_table(
        "dialplans",
        sa.Column("name", sa.String(length=128), nullable=False),
        sa.Column("context", sa.String(length=128), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("description", sa.String(), nullable=True),
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("dialplan_uuid"),
    )

    op.create_table(
        "dialplan_details",
        sa.Column("detail_uuid", sa.Uuid(), nullable=False),
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=False),
        sa.Column("tag", sa.String(length=32), nullable=False),
        sa.Column("type", sa.String(length=64), nullable=False),
        sa.Column("data", sa.String(length=512), nullable=False),
        sa.Column("order", sa.Integer(), nullable=False),
        sa.ForeignKeyConstraint(["dialplan_uuid"], ["dialplans.dialplan_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("detail_uuid"),
    )

    # gateways
    op.create_table(
        "gateways",
        sa.Column("gateway", sa.String(length=64), nullable=False),
        sa.Column("username", sa.String(length=64), nullable=True),
        sa.Column("password", sa.String(length=64), nullable=True),
        sa.Column("proxy", sa.String(length=128), nullable=True),
        sa.Column("register", sa.Boolean(), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("gateway_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("gateway_uuid"),
    )

    op.create_table(
        "sip_profiles",
        sa.Column("name", sa.String(length=64), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("sip_port", sa.Integer(), nullable=False),
        sa.Column("sip_ip", sa.String(), nullable=True),
        sa.Column("sip_profile_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("sip_profile_uuid"),
    )

    op.create_table(
        "ivr_menus",
        sa.Column("name", sa.String(length=128), nullable=False),
        sa.Column("extension", sa.String(length=32), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("ivr_menu_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("ivr_menu_uuid"),
    )

    op.create_table(
        "ivr_menu_options",
        sa.Column("option_uuid", sa.Uuid(), nullable=False),
        sa.Column("ivr_menu_uuid", sa.Uuid(), nullable=False),
        sa.Column("digits", sa.String(length=8), nullable=False),
        sa.Column("action", sa.String(length=64), nullable=False),
        sa.Column("param", sa.String(length=255), nullable=False),
        sa.Column("order", sa.Integer(), nullable=False),
        sa.ForeignKeyConstraint(["ivr_menu_uuid"], ["ivr_menus.ivr_menu_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("option_uuid"),
    )

    op.create_table(
        "devices",
        sa.Column("mac", sa.String(length=17), nullable=False),
        sa.Column("vendor", sa.String(length=32), nullable=False),
        sa.Column("model", sa.String(length=64), nullable=True),
        sa.Column("label", sa.String(length=128), nullable=True),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("extension_uuid", sa.Uuid(), nullable=True),
        sa.Column("device_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["extension_uuid"], ["extensions.extension_uuid"]),
        sa.PrimaryKeyConstraint("device_uuid"),
    )

    # CDR partitioned by HASH(domain_id) 32 partitions
    op.execute(
        """
        CREATE TABLE cdr (
            cdr_uuid uuid NOT NULL DEFAULT gen_random_uuid(),
            domain_id uuid NOT NULL,
            caller_id_number text,
            caller_id_name text,
            destination_number text NOT NULL,
            start_stamp timestamptz NOT NULL,
            answer_stamp timestamptz,
            end_stamp timestamptz,
            duration integer,
            billsec integer,
            direction text,
            sip_call_id text,
            hangup_cause text,
            created_at timestamptz NOT NULL DEFAULT now(),
            PRIMARY KEY (cdr_uuid, domain_id)
        ) PARTITION BY HASH (domain_id);
        """
    )
    for i in range(32):
        op.execute(f"CREATE TABLE cdr_p{i} PARTITION OF cdr FOR VALUES WITH (MODULUS 32, REMAINDER {i});")
        op.execute(f"CREATE INDEX ON cdr_p{i} (domain_id, start_stamp DESC);")
        op.execute(f"CREATE INDEX ON cdr_p{i} (domain_id, destination_number);")

    # RLS — enable FORCE on tenant tables
    for tbl in ["extensions", "dialplans", "gateways", "sip_profiles", "ivr_menus", "devices"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        # policy using current_setting; fallback to true if not set (for superadmin migrations)
        op.execute(
            f"""
            CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
            """
        )
    # CDR partition parent needs RLS too (applies to partitions)
    op.execute("ALTER TABLE cdr ENABLE ROW LEVEL SECURITY;")
    op.execute("ALTER TABLE cdr FORCE ROW LEVEL SECURITY;")
    op.execute(
        """
        CREATE POLICY tenant_isolation ON cdr
        USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
        WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
        """
    )


def downgrade() -> None:
    op.execute("DROP TABLE IF EXISTS cdr CASCADE;")
    for tbl in ["devices", "ivr_menu_options", "ivr_menus", "sip_profiles", "gateways", "dialplan_details", "dialplans", "extensions", "group_permissions", "user_groups", "users", "groups", "permissions", "domain_plugins", "domain_settings", "domains"]:
        op.execute(f"DROP TABLE IF EXISTS {tbl} CASCADE;")
