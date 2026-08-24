"""conferences + conference_users with RLS and indexes

Revision ID: 006_conferences
Revises: 005_voicemails
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

# revision identifiers, used by Alembic.
revision: str = "006_conferences"
down_revision: Union[str, None] = "005_voicemails"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "conferences",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("conference_name", sa.String(length=128), nullable=False),
        sa.Column("conference_extension", sa.String(length=32), nullable=False),
        sa.Column("conference_pin_number", sa.String(length=32), nullable=True),
        sa.Column("conference_profile", sa.String(length=64), nullable=False),
        sa.Column("conference_enabled", sa.Boolean(), nullable=False),
        sa.Column("conference_description", sa.Text(), nullable=True),
        sa.Column("conference_flags", sa.String(length=255), nullable=True),
        sa.Column("conference_order", sa.Integer(), nullable=True),
        sa.Column("conference_context", sa.String(length=128), nullable=True),
        sa.Column("conference_email_address", sa.String(length=512), nullable=True),
        sa.Column("conference_account_code", sa.String(length=64), nullable=True),
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=True),
        sa.Column("conference_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["dialplan_uuid"], ["dialplans.dialplan_uuid"], ondelete="SET NULL"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("conference_uuid"),
    )
    op.create_index("ix_conferences_domain_id", "conferences", ["domain_id"])
    op.create_index("ix_conferences_domain_extension", "conferences", ["domain_id", "conference_extension"], unique=True)
    op.create_index("ix_conferences_name", "conferences", ["conference_name"])
    op.create_index("ix_conferences_extension", "conferences", ["conference_extension"])
    op.create_index("ix_conferences_profile", "conferences", ["conference_profile"])
    op.create_index("ix_conferences_enabled", "conferences", ["conference_enabled"])

    op.create_table(
        "conference_users",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("conference_uuid", sa.Uuid(), nullable=False),
        sa.Column("user_uuid", sa.Uuid(), nullable=False),
        sa.Column("conference_user_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["conference_uuid"], ["conferences.conference_uuid"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("conference_user_uuid"),
    )
    op.create_index("ix_conference_users_domain_id", "conference_users", ["domain_id"])
    op.create_index("ix_conference_users_conference_uuid", "conference_users", ["conference_uuid"])
    op.create_index("ix_conference_users_user_uuid", "conference_users", ["user_uuid"])

    # legacy views for Ohngea PBX compatibility
    op.execute("""
        CREATE VIEW v_conferences AS
        SELECT
            domain_id AS domain_uuid,
            conference_uuid,
            dialplan_uuid,
            conference_name,
            conference_extension,
            conference_pin_number,
            conference_profile,
            conference_flags,
            conference_order,
            conference_description,
            conference_context,
            conference_email_address,
            conference_account_code,
            conference_enabled,
            created_at AS insert_date,
            updated_at AS update_date
        FROM conferences;
    """)
    op.execute("""
        CREATE VIEW v_conference_users AS
        SELECT
            domain_id AS domain_uuid,
            conference_user_uuid,
            conference_uuid,
            user_uuid,
            created_at AS insert_date
        FROM conference_users;
    """)

    # RLS
    for tbl in ["conferences", "conference_users"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(
            f"""
            CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
            """
        )

    # seed permissions
    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name)
        VALUES
            (gen_random_uuid(), 'conference_view'),
            (gen_random_uuid(), 'conference_add'),
            (gen_random_uuid(), 'conference_edit'),
            (gen_random_uuid(), 'conference_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )


def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_conference_users;")
    op.execute("DROP VIEW IF EXISTS v_conferences;")
    op.execute("DROP TABLE IF EXISTS conference_users CASCADE;")
    op.execute("DROP TABLE IF EXISTS conferences CASCADE;")
