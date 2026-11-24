"""admin editors tables: default/domain settings, email templates, menu items, user logs, notifications, services

Revision ID: 041_admin_editors
Revises: 040_contacts_fix
Create Date: 2026-09-23
"""
from typing import Sequence, Union

from alembic import op

revision: str = "041_admin_editors"
down_revision: Union[str, None] = "040_contacts_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _exec(sql: str):
    try:
        op.execute(sql)
    except Exception:
        pass


def upgrade() -> None:
    # default_settings canonical shape lives in 035; ensure-exists with same columns.
    _exec("""
        CREATE TABLE IF NOT EXISTS default_settings (
            default_setting_uuid UUID PRIMARY KEY,
            default_setting_category VARCHAR(64) NOT NULL,
            default_setting_subcategory VARCHAR(64) NOT NULL DEFAULT '',
            default_setting_name VARCHAR(64) NOT NULL,
            default_setting_value VARCHAR(1024),
            default_setting_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            default_setting_description VARCHAR(512),
            created_at TIMESTAMP NOT NULL DEFAULT now(),
            updated_at TIMESTAMP NOT NULL DEFAULT now()
        );
    """)
    _exec("CREATE INDEX IF NOT EXISTS ix_domain_settings_domain ON domain_settings (domain_id);")
    _exec("""
        CREATE TABLE IF NOT EXISTS email_templates (
            email_template_uuid UUID PRIMARY KEY,
            domain_id UUID REFERENCES domains(domain_id) ON DELETE CASCADE,
            template_name VARCHAR(128) NOT NULL,
            template_language VARCHAR(16),
            template_subject VARCHAR(512),
            template_body TEXT,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            description VARCHAR(512),
            created_at TIMESTAMP NOT NULL DEFAULT now(),
            updated_at TIMESTAMP NOT NULL DEFAULT now()
        );
    """)
    _exec("""
        CREATE TABLE IF NOT EXISTS menu_items (
            menu_item_uuid UUID PRIMARY KEY,
            parent_uuid UUID REFERENCES menu_items(menu_item_uuid) ON DELETE CASCADE,
            title VARCHAR(128) NOT NULL,
            path VARCHAR(256),
            icon VARCHAR(64),
            item_order INTEGER NOT NULL DEFAULT 0,
            groups VARCHAR(512),
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            description VARCHAR(512),
            created_at TIMESTAMP NOT NULL DEFAULT now(),
            updated_at TIMESTAMP NOT NULL DEFAULT now()
        );
    """)
    _exec("""
        CREATE TABLE IF NOT EXISTS user_logs (
            user_log_uuid UUID PRIMARY KEY,
            domain_id UUID REFERENCES domains(domain_id) ON DELETE CASCADE,
            user_uuid UUID,
            username VARCHAR(128),
            action VARCHAR(64) NOT NULL,
            detail VARCHAR(1024),
            ip_address VARCHAR(64),
            created_at TIMESTAMP NOT NULL DEFAULT now()
        );
    """)
    _exec("CREATE INDEX IF NOT EXISTS ix_user_logs_domain ON user_logs (domain_id);")
    _exec("CREATE INDEX IF NOT EXISTS ix_user_logs_created ON user_logs (created_at DESC);")
    _exec("""
        CREATE TABLE IF NOT EXISTS notifications (
            notification_uuid UUID PRIMARY KEY,
            domain_id UUID REFERENCES domains(domain_id) ON DELETE CASCADE,
            user_uuid UUID,
            title VARCHAR(256) NOT NULL,
            body VARCHAR(2048),
            link VARCHAR(512),
            read BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMP NOT NULL DEFAULT now()
        );
    """)
    _exec("CREATE INDEX IF NOT EXISTS ix_notifications_user ON notifications (user_uuid);")
    _exec("""
        CREATE TABLE IF NOT EXISTS services (
            service_uuid UUID PRIMARY KEY,
            name VARCHAR(128) NOT NULL,
            host VARCHAR(128),
            port INTEGER,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            service_order INTEGER NOT NULL DEFAULT 0,
            description VARCHAR(512),
            created_at TIMESTAMP NOT NULL DEFAULT now(),
            updated_at TIMESTAMP NOT NULL DEFAULT now()
        );
    """)
    # RLS on domain-scoped tables (nullable-domain pattern like groups)
    for tbl in ["domain_settings", "email_templates", "user_logs", "notifications"]:
        _exec(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        _exec(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        _exec(f"DROP POLICY IF EXISTS tenant_{tbl} ON {tbl};")
        _exec(
            f"""
            CREATE POLICY tenant_{tbl} ON {tbl}
            USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
            WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
            """
        )


def downgrade() -> None:
    for tbl in ["services", "notifications", "user_logs", "menu_items", "email_templates", "domain_settings", "default_settings"]:
        _exec(f"DROP TABLE IF EXISTS {tbl};")
