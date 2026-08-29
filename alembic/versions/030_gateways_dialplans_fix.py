"""gateways dialplans fix: perms + dialplan missing columns

Revision ID: 030_gateways_dialplans_fix
Revises: 029_vars_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "030_gateways_dialplans_fix"
down_revision: Union[str, None] = "029_vars_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # dialplans — add missing legacy columns (nullable)
    # dialplan_number text, dialplan_destination bool, dialplan_xml text
    for col, typ in [
        ("dialplan_number", sa.String(length=128)),
        ("dialplan_destination", sa.Boolean()),
        ("dialplan_xml", sa.Text()),
    ]:
        try:
            op.add_column("dialplans", sa.Column(col, typ, nullable=True))
        except Exception:
            pass

    # dialplan_details — add nullable parity columns
    # domain_id uuid FK, break/inline text, group numeric, detail_enabled bool (+ legacy alias)
    # Use nullable=True for all to avoid breaking existing rows
    try:
        op.add_column("dialplan_details", sa.Column("domain_id", sa.Uuid(), nullable=True))
    except Exception:
        pass
    # add FK constraint for domain_id if not exists (best-effort)
    try:
        op.create_foreign_key("fk_dialplan_details_domain_id", "dialplan_details", "domains", ["domain_id"], ["domain_id"], ondelete="CASCADE")
    except Exception:
        pass
    try:
        op.create_index("ix_dialplan_details_domain_id", "dialplan_details", ["domain_id"])
    except Exception:
        pass

    for col, typ in [
        ("dialplan_detail_break", sa.String(length=32)),
        ("dialplan_detail_inline", sa.String(length=32)),
        ("dialplan_detail_group", sa.Integer()),
        ("detail_enabled", sa.Boolean()),
    ]:
        try:
            op.add_column("dialplan_details", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    # defaults
    try:
        op.execute("UPDATE dialplan_details SET detail_enabled = true WHERE detail_enabled IS NULL")
    except Exception:
        pass
    try:
        op.execute("UPDATE dialplan_details SET dialplan_detail_group = 0 WHERE dialplan_detail_group IS NULL")
    except Exception:
        pass

    # ensure dialplans details ordering index helpful
    try:
        op.create_index("ix_dialplan_details_group_order", "dialplan_details", ["dialplan_detail_group", "order"])
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.drop_index("ix_dialplan_details_group_order", table_name="dialplan_details")
    except Exception:
        pass
    for col in ["detail_enabled", "dialplan_detail_group", "dialplan_detail_inline", "dialplan_detail_break"]:
        try:
            op.drop_column("dialplan_details", col)
        except Exception:
            pass
    try:
        op.drop_constraint("fk_dialplan_details_domain_id", "dialplan_details", type_="foreignkey")
    except Exception:
        pass
    try:
        op.drop_index("ix_dialplan_details_domain_id", table_name="dialplan_details")
    except Exception:
        pass
    try:
        op.drop_column("dialplan_details", "domain_id")
    except Exception:
        pass
    for col in ["dialplan_xml", "dialplan_destination", "dialplan_number"]:
        try:
            op.drop_column("dialplans", col)
        except Exception:
            pass
