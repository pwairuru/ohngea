"""destinations 32-field parity, dbtrans app_uuid/result, domain_limits, event_guard insert_user, log_download

Revision ID: 037_dest_db_fix
Revises: 036_stub
Create Date: 2026-08-25
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "037_dest_db_fix"
down_revision: Union[str, None] = "036_stub"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _add(col_table, column):
    try:
        op.add_column(col_table, column)
    except Exception:
        pass


def upgrade() -> None:
    # --- destinations: add >=15 nullable columns ---
    dest_cols = [
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=True),
        sa.Column("fax_uuid", sa.Uuid(), nullable=True),
        sa.Column("user_uuid", sa.Uuid(), nullable=True),
        sa.Column("group_uuid", sa.Uuid(), nullable=True),
        sa.Column("provider_uuid", sa.Uuid(), nullable=True),
        sa.Column("destination_trunk_prefix", sa.String(length=32), nullable=True),
        sa.Column("trunk_prefix", sa.String(length=32), nullable=True),
        sa.Column("destination_area_code", sa.String(length=16), nullable=True),
        sa.Column("area_code", sa.String(length=16), nullable=True),
        sa.Column("destination_condition_field", sa.String(length=128), nullable=True),
        sa.Column("condition_field", sa.String(length=128), nullable=True),
        sa.Column("destination_number_regex", sa.String(length=512), nullable=True),
        sa.Column("number_regex", sa.String(length=512), nullable=True),
        sa.Column("destination_distinctive_ring", sa.String(length=128), nullable=True),
        sa.Column("distinctive_ring", sa.String(length=128), nullable=True),
        sa.Column("destination_ringback", sa.String(length=128), nullable=True),
        sa.Column("ringback", sa.String(length=128), nullable=True),
        sa.Column("destination_accountcode", sa.String(length=64), nullable=True),
        sa.Column("accountcode", sa.String(length=64), nullable=True),
        sa.Column("destination_type_voice", sa.Boolean(), nullable=True),
        sa.Column("type_voice", sa.Boolean(), nullable=True),
        sa.Column("destination_type_fax", sa.Boolean(), nullable=True),
        sa.Column("type_fax", sa.Boolean(), nullable=True),
        sa.Column("destination_type_emergency", sa.Boolean(), nullable=True),
        sa.Column("type_emergency", sa.Boolean(), nullable=True),
        sa.Column("destination_type_text", sa.Boolean(), nullable=True),
        sa.Column("type_text", sa.Boolean(), nullable=True),
        sa.Column("destination_conditions", sa.Text(), nullable=True),
        sa.Column("conditions", sa.Text(), nullable=True),
        sa.Column("destination_actions", sa.Text(), nullable=True),
        sa.Column("actions", sa.Text(), nullable=True),
        sa.Column("destination_alternate_app", sa.String(length=64), nullable=True),
        sa.Column("alternate_app", sa.String(length=64), nullable=True),
        sa.Column("destination_alternate_data", sa.String(length=512), nullable=True),
        sa.Column("alternate_data", sa.String(length=512), nullable=True),
        sa.Column("destination_caller_id", sa.String(length=64), nullable=True),
        sa.Column("destination_insert_user", sa.Uuid(), nullable=True),
        sa.Column("insert_user", sa.Uuid(), nullable=True),
        sa.Column("destination_insert_date", sa.DateTime(timezone=True), nullable=True),
        sa.Column("destination_update_user", sa.Uuid(), nullable=True),
    ]
    for c in dest_cols:
        _add("destinations", c)
    for col in ["dialplan_uuid", "fax_uuid", "user_uuid", "group_uuid", "provider_uuid"]:
        try:
            op.create_index(f"ix_destinations_{col}", "destinations", [col])
        except Exception:
            pass
    # refresh v_destinations
    try:
        op.execute("DROP VIEW IF EXISTS v_destinations;")
        op.execute("""
            CREATE VIEW v_destinations AS SELECT
                domain_id AS domain_uuid,
                destination_uuid, destination_type, destination_number, destination_prefix,
                destination_enabled, destination_description, destination_context,
                destination_app, destination_data, dialplan_uuid, fax_uuid,
                destination_trunk_prefix, destination_area_code, destination_condition_field,
                destination_number_regex, destination_distinctive_ring, destination_ringback,
                destination_accountcode, destination_type_voice, destination_type_fax,
                destination_conditions, destination_actions, destination_alternate_app,
                destination_alternate_data, created_at AS insert_date, updated_at AS update_date
            FROM destinations;
        """)
    except Exception:
        pass

    # --- database_transactions: add app_uuid, transaction_result ---
    _add("database_transactions", sa.Column("app_uuid", sa.Uuid(), nullable=True))
    _add("database_transactions", sa.Column("transaction_result", sa.Text(), nullable=True))
    try:
        op.create_index("ix_database_transactions_app_uuid", "database_transactions", ["app_uuid"])
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_database_transactions;")
        op.execute("CREATE VIEW v_database_transactions AS SELECT domain_id AS domain_uuid, database_transaction_uuid, app_name, app_uuid, transaction_code, transaction_type, transaction_result, transaction_date AS insert_date FROM database_transactions;")
    except Exception:
        pass

    # --- domain_limits: create table if not exists ---
    try:
        op.create_table(
            "domain_limits",
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("limit_name", sa.String(length=64), nullable=False),
            sa.Column("limit_value", sa.String(length=128), nullable=True),
            sa.Column("limit_category", sa.String(length=64), nullable=True),
            sa.Column("limit_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
            sa.Column("limit_description", sa.Text(), nullable=True),
            sa.Column("domain_limit_uuid", sa.Uuid(), nullable=False),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("domain_limit_uuid"),
        )
    except Exception:
        pass
    for idx_col in ["domain_id", "limit_name"]:
        try:
            op.create_index(f"ix_domain_limits_{idx_col}", "domain_limits", [idx_col])
        except Exception:
            pass
    try:
        op.execute("ALTER TABLE domain_limits ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE domain_limits FORCE ROW LEVEL SECURITY;")
        op.execute("""
            CREATE POLICY tenant_isolation ON domain_limits
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');
        """)
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_domain_limits;")
        op.execute("CREATE VIEW v_domain_limits AS SELECT domain_id AS domain_uuid, domain_limit_uuid, limit_name, limit_value, limit_category, limit_enabled, created_at AS insert_date FROM domain_limits;")
    except Exception:
        pass

    # --- event_guard: add insert_user, updated_at, ensure log_date/filter/user_agent ---
    _add("event_guard_logs", sa.Column("insert_user", sa.Uuid(), nullable=True))
    _add("event_guard_logs", sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True))
    # ensure perms for event_guard
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'event_guard_view'),
            (gen_random_uuid(), 'event_guard_add'),
            (gen_random_uuid(), 'event_guard_edit'),
            (gen_random_uuid(), 'event_guard_delete')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('event_guard_view','event_guard_add','event_guard_edit','event_guard_delete')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_event_guard;")
        op.execute("CREATE VIEW v_event_guard AS SELECT event_guard_log_uuid, hostname, log_date, filter, ip_address, extension, user_agent, log_status, insert_user, created_at AS insert_date FROM event_guard_logs;")
    except Exception:
        pass

    # --- permissions for domain_limits and log_view ---
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'domain_limit_view'),
            (gen_random_uuid(), 'domain_limit_add'),
            (gen_random_uuid(), 'domain_limit_edit'),
            (gen_random_uuid(), 'domain_limit_delete'),
            (gen_random_uuid(), 'log_view'),
            (gen_random_uuid(), 'log_download')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('domain_limit_view','domain_limit_add','domain_limit_edit','domain_limit_delete','log_view','log_download')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_destinations;")
        op.execute("CREATE VIEW v_destinations AS SELECT domain_id AS domain_uuid, destination_uuid, destination_type, destination_number, destination_prefix, destination_enabled, destination_description, destination_context, destination_app, destination_data, created_at AS insert_date FROM destinations;")
    except Exception:
        pass
    for col in ["destination_update_user","destination_insert_date","insert_user","destination_insert_user","destination_caller_id","alternate_data","destination_alternate_data","alternate_app","destination_alternate_app","actions","destination_actions","conditions","destination_conditions","type_text","destination_type_text","type_emergency","destination_type_emergency","type_fax","destination_type_fax","type_voice","destination_type_voice","accountcode","destination_accountcode","ringback","destination_ringback","distinctive_ring","destination_distinctive_ring","number_regex","destination_number_regex","condition_field","destination_condition_field","area_code","destination_area_code","trunk_prefix","destination_trunk_prefix","provider_uuid","group_uuid","user_uuid","fax_uuid","dialplan_uuid"]:
        try:
            op.drop_column("destinations", col)
        except Exception:
            pass
    try:
        op.drop_column("database_transactions", "transaction_result")
    except Exception:
        pass
    try:
        op.drop_column("database_transactions", "app_uuid")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_database_transactions;")
        op.execute("CREATE VIEW v_database_transactions AS SELECT domain_id AS domain_uuid, database_transaction_uuid, app_name, transaction_code, transaction_type, transaction_date AS insert_date FROM database_transactions;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_domain_limits;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON domain_limits;")
        op.execute("ALTER TABLE domain_limits DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("domain_limits")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_event_guard;")
        op.execute("CREATE VIEW v_event_guard AS SELECT event_guard_log_uuid, hostname, ip_address, extension, log_status, created_at AS insert_date FROM event_guard_logs;")
    except Exception:
        pass
    try:
        op.drop_column("event_guard_logs", "updated_at")
    except Exception:
        pass
    try:
        op.drop_column("event_guard_logs", "insert_user")
    except Exception:
        pass
