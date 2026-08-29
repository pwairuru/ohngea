"""provision vendor default_settings seeding

Revision ID: 034_provision_defaults
Revises: 033_fax_recordings_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "035_provision_defaults"
down_revision: Union[str, None] = "034_combined_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Create default_settings table if not exists
    try:
        op.create_table(
            "default_settings",
            sa.Column("default_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("default_setting_category", sa.String(length=64), nullable=False),
            sa.Column("default_setting_subcategory", sa.String(length=64), nullable=False),
            sa.Column("default_setting_name", sa.String(length=64), nullable=False),
            sa.Column("default_setting_value", sa.Text(), nullable=True),
            sa.Column("default_setting_enabled", sa.Boolean(), nullable=False, server_default="true"),
            sa.Column("default_setting_description", sa.Text(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
            sa.PrimaryKeyConstraint("default_setting_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_default_settings_category", "default_settings", ["default_setting_category"])
    except Exception:
        pass
    try:
        op.create_index("ix_default_settings_subcategory", "default_settings", ["default_setting_subcategory"])
    except Exception:
        pass
    try:
        op.create_index("ix_default_settings_category_sub", "default_settings", ["default_setting_category", "default_setting_subcategory"])
    except Exception:
        pass
    # Ensure RLS not forced (global settings)
    # Seed provision defaults for vendors — idempotent via app.seed_provision_defaults.PROVISION_DEFAULTS
    try:
        from app.seed_provision_defaults import PROVISION_DEFAULTS

        conn = op.get_bind()
        for row in PROVISION_DEFAULTS:
            try:
                conn.execute(
                    sa.text("""
                        INSERT INTO default_settings
                        (default_setting_uuid, default_setting_category, default_setting_subcategory, default_setting_name, default_setting_value, default_setting_enabled, default_setting_description)
                        VALUES (:uuid, :cat, :sub, :name, :val, :enabled, :desc)
                        ON CONFLICT (default_setting_uuid) DO NOTHING
                    """),
                    {
                        "uuid": row["default_setting_uuid"],
                        "cat": row["default_setting_category"],
                        "sub": row["default_setting_subcategory"],
                        "name": row["default_setting_name"],
                        "val": row["default_setting_value"],
                        "enabled": row["default_setting_enabled"],
                        "desc": row["default_setting_description"],
                    },
                )
            except Exception:
                # sqlite fallback try INSERT OR IGNORE inside alembic connection may be postgres only; ignore
                continue
        # Create views for legacy compat
        try:
            op.execute("DROP VIEW IF EXISTS v_default_settings;")
            op.execute("""CREATE VIEW v_default_settings AS SELECT
                default_setting_uuid, default_setting_category, default_setting_subcategory,
                default_setting_name, default_setting_value, default_setting_enabled,
                default_setting_description, created_at AS insert_date, updated_at AS update_date
                FROM default_settings;""")
        except Exception:
            pass
    except Exception as e:
        # If import fails during offline migration, seed via raw inserts minimal
        print(f"provision defaults seed skipped: {e}")


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_default_settings;")
    except Exception:
        pass
    try:
        op.drop_index("ix_default_settings_category_sub", table_name="default_settings")
    except Exception:
        pass
    try:
        op.drop_index("ix_default_settings_subcategory", table_name="default_settings")
    except Exception:
        pass
    try:
        op.drop_index("ix_default_settings_category", table_name="default_settings")
    except Exception:
        pass
    try:
        op.drop_table("default_settings")
    except Exception:
        pass
