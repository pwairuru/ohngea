"""stub bridge to satisfy 037 revises 036

Revision ID: 036_stub
Revises: 035_provision_defaults
Create Date: 2026-08-25
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "036_stub"
down_revision: Union[str, None] = "036_call_batch_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    pass


def downgrade() -> None:
    pass
