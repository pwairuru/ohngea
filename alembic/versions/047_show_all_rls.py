"""cross-domain read unlock for ?show=all (X_all perm holders).

Revision ID: 047_show_all_rls
Revises: 046_services_parity
Adds a permissive `tenant_all_{tbl}` policy (USING current_setting
'app.domain_all' = '1') to every table that already has a `tenant_*` RLS
policy. The Rust API sets transaction-local `app.domain_all = '1'` only
inside the verified `?show=all` + JWT `_all`-perm path; writes stay
tenant-locked (no WITH CHECK on the new policy).
"""

revision: str = "047_show_all_rls"
down_revision: Union[str, None] = "046_services_parity"
branch_labels = None
depends_on = None

from alembic import op
import sqlalchemy as sa


def _exec(sql: str):
    try:
        op.execute(sql)
    except Exception:
        pass


def upgrade() -> None:
    tables = op.get_bind().execute(
        sa.text("SELECT tablename FROM pg_policies "
        "WHERE schemaname = 'public' AND policyname LIKE 'tenant\\_%' "
        "AND policyname NOT LIKE 'tenant\\_all\\_%'")
    ).fetchall()
    for (tbl,) in tables:
        _exec(f"DROP POLICY IF EXISTS tenant_all_{tbl} ON {tbl};")
        _exec(
            f"CREATE POLICY tenant_all_{tbl} ON {tbl} "
            f"USING (current_setting('app.domain_all', true) = '1')"
        )


def downgrade() -> None:
    tables = op.get_bind().execute(
        sa.text("SELECT tablename FROM pg_policies "
        "WHERE schemaname = 'public' AND policyname LIKE 'tenant\\_all\\_%'")
    ).fetchall()
    for (tbl,) in tables:
        _exec(f"DROP POLICY IF EXISTS tenant_all_{tbl} ON {tbl};")
