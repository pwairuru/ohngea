"""recordings partitioned + contacts

Revision ID: 002_recordings
Revises: 001_initial
Create Date: 2026-08-21
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "002_recordings"
down_revision: Union[str, None] = "001_initial"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "contacts",
        sa.Column("domain_id", sa.Uuid(), nullable=False),
        sa.Column("name", sa.String(length=128), nullable=False),
        sa.Column("number", sa.String(length=32), nullable=False),
        sa.Column("email", sa.String(length=255), nullable=True),
        sa.Column("organization", sa.String(length=128), nullable=True),
        sa.Column("contact_uuid", sa.Uuid(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("contact_uuid"),
    )
    op.create_index("ix_contacts_domain_id", "contacts", ["domain_id"])
    op.create_index("ix_contacts_number", "contacts", ["number"])

    # recordings partitioned by HASH(domain_id) 32 like cdr
    op.execute(
        """
        CREATE TABLE recordings (
            recording_uuid uuid NOT NULL DEFAULT gen_random_uuid(),
            domain_id uuid NOT NULL,
            name text NOT NULL,
            file_path text NOT NULL,
            duration integer,
            file_size integer,
            created_by uuid,
            created_at timestamptz NOT NULL DEFAULT now(),
            PRIMARY KEY (recording_uuid, domain_id),
            FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE
        ) PARTITION BY HASH (domain_id);
        """
    )
    for i in range(32):
        op.execute(f"CREATE TABLE recordings_p{i} PARTITION OF recordings FOR VALUES WITH (MODULUS 32, REMAINDER {i});")
        op.execute(f"CREATE INDEX ON recordings_p{i} (domain_id, created_at DESC);")

    for tbl in ["contacts", "recordings"]:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(
            f"""
            CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
            """
        )


def downgrade() -> None:
    op.execute("DROP TABLE IF EXISTS recordings CASCADE;")
    op.execute("DROP TABLE IF EXISTS contacts CASCADE;")
