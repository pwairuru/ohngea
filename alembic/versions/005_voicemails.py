"""voicemails + voicemail_destinations with RLS and indexes

Revision ID: 005_voicemails
Revises: 004_time_conditions
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

# revision identifiers, used by Alembic.
revision: str = "005_voicemails"
down_revision: Union[str, None] = "004_time_conditions"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "voicemails",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("voicemail_id", sa.String(length=32), nullable=False),
        sa.Column("voicemail_password", sa.String(length=32), nullable=False),
        sa.Column("enabled", sa.Boolean(), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column("greeting_id", sa.Integer(), nullable=True),
        sa.Column("toll_allow", sa.String(length=64), nullable=True),
        sa.Column("voicemail_mail_to", sa.String(length=512), nullable=True),
        sa.Column("voicemail_file", sa.String(length=32), nullable=True),
        sa.Column("alternate_greet_id", sa.String(length=64), nullable=True),
        sa.Column("voicemail_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("voicemail_uuid"),
    )
    op.create_index("ix_voicemails_domain_id", "voicemails", ["domain_id"])
    op.create_index("ix_voicemails_domain_voicemail_id", "voicemails", ["domain_id", "voicemail_id"], unique=True)
    op.create_index("ix_voicemails_voicemail_id", "voicemails", ["voicemail_id"])
    op.create_index("ix_voicemails_enabled", "voicemails", ["enabled"])
    op.create_index("ix_voicemails_greeting_id", "voicemails", ["greeting_id"])

    op.create_table(
        "voicemail_destinations",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("voicemail_uuid", sa.Uuid(), nullable=False),
        sa.Column("voicemail_uuid_copy", sa.Uuid(), nullable=False),
        sa.Column("voicemail_destination_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["voicemail_uuid"], ["voicemails.voicemail_uuid"], ondelete="CASCADE"),
        sa.ForeignKeyConstraint(["voicemail_uuid_copy"], ["voicemails.voicemail_uuid"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("voicemail_destination_uuid"),
    )
    op.create_index("ix_voicemail_destinations_domain_id", "voicemail_destinations", ["domain_id"])
    op.create_index("ix_voicemail_destinations_voicemail_uuid", "voicemail_destinations", ["voicemail_uuid"])
    op.create_index("ix_voicemail_destinations_copy_uuid", "voicemail_destinations", ["voicemail_uuid_copy"])

    # legacy view aliases for Ohngea PBX (originally v_voicemails)
    op.execute("""
        CREATE VIEW v_voicemails AS
        SELECT
            domain_id AS domain_uuid,
            voicemail_uuid,
            voicemail_id,
            voicemail_password,
            greeting_id,
            toll_allow,
            voicemail_mail_to,
            voicemail_file,
            enabled AS voicemail_enabled,
            description AS voicemail_description,
            alternate_greet_id AS voicemail_alternate_greet_id,
            created_at AS insert_date,
            updated_at AS update_date
        FROM voicemails;
    """)
    op.execute("""
        CREATE VIEW v_voicemail_destinations AS
        SELECT
            domain_id AS domain_uuid,
            voicemail_destination_uuid,
            voicemail_uuid,
            voicemail_uuid_copy,
            created_at AS insert_date
        FROM voicemail_destinations;
    """)

    # RLS
    for tbl in ["voicemails", "voicemail_destinations"]:
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
            (gen_random_uuid(), 'voicemail_view'),
            (gen_random_uuid(), 'voicemail_add'),
            (gen_random_uuid(), 'voicemail_edit'),
            (gen_random_uuid(), 'voicemail_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )


def downgrade() -> None:
    op.execute("DROP VIEW IF EXISTS v_voicemail_destinations;")
    op.execute("DROP VIEW IF EXISTS v_voicemails;")
    op.execute("DROP TABLE IF EXISTS voicemail_destinations CASCADE;")
    op.execute("DROP TABLE IF EXISTS voicemails CASCADE;")
