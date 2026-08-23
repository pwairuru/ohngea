"""groups+users full parity: group_level, protected, global domain, user fields, group_permissions denorm

Revision ID: 023_groups_users_parity
Revises: 022_gateways_full
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
import sqlmodel

revision: str = "023_groups_users_parity"
down_revision: Union[str, None] = "022_gateways_full"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # groups: make domain_id nullable (global), add level/protected/updated_at
    try:
        op.alter_column("groups", "domain_id", existing_type=sa.Uuid(), nullable=True)
    except Exception:
        pass
    try:
        op.add_column("groups", sa.Column("group_level", sa.Integer(), nullable=False, server_default="10"))
    except Exception:
        pass
    try:
        op.add_column("groups", sa.Column("group_protected", sa.Boolean(), nullable=False, server_default="false"))
    except Exception:
        pass
    try:
        op.add_column("groups", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    except Exception:
        pass
    try:
        op.execute("UPDATE groups SET updated_at = created_at WHERE updated_at IS NULL")
    except Exception:
        pass
    try:
        op.alter_column("groups", "updated_at", nullable=False)
    except Exception:
        pass
    try:
        op.create_index("ix_groups_group_level", "groups", ["group_level"])
    except Exception:
        pass
    try:
        op.create_index("ix_groups_domain_id", "groups", ["domain_id"])
    except Exception:
        pass
    # set defaults for existing superadmin-ish groups: try to set levels based on name
    try:
        op.execute("UPDATE groups SET group_level = 80 WHERE name = 'superadmin'")
    except Exception:
        pass
    try:
        op.execute("UPDATE groups SET group_level = 50 WHERE name = 'admin'")
    except Exception:
        pass
    try:
        op.execute("UPDATE groups SET group_level = 30 WHERE name = 'user'")
    except Exception:
        pass
    try:
        op.execute("UPDATE groups SET group_level = 20 WHERE name IN ('agent','fax')")
    except Exception:
        pass
    try:
        op.execute("UPDATE groups SET group_level = 10 WHERE name = 'public'")
    except Exception:
        pass

    # users: add legacy parity cols
    try:
        op.add_column("users", sa.Column("contact_uuid", sa.Uuid(), nullable=True))
    except Exception:
        pass
    try:
        op.add_column("users", sa.Column("user_status", sa.String(length=64), nullable=True))
    except Exception:
        pass
    try:
        op.add_column("users", sa.Column("user_type", sa.String(length=32), nullable=True, server_default="default"))
    except Exception:
        pass
    try:
        op.add_column("users", sa.Column("api_key", sa.String(length=64), nullable=True))
    except Exception:
        pass
    try:
        op.add_column("users", sa.Column("user_totp_secret", sa.String(length=64), nullable=True))
    except Exception:
        pass

    # user_groups: add group_name denorm
    try:
        op.add_column("user_groups", sa.Column("group_name", sa.String(length=64), nullable=True))
    except Exception:
        pass
    try:
        op.execute("UPDATE user_groups SET group_name = (SELECT name FROM groups WHERE groups.group_uuid = user_groups.group_uuid) WHERE group_name IS NULL")
    except Exception:
        pass

    # group_permissions: add denorm + flags
    try:
        op.add_column("group_permissions", sa.Column("domain_id", sa.Uuid(), nullable=True))
    except Exception:
        pass
    try:
        op.add_column("group_permissions", sa.Column("group_name", sa.String(length=64), nullable=True))
    except Exception:
        pass
    try:
        op.add_column("group_permissions", sa.Column("permission_name", sa.String(length=64), nullable=True))
    except Exception:
        pass
    try:
        op.add_column("group_permissions", sa.Column("permission_protected", sa.Boolean(), nullable=False, server_default="false"))
    except Exception:
        pass
    try:
        op.add_column("group_permissions", sa.Column("permission_assigned", sa.Boolean(), nullable=False, server_default="true"))
    except Exception:
        pass
    try:
        op.create_foreign_key("fk_group_permissions_domain_id", "group_permissions", "domains", ["domain_id"], ["domain_id"], ondelete="CASCADE")
    except Exception:
        pass
    try:
        op.execute("UPDATE group_permissions SET group_name = (SELECT name FROM groups WHERE groups.group_uuid = group_permissions.group_uuid) WHERE group_name IS NULL")
    except Exception:
        pass
    try:
        op.execute("UPDATE group_permissions SET permission_name = (SELECT name FROM permissions WHERE permissions.permission_uuid = group_permissions.permission_uuid) WHERE permission_name IS NULL")
    except Exception:
        pass
    try:
        op.execute("UPDATE group_permissions SET domain_id = (SELECT domain_id FROM groups WHERE groups.group_uuid = group_permissions.group_uuid) WHERE domain_id IS NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_group_permissions_group_name", "group_permissions", ["group_name"])
    except Exception:
        pass
    try:
        op.create_index("ix_group_permissions_permission_name", "group_permissions", ["permission_name"])
    except Exception:
        pass

    # user_settings table
    try:
        op.create_table(
            "user_settings",
            sa.Column("user_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("user_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("user_setting_category", sa.String(length=64), nullable=False),
            sa.Column("user_setting_subcategory", sa.String(length=64), nullable=True),
            sa.Column("user_setting_name", sa.String(length=64), nullable=True),
            sa.Column("user_setting_value", sa.String(length=512), nullable=True),
            sa.Column("user_setting_enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("user_setting_description", sa.String(length=512), nullable=True),
            sa.PrimaryKeyConstraint("user_setting_uuid"),
            sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_user_settings_user_uuid", "user_settings", ["user_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_user_settings_domain_id", "user_settings", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_user_settings_category", "user_settings", ["user_setting_category"])
    except Exception:
        pass
    try:
        op.create_index("ix_user_settings_subcategory", "user_settings", ["user_setting_subcategory"])
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.drop_table("user_settings")
    except Exception:
        pass
    try:
        op.drop_index("ix_group_permissions_permission_name", table_name="group_permissions")
    except Exception:
        pass
    try:
        op.drop_index("ix_group_permissions_group_name", table_name="group_permissions")
    except Exception:
        pass
    try:
        op.drop_constraint("fk_group_permissions_domain_id", "group_permissions", type_="foreignkey")
    except Exception:
        pass
    for c in ["permission_assigned", "permission_protected", "permission_name", "group_name", "domain_id"]:
        try:
            op.drop_column("group_permissions", c)
        except Exception:
            pass
    try:
        op.drop_column("user_groups", "group_name")
    except Exception:
        pass
    for c in ["user_totp_secret", "api_key", "user_type", "user_status", "contact_uuid"]:
        try:
            op.drop_column("users", c)
        except Exception:
            pass
    try:
        op.drop_index("ix_groups_domain_id", table_name="groups")
    except Exception:
        pass
    try:
        op.drop_index("ix_groups_group_level", table_name="groups")
    except Exception:
        pass
    for c in ["updated_at", "group_protected", "group_level"]:
        try:
            op.drop_column("groups", c)
        except Exception:
            pass
    try:
        op.alter_column("groups", "domain_id", existing_type=sa.Uuid(), nullable=False)
    except Exception:
        pass
