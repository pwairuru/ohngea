"""gap6 recordings S3 fax/email queue workers

Revision ID: 028_gap6
Revises: 027_call_forward
Create Date: 2026-08-23
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "028_gap6"
down_revision: Union[str, None] = "027_call_forward"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

def upgrade() -> None:
    # recordings S3 fields
    for col, typ in [
        ("s3_bucket", sa.String(length=128)),
        ("content_type", sa.String(length=128)),
        ("storage_backend", sa.String(length=32)),
    ]:
        try:
            op.add_column("recordings", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    # set default storage_backend to s3 where null
    try:
        op.execute("UPDATE recordings SET storage_backend='s3' WHERE storage_backend IS NULL")
    except Exception:
        pass
    # fax_queue worker helpers
    for col, typ in [
        ("fax_retry_count", sa.Integer()),
        ("fax_error", sa.String(length=512)),
    ]:
        try:
            op.add_column("fax_queue", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    try:
        op.execute("UPDATE fax_queue SET fax_retry_count=0 WHERE fax_retry_count IS NULL")
    except Exception:
        pass
    # email_queue helpers
    for col, typ in [
        ("email_error", sa.String(length=512)),
        ("email_attempts", sa.Integer()),
    ]:
        try:
            op.add_column("email_queue", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    try:
        op.execute("UPDATE email_queue SET email_attempts=0 WHERE email_attempts IS NULL")
    except Exception:
        pass

def downgrade() -> None:
    for col in ["storage_backend", "content_type", "s3_bucket"]:
        try:
            op.drop_column("recordings", col)
        except Exception:
            pass
    for col in ["fax_error", "fax_retry_count"]:
        try:
            op.drop_column("fax_queue", col)
        except Exception:
            pass
    for col in ["email_attempts", "email_error"]:
        try:
            op.drop_column("email_queue", col)
        except Exception:
            pass
