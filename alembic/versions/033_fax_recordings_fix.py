"""fax and recordings parity fix

Revision ID: 033_fax_recordings_fix
Revises: 032_devices_sip_parity
Create Date: 2026-08-25
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "033_fax_recordings_fix"
down_revision: Union[str, None] = "032_devices_sip_parity"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # fax -- add nullable columns for legacy parity
    for col, typ in [
        ("dialplan_uuid", sa.Uuid()),
        ("fax_destination_number", sa.String(length=32)),
        ("fax_prefix", sa.String(length=32)),
        ("fax_email_connection_type", sa.String(length=32)),
        ("fax_email_connection_host", sa.String(length=255)),
        ("fax_email_connection_port", sa.String(length=32)),
        ("fax_email_connection_security", sa.String(length=32)),
        ("fax_email_connection_validate", sa.Boolean()),
        ("fax_email_connection_username", sa.String(length=255)),
        ("fax_email_connection_password", sa.String(length=255)),
        ("fax_email_connection_mailbox", sa.String(length=128)),
        ("fax_email_inbound_subject_tag", sa.String(length=128)),
        ("fax_email_outbound_subject_tag", sa.String(length=128)),
        ("fax_email_outbound_authorized_senders", sa.String(length=512)),
        ("fax_send_channels", sa.Integer()),
        ("accountcode", sa.String(length=128)),
    ]:
        try:
            op.add_column("fax", sa.Column(col, typ, nullable=True))
        except Exception:
            pass

    try:
        op.create_foreign_key("fk_fax_dialplan", "fax", "dialplans", ["dialplan_uuid"], ["dialplan_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_fax_dialplan_uuid", "fax", ["dialplan_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_fax_destination_number", "fax", ["fax_destination_number"])
    except Exception:
        pass

    # fax_users table (mapping fax -> user)
    try:
        op.create_table(
            "fax_users",
            sa.Column("fax_user_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("fax_uuid", sa.Uuid(), nullable=False),
            sa.Column("user_uuid", sa.Uuid(), nullable=False),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["fax_uuid"], ["fax.fax_uuid"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("fax_user_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_fax_users_domain_id", "fax_users", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_fax_users_fax_uuid", "fax_users", ["fax_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_fax_users_user_uuid", "fax_users", ["user_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE fax_users ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE fax_users FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON fax_users
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # fax_files table
    try:
        op.create_table(
            "fax_files",
            sa.Column("fax_file_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("fax_uuid", sa.Uuid(), nullable=False),
            sa.Column("file_name", sa.String(length=255), nullable=True),
            sa.Column("file_path", sa.String(length=512), nullable=True),
            sa.Column("file_size", sa.Integer(), nullable=True),
            sa.Column("fax_file_path", sa.String(length=512), nullable=True),
            sa.Column("fax_file_type", sa.String(length=32), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["fax_uuid"], ["fax.fax_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("fax_file_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_fax_files_domain_id", "fax_files", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_fax_files_fax_uuid", "fax_files", ["fax_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE fax_files ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE fax_files FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON fax_files
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # fax_logs table
    try:
        op.create_table(
            "fax_logs",
            sa.Column("fax_log_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("fax_uuid", sa.Uuid(), nullable=False),
            sa.Column("fax_success", sa.String(length=32), nullable=True),
            sa.Column("result_code", sa.Integer(), nullable=True),
            sa.Column("fax_result_text", sa.String(length=512), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["fax_uuid"], ["fax.fax_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("fax_log_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_fax_logs_domain_id", "fax_logs", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_fax_logs_fax_uuid", "fax_logs", ["fax_uuid"])
    except Exception:
        pass
    try:
        op.execute("ALTER TABLE fax_logs ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE fax_logs FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON fax_logs
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

    # legacy compat views v_fax, v_fax_users, v_fax_files, v_fax_logs
    try:
        op.execute("DROP VIEW IF EXISTS v_fax;")
        op.execute(
            """CREATE VIEW v_fax AS SELECT
            domain_id AS domain_uuid, fax_uuid, dialplan_uuid, fax_extension, fax_destination_number, fax_prefix,
            fax_name, fax_email, fax_pin_number, fax_caller_id_name, fax_caller_id_number, fax_forward_number,
            fax_toll_allow, fax_send_channels, accountcode,
            fax_email_connection_type, fax_email_connection_host, fax_email_connection_port,
            fax_email_connection_security, fax_email_connection_validate, fax_email_connection_username,
            fax_email_connection_password, fax_email_connection_mailbox,
            fax_email_inbound_subject_tag, fax_email_outbound_subject_tag, fax_email_outbound_authorized_senders,
            enabled, description AS fax_description,
            created_at AS insert_date, updated_at AS update_date FROM fax;"""
        )
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_users;")
        op.execute("CREATE VIEW v_fax_users AS SELECT domain_id AS domain_uuid, fax_user_uuid, fax_uuid, user_uuid, created_at AS insert_date FROM fax_users;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_files;")
        op.execute("CREATE VIEW v_fax_files AS SELECT domain_id AS domain_uuid, fax_file_uuid, fax_uuid, file_name AS fax_file_name, file_path AS fax_file_path, file_size AS fax_file_size, created_at AS insert_date FROM fax_files;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_logs;")
        op.execute("CREATE VIEW v_fax_logs AS SELECT domain_id AS domain_uuid, fax_log_uuid, fax_uuid, fax_success, result_code AS fax_result_code, fax_result_text, created_at AS fax_date FROM fax_logs;")
    except Exception:
        pass

    # recordings -- add legacy parity columns nullable
    for col, typ in [
        ("recording_voice", sa.String(length=64)),
        ("recording_speed", sa.Numeric(precision=4, scale=2)),
        ("recording_message", sa.Text()),
        ("recording_base64", sa.Text()),
    ]:
        try:
            op.add_column("recordings", sa.Column(col, typ, nullable=True))
        except Exception:
            pass

    # seed recordings perms
    try:
        op.execute(
            """
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'recording_view'),
            (gen_random_uuid(), 'recording_add'),
            (gen_random_uuid(), 'recording_edit'),
            (gen_random_uuid(), 'recording_delete')
            ON CONFLICT (name) DO NOTHING;
            """
        )
        op.execute(
            """
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('recording_view','recording_add','recording_edit','recording_delete')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
            """
        )
    except Exception:
        pass
    try:
        op.execute(
            """
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='admin' AND p.name IN ('recording_view','recording_add','recording_edit','recording_delete')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
            """
        )
    except Exception:
        pass

    # refresh v_recordings view for legacy compat
    try:
        op.execute("DROP VIEW IF EXISTS v_recordings;")
        op.execute(
            """CREATE VIEW v_recordings AS SELECT
            domain_id AS domain_uuid, recording_uuid, name AS recording_name, file_path AS recording_filename,
            recording_voice, recording_speed, recording_message, recording_base64,
            file_size AS recording_file_size, created_at AS insert_date FROM recordings;"""
        )
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_recordings;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_logs;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_files;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_users;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax;")
    except Exception:
        pass
    for col in ["recording_base64", "recording_message", "recording_speed", "recording_voice"]:
        try:
            op.drop_column("recordings", col)
        except Exception:
            pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON fax_logs;")
        op.execute("ALTER TABLE fax_logs DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("fax_logs")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON fax_files;")
        op.execute("ALTER TABLE fax_files DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("fax_files")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON fax_users;")
        op.execute("ALTER TABLE fax_users DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("fax_users")
    except Exception:
        pass
    try:
        op.drop_constraint("fk_fax_dialplan", "fax", type_="foreignkey")
    except Exception:
        pass
    for col in ["accountcode", "fax_send_channels", "fax_email_outbound_authorized_senders", "fax_email_outbound_subject_tag", "fax_email_inbound_subject_tag", "fax_email_connection_mailbox", "fax_email_connection_password", "fax_email_connection_username", "fax_email_connection_validate", "fax_email_connection_security", "fax_email_connection_port", "fax_email_connection_host", "fax_email_connection_type", "fax_prefix", "fax_destination_number", "dialplan_uuid"]:
        try:
            op.drop_column("fax", col)
        except Exception:
            pass
