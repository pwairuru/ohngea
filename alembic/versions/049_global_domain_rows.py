"""global/shared rows for _domain-perm tables (PHP parity).

Revision ID: 049_global_domain_rows
Revises: 048_gateway_nullable_domain
PHP list pages include NULL-domain rows (unconditionally for most tables;
gated behind `*_domain`/`*_global` perms for gateways, call_block,
dialplans, music_on_hold): relax the columns and move the RLS policies
from strict to nullable shape (`tenant_all_*` from 047 is kept as-is).
"""

revision: str = "049_global_domain_rows"
down_revision: Union[str, None] = "048_gateway_nullable_domain"
branch_labels = None
depends_on = None

from alembic import op

TABLES = [
    "destinations",
    "ivr_menus",
    "devices",
    "voicemails",
    "conferences",
    "fax",
    "time_conditions",
    "call_flows",
    "phrases",
    "streams",
    "call_block",
    "dialplans",
    "dialplan_details",
    "music_on_hold",
]


def _exec(sql: str):
    try:
        op.execute(sql)
    except Exception:
        pass


def upgrade() -> None:
    for tbl in TABLES:
        _exec(f"ALTER TABLE IF EXISTS {tbl} ALTER COLUMN domain_id DROP NOT NULL;")
        _exec(f"DROP POLICY IF EXISTS tenant_{tbl} ON {tbl};")
        _exec(
            f"""
            CREATE POLICY tenant_{tbl} ON {tbl}
            USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
            WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
            """
        )


def downgrade() -> None:
    for tbl in TABLES:
        _exec(f"DELETE FROM {tbl} WHERE domain_id IS NULL;")
        _exec(f"ALTER TABLE IF EXISTS {tbl} ALTER COLUMN domain_id SET NOT NULL;")
        _exec(f"DROP POLICY IF EXISTS tenant_{tbl} ON {tbl};")
        _exec(
            f"""
            CREATE POLICY tenant_{tbl} ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
            """
        )
