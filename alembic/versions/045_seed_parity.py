"""seed parity globals out of box (default_settings full + sofia + permissions).

Revision ID: 045_seed_parity
Revises: 044_sip_profile_ext_ips
Create Date: 2026-09-30
Fresh `alembic upgrade head` (Docker CMD) seeds working-PBX globals.
Idempotent via ON CONFLICT DO NOTHING. Per-domain tables seed via create_domain().
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "045_seed_parity"
down_revision: Union[str, None] = "044_sip_profile_ext_ips"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _lit(v):
    if v is None:
        return "NULL"
    if isinstance(v, bool):
        return "true" if v else "false"
    return "'" + str(v).replace("'", "''").replace(":", "\\:") + "'"


def upgrade() -> None:
    try:
        op.create_table(
            "sofia_global_settings",
            sa.Column("sofia_global_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("global_setting_name", sa.String(length=128), nullable=False),
            sa.Column("global_setting_value", sa.String(length=512), nullable=False),
            sa.Column("global_setting_enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("global_setting_description", sa.Text(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
            sa.PrimaryKeyConstraint("sofia_global_setting_uuid"),
        )
    except Exception:
        pass
    try:
        from app.seeds.default_settings import DEFAULT_SETTINGS

        for row in DEFAULT_SETTINGS:
            try:
                op.execute(
                    sa.text(f"""
                        INSERT INTO default_settings
                        (default_setting_uuid, default_setting_category, default_setting_subcategory,
                         default_setting_name, default_setting_value, default_setting_enabled, default_setting_description)
                        VALUES ({_lit(row["uuid"])}, {_lit(row["category"])}, {_lit(row["subcategory"])},
                                {_lit(row["name"])}, {_lit(row["value"])}, {_lit(row["enabled"])}, {_lit(row["description"])})
                        ON CONFLICT (default_setting_uuid) DO NOTHING
                    """)
                )
            except Exception:
                continue
    except Exception:
        pass
    try:
        from app.seeds.sofia import SOFIA_GLOBAL_SETTINGS

        for row in SOFIA_GLOBAL_SETTINGS:
            try:
                op.execute(
                    sa.text(f"""
                        INSERT INTO sofia_global_settings
                        (sofia_global_setting_uuid, global_setting_name, global_setting_value,
                         global_setting_enabled, global_setting_description)
                        VALUES ({_lit(row["uuid"])}, {_lit(row["name"])}, {_lit(row["value"])},
                                {_lit(row["enabled"])}, {_lit(row["description"])})
                        ON CONFLICT (sofia_global_setting_uuid) DO NOTHING
                    """)
                )
            except Exception:
                continue
    except Exception:
        pass
    try:
        from app.seeds.permissions import ALL_PERMISSIONS

        for name in ALL_PERMISSIONS:
            try:
                op.execute(
                    sa.text(f"""
                        INSERT INTO permissions (permission_uuid, name)
                        VALUES (gen_random_uuid(), {_lit(name)})
                        ON CONFLICT (name) DO NOTHING
                    """)
                )
            except Exception:
                continue
    except Exception:
        pass


def downgrade() -> None:
    pass
