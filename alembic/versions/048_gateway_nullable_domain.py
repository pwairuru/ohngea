"""global/shared gateways (PHP `gateway_domain` parity).

Revision ID: 048_gateway_nullable_domain
Revises: 047_show_all_rls
`v_gateways` rows may have NULL domain_uuid (shared across tenants):
relax the column and move the RLS policy from strict to nullable shape
(`tenant_all_gateways` from 047 is kept as-is).
"""

revision: str = "048_gateway_nullable_domain"
down_revision: Union[str, None] = "047_show_all_rls"
branch_labels = None
depends_on = None

from alembic import op


def _exec(sql: str):
    try:
        op.execute(sql)
    except Exception:
        pass


def upgrade() -> None:
    _exec("ALTER TABLE gateways ALTER COLUMN domain_id DROP NOT NULL;")
    _exec("DROP POLICY IF EXISTS tenant_gateways ON gateways;")
    _exec(
        """
        CREATE POLICY tenant_gateways ON gateways
        USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
        WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
        """
    )


def downgrade() -> None:
    _exec("DELETE FROM gateways WHERE domain_id IS NULL;")
    _exec("ALTER TABLE gateways ALTER COLUMN domain_id SET NOT NULL;")
    _exec("DROP POLICY IF EXISTS tenant_gateways ON gateways;")
    _exec(
        """
        CREATE POLICY tenant_gateways ON gateways
        USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)
        WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid);
        """
    )
