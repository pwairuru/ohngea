"""fix missing permissions + group_permissions for time_conditions, voicemails, conferences, ring_groups

Revision ID: 009_fix_permissions
Revises: 008_call_flows
Create Date: 2026-08-22
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "009_fix_permissions"
down_revision: Union[str, None] = "008_call_flows"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ensure all permissions exist
    op.execute(
        """
        INSERT INTO permissions (permission_uuid, name)
        VALUES
            (gen_random_uuid(), 'ring_group_view'),
            (gen_random_uuid(), 'ring_group_add'),
            (gen_random_uuid(), 'ring_group_edit'),
            (gen_random_uuid(), 'ring_group_delete'),
            (gen_random_uuid(), 'ring_group_forward'),
            (gen_random_uuid(), 'ring_group_missed_call'),
            (gen_random_uuid(), 'time_condition_view'),
            (gen_random_uuid(), 'time_condition_add'),
            (gen_random_uuid(), 'time_condition_edit'),
            (gen_random_uuid(), 'time_condition_delete'),
            (gen_random_uuid(), 'voicemail_view'),
            (gen_random_uuid(), 'voicemail_add'),
            (gen_random_uuid(), 'voicemail_edit'),
            (gen_random_uuid(), 'voicemail_delete'),
            (gen_random_uuid(), 'conference_view'),
            (gen_random_uuid(), 'conference_add'),
            (gen_random_uuid(), 'conference_edit'),
            (gen_random_uuid(), 'conference_delete'),
            (gen_random_uuid(), 'fax_view'),
            (gen_random_uuid(), 'fax_add'),
            (gen_random_uuid(), 'fax_edit'),
            (gen_random_uuid(), 'fax_delete'),
            (gen_random_uuid(), 'call_flow_view'),
            (gen_random_uuid(), 'call_flow_add'),
            (gen_random_uuid(), 'call_flow_edit'),
            (gen_random_uuid(), 'call_flow_delete')
        ON CONFLICT (name) DO NOTHING;
        """
    )
    # grant all to superadmin (idempotent via NOT EXISTS)
    op.execute(
        """
        INSERT INTO group_permissions (id, group_uuid, permission_uuid)
        SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
        FROM groups g, permissions p
        WHERE g.name='superadmin'
          AND p.name IN (
            'ring_group_view','ring_group_add','ring_group_edit','ring_group_delete','ring_group_forward','ring_group_missed_call',
            'time_condition_view','time_condition_add','time_condition_edit','time_condition_delete',
            'voicemail_view','voicemail_add','voicemail_edit','voicemail_delete',
            'conference_view','conference_add','conference_edit','conference_delete',
            'fax_view','fax_add','fax_edit','fax_delete',
            'call_flow_view','call_flow_add','call_flow_edit','call_flow_delete'
          )
          AND NOT EXISTS (
            SELECT 1 FROM group_permissions gp2
            WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid
          );
        """
    )


def downgrade() -> None:
    pass
