"""contacts base fields fix + view

Revision ID: 040_contacts_fix
Revises: 039_voicemail_cdr_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "040_contacts_fix"
down_revision: Union[str, None] = "039_voicemail_cdr_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _add(table, column):
    try:
        op.add_column(table, column)
    except Exception:
        pass


def upgrade() -> None:
    cols = [
        sa.Column("contact_type", sa.String(length=32), nullable=True),
        sa.Column("contact_parent_uuid", sa.Uuid(), nullable=True),
        sa.Column("contact_name_prefix", sa.String(length=64), nullable=True),
        sa.Column("contact_name_given", sa.String(length=128), nullable=True),
        sa.Column("contact_name_middle", sa.String(length=128), nullable=True),
        sa.Column("contact_name_family", sa.String(length=128), nullable=True),
        sa.Column("contact_name_suffix", sa.String(length=64), nullable=True),
        sa.Column("contact_nickname", sa.String(length=128), nullable=True),
        sa.Column("contact_title", sa.String(length=128), nullable=True),
        sa.Column("contact_role", sa.String(length=128), nullable=True),
        sa.Column("contact_category", sa.String(length=64), nullable=True),
        sa.Column("contact_url", sa.String(length=512), nullable=True),
        sa.Column("contact_time_zone", sa.String(length=64), nullable=True),
        sa.Column("contact_note", sa.Text(), nullable=True),
    ]
    for c in cols:
        _add("contacts", c)
    # FK for parent (optional, try add constraint)
    try:
        op.create_foreign_key("fk_contacts_parent", "contacts", "contacts", ["contact_parent_uuid"], ["contact_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_contacts_parent", "contacts", ["contact_parent_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_contacts_type", "contacts", ["contact_type"])
    except Exception:
        pass
    # refresh legacy view v_contacts (domain_id -> domain_uuid)
    try:
        op.execute("DROP VIEW IF EXISTS v_contacts;")
        op.execute("""
            CREATE VIEW v_contacts AS
            SELECT
                domain_id AS domain_uuid,
                domain_id,
                contact_uuid,
                contact_parent_uuid,
                contact_type,
                organization AS contact_organization,
                contact_name_prefix,
                contact_name_given,
                contact_name_middle,
                contact_name_family,
                contact_name_suffix,
                contact_nickname,
                contact_title,
                contact_role,
                contact_category,
                contact_url,
                contact_time_zone,
                contact_note,
                name,
                number,
                email,
                organization,
                created_at AS insert_date
            FROM contacts;
        """)
    except Exception:
        # fallback: simple view if columns missing
        try:
            op.execute("DROP VIEW IF EXISTS v_contacts;")
            op.execute("CREATE VIEW v_contacts AS SELECT domain_id AS domain_uuid, * FROM contacts;")
        except Exception:
            pass
    # also ensure contacts view selects organization alias
    # legacy columns like last_mod etc not needed


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_contacts;")
    except Exception:
        pass
    try:
        op.drop_constraint("fk_contacts_parent", "contacts", type_="foreignkey")
    except Exception:
        pass
    try:
        op.drop_index("ix_contacts_parent", table_name="contacts")
    except Exception:
        pass
    try:
        op.drop_index("ix_contacts_type", table_name="contacts")
    except Exception:
        pass
    for col in ["contact_note","contact_time_zone","contact_url","contact_category","contact_role","contact_title","contact_nickname","contact_name_suffix","contact_name_family","contact_name_middle","contact_name_given","contact_name_prefix","contact_parent_uuid","contact_type"]:
        try:
            op.drop_column("contacts", col)
        except Exception:
            pass
