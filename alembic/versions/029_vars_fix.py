"""vars fix: var_command, var_hostname + access_control_nodes updated_at

Revision ID: 029_vars_fix
Revises: 028_gap6
Create Date: 2026-08-25
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "029_vars_fix"
down_revision: Union[str, None] = "028_gap6"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # vars: add missing columns for parity with FusionPBX legacy
    for col, typ in [
        ("var_command", sa.String(length=512)),
        ("var_hostname", sa.String(length=255)),
    ]:
        try:
            op.add_column("vars", sa.Column(col, typ, nullable=True))
        except Exception:
            pass

    # access_control_nodes: add updated_at column (was missing vs model)
    try:
        op.add_column("access_control_nodes", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    except Exception:
        pass
    try:
        op.execute("UPDATE access_control_nodes SET updated_at = created_at WHERE updated_at IS NULL")
    except Exception:
        pass
    # refresh view v_vars to include new columns if needed
    try:
        op.execute("DROP VIEW IF EXISTS v_vars;")
        op.execute("CREATE VIEW v_vars AS SELECT domain_id AS domain_uuid, var_uuid, var_category, var_name, var_value, var_enabled, var_description, var_command, var_hostname, created_at AS insert_date FROM vars;")
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_vars;")
        op.execute("CREATE VIEW v_vars AS SELECT domain_id AS domain_uuid, var_uuid, var_category, var_name, var_value, var_enabled, var_description, created_at AS insert_date FROM vars;")
    except Exception:
        pass
    for col in ["var_hostname", "var_command"]:
        try:
            op.drop_column("vars", col)
        except Exception:
            pass
    try:
        op.drop_column("access_control_nodes", "updated_at")
    except Exception:
        pass
