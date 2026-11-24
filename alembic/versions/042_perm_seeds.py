"""seed permissions for batch B/T/M/C endpoints + grant superadmin

Revision ID: 042_perm_seeds
Revises: 041_admin_editors
Create Date: 2026-09-23
"""
from typing import Sequence, Union

from alembic import op

revision: str = "042_perm_seeds"
down_revision: Union[str, None] = "041_admin_editors"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

PERMS = [
    "active_calls_hangup",
    "conferences_active_control",
    "conference_session_delete",
    "default_setting_view", "default_setting_add", "default_setting_edit", "default_setting_delete",
    "domain_setting_view", "domain_setting_add", "domain_setting_edit", "domain_setting_delete",
    "email_template_view", "email_template_add", "email_template_edit", "email_template_delete",
    "menu_view", "menu_add", "menu_edit", "menu_delete",
    "user_log_view", "user_log_delete",
    "notification_view", "notification_add", "notification_edit", "notification_delete",
    "service_view", "service_add", "service_edit", "service_delete",
    "contact_attachment_view", "contact_attachment_add", "contact_attachment_delete",
    "contact_group_view", "contact_group_add", "contact_group_delete",
    "contact_user_view", "contact_user_add", "contact_user_delete",
    "contact_setting_view", "contact_setting_add", "contact_setting_delete",
    "contact_relation_view", "contact_relation_add", "contact_relation_delete",
    "contact_time_view", "contact_time_add", "contact_time_delete",
]


def upgrade() -> None:
    values = ", ".join(f"(gen_random_uuid(), '{p}')" for p in PERMS)
    try:
        op.execute(f"INSERT INTO permissions (permission_uuid, name) VALUES {values} ON CONFLICT (name) DO NOTHING;")
    except Exception:
        pass
    names = ", ".join(f"'{p}'" for p in PERMS)
    try:
        op.execute(
            f"INSERT INTO group_permissions (id, group_uuid, permission_uuid) "
            f"SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p "
            f"WHERE g.name='superadmin' AND p.name IN ({names}) "
            f"AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);"
        )
    except Exception:
        pass


def downgrade() -> None:
    pass
