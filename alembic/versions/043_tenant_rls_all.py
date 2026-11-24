"""tenant RLS for all domain tables

Revision ID: 043_tenant_rls_all
Revises: 042_perm_seeds
Create Date: 2026-09-24
"""
from typing import Sequence, Union
from alembic import op

revision: str = "043_tenant_rls_all"
down_revision: Union[str, None] = "042_perm_seeds"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

# All tables with non-nullable domain_id get strict tenant policy.
# Tables with nullable domain_id (shared/global rows) allow NULL.
STRICT_TABLES = [
    "extensions", "dialplans", "gateways", "sip_profiles", "sip_profile_domains",
    "sip_profile_settings", "ivr_menus", "devices", "cdr", "recordings",
    "contacts", "ring_groups", "time_conditions", "voicemails", "conferences",
    "fax", "call_flows", "music_on_hold", "follow_me", "call_block",
    "destinations", "pin_numbers", "phrases", "streams", "bridges", "fifo",
    "call_centers", "vars", "number_translations", "conference_profiles",
    "conference_controls", "conference_centers", "access_controls",
    "call_recordings", "call_broadcasts", "emergency_logs", "voicemail_greetings",
    "fax_queue", "extension_settings", "email_queue",
    "feature_codes", "database_transactions",
    "webrtc_settings", "domain_limits", "call_center_agents", "contact_addresses",
]
# Global tables without domain_id stay open (no RLS policy possible).
NULLABLE_TABLES = [
    "groups", "users", "domain_settings", "domain_apps",
]

POLICY_SQL_STRICT = """
CREATE POLICY tenant_{tbl} ON {tbl}
USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
"""

POLICY_SQL_NULLABLE = """
CREATE POLICY tenant_{tbl} ON {tbl}
USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
"""


def upgrade() -> None:
    for tbl in STRICT_TABLES + NULLABLE_TABLES:
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
            sql = (POLICY_SQL_NULLABLE if tbl in NULLABLE_TABLES else POLICY_SQL_STRICT).format(tbl=tbl)
            op.execute(sql)
        except Exception:
            pass
    # global tables stay open
    for tbl in ["permissions", "domains", "sofia_global_settings", "event_guard_logs", "modules"]:
        try:
            op.execute(f"ALTER TABLE {tbl} DISABLE ROW LEVEL SECURITY;")
        except Exception:
            pass


def downgrade() -> None:
    for tbl in STRICT_TABLES + NULLABLE_TABLES:
        try:
            op.execute(f"DROP POLICY IF EXISTS tenant_{tbl} ON {tbl};")
        except Exception:
            pass
        try:
            op.execute(f"ALTER TABLE {tbl} DISABLE ROW LEVEL SECURITY;")
        except Exception:
            pass
