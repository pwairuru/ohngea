"""logs + domain_limits perms

Revision ID: 018_logs_perm
Revises: 017_dbtrans
Create Date: 2026-08-22
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
revision: str = "018_logs_perm"
down_revision: Union[str, None] = "017_dbtrans"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    op.execute("INSERT INTO permissions (permission_uuid, name) VALUES (gen_random_uuid(), 'log_view'), (gen_random_uuid(), 'log_download'), (gen_random_uuid(), 'domain_limit_view') ON CONFLICT (name) DO NOTHING;")
    op.execute("INSERT INTO group_permissions (id, group_uuid, permission_uuid) SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid FROM groups g, permissions p WHERE g.name='superadmin' AND p.name IN ('log_view','log_download','domain_limit_view') AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);")
    op.execute("DROP VIEW IF EXISTS v_logs;")
    op.execute("CREATE VIEW v_logs AS SELECT '00000000-0000-0000-0000-000000000000'::uuid AS domain_uuid, gen_random_uuid() AS log_uuid, now() AS insert_date;")
    op.execute("DROP VIEW IF EXISTS v_domain_limits;")
    op.execute("CREATE VIEW v_domain_limits AS SELECT domain_id AS domain_uuid, domain_id, 'limits' AS name FROM domains;")

def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_domain_limits; DROP VIEW IF EXISTS v_logs;")
    op.execute("DELETE FROM group_permissions WHERE permission_uuid IN (SELECT permission_uuid FROM permissions WHERE name IN ('log_view','log_download','domain_limit_view'));")
    op.execute("DELETE FROM permissions WHERE name IN ('log_view','log_download','domain_limit_view');")
