"""groups users RLS + orphan wildcard fix

Revision ID: 026_groups_users_rls
Revises: 025_webrtc_settings_rls
Create Date: 2026-08-23
"""
from typing import Sequence, Union

from alembic import op

revision: str = "026_groups_users_rls"
down_revision: Union[str, None] = "025_webrtc_settings_rls"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # groups: domain_id IS NULL OR equals current_setting
    try:
        op.execute("ALTER TABLE groups ENABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE groups FORCE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_groups ON groups;")
    except Exception:
        pass
    try:
        op.execute(
            """
            CREATE POLICY tenant_groups ON groups
            USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
            WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
            """
        )
    except Exception:
        pass

    for tbl in ["users", "user_groups", "user_settings", "group_permissions"]:
        try:
            op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        except Exception:
            pass
        try:
            op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        except Exception:
            pass
        try:
            op.execute(f"DROP POLICY IF EXISTS tenant_{tbl} ON {tbl};")
        except Exception:
            pass
        try:
            op.execute(
                f"""
                CREATE POLICY tenant_{tbl} ON {tbl}
                USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
                WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
                """
            )
        except Exception:
            pass
    # ensure permissions stays without RLS (global)
    try:
        op.execute("ALTER TABLE permissions DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP POLICY IF EXISTS tenant_groups ON groups;")
    except Exception:
        pass
    for tbl in ["users", "user_groups", "user_settings", "group_permissions"]:
        try:
            op.execute(f"DROP POLICY IF EXISTS tenant_{tbl} ON {tbl};")
        except Exception:
            pass
    for tbl in ["groups", "users", "user_groups", "user_settings", "group_permissions"]:
        try:
            op.execute(f"ALTER TABLE {tbl} DISABLE ROW LEVEL SECURITY;")
        except Exception:
            pass
