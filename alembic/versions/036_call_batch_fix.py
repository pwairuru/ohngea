"""call batch parity fix 036

Revision ID: 036_call_batch_fix
Revises: 035_provision_defaults
Create Date: 2026-08-25

- call_block: add missing 9 fields (direction, extension_uuid, country_code, app, data, date_added, insert_user, update_user, etc.)
- call_broadcast: add 7 fields (start_time, recording_uuid, avmd, destination_data, accountcode, toll_allow)
- call_centers: add ~20 queue columns + expand agents + create call_center_tiers
- call_flows: add status, pin_number, label, app/data, alternate_sound/app/data, dialplan_uuid
- recordings: add transcription, summary; create call_recordings table
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "036_call_batch_fix"
down_revision: Union[str, None] = "035_provision_defaults"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _rls(tbl: str):
    try:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(
            f"""CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass

def _add_col(table, column):
    try:
        op.add_column(table, column)
    except Exception:
        pass


def upgrade() -> None:
    # --- 1. call_block ---
    _add_col("call_block", sa.Column("call_block_direction", sa.String(length=32), nullable=True))
    _add_col("call_block", sa.Column("extension_uuid", sa.Uuid(), nullable=True))
    _add_col("call_block", sa.Column("call_block_country_code", sa.String(length=8), nullable=True))
    _add_col("call_block", sa.Column("call_block_app", sa.String(length=64), nullable=True))
    _add_col("call_block", sa.Column("call_block_data", sa.String(length=512), nullable=True))
    _add_col("call_block", sa.Column("date_added", sa.DateTime(timezone=True), nullable=True))
    _add_col("call_block", sa.Column("insert_user", sa.Uuid(), nullable=True))
    _add_col("call_block", sa.Column("update_user", sa.Uuid(), nullable=True))
    # try FKs (best effort, ignore if extensions/users not present or sqlite)
    for fk, col, ref_tbl, ref_col in [
        ("fk_call_block_extension", "extension_uuid", "extensions", "extension_uuid"),
        ("fk_call_block_insert_user", "insert_user", "users", "user_uuid"),
        ("fk_call_block_update_user", "update_user", "users", "user_uuid"),
    ]:
        try:
            op.create_foreign_key(fk, "call_block", ref_tbl, [col], [ref_col], ondelete="SET NULL")
        except Exception:
            pass
    try:
        op.create_index("ix_call_block_extension", "call_block", ["extension_uuid"])
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_block;")
        op.execute("""CREATE VIEW v_call_block AS SELECT
            domain_id AS domain_uuid,
            call_block_uuid,
            call_block_direction,
            extension_uuid,
            call_block_name,
            call_block_country_code,
            call_block_number,
            call_block_count,
            call_block_action,
            call_block_app,
            call_block_data,
            date_added,
            call_block_enabled,
            call_block_description,
            created_at AS insert_date,
            insert_user,
            updated_at AS update_date,
            update_user
            FROM call_block;""")
    except Exception:
        pass

    # --- 2. call_broadcasts ---
    _add_col("call_broadcasts", sa.Column("broadcast_start_time", sa.Integer(), nullable=True))
    _add_col("call_broadcasts", sa.Column("recording_uuid", sa.Uuid(), nullable=True))
    _add_col("call_broadcasts", sa.Column("broadcast_avmd", sa.Boolean(), nullable=True))
    _add_col("call_broadcasts", sa.Column("broadcast_destination_data", sa.String(length=512), nullable=True))
    _add_col("call_broadcasts", sa.Column("broadcast_accountcode", sa.String(length=64), nullable=True))
    _add_col("call_broadcasts", sa.Column("broadcast_toll_allow", sa.String(length=128), nullable=True))
    # NOTE: no FK to recordings(recording_uuid) — its PK is composite
    # (recording_uuid, domain_id), so a single-column FK is invalid. Integrity
    # is enforced at the application layer. (No DROP here either: emitting any
    # failing statement aborts the whole PG transaction even when caught.)
    try:
        op.create_index("ix_call_broadcasts_recording", "call_broadcasts", ["recording_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_call_broadcasts_start", "call_broadcasts", ["broadcast_start_time"])
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_broadcasts;")
        op.execute("""CREATE VIEW v_call_broadcasts AS SELECT
            domain_id AS domain_uuid,
            call_broadcast_uuid,
            broadcast_name,
            broadcast_description,
            broadcast_start_time,
            broadcast_timeout,
            broadcast_concurrent_limit,
            recording_uuid,
            broadcast_caller_id_name,
            broadcast_caller_id_number,
            broadcast_destination_type,
            broadcast_destination_data,
            broadcast_phone_numbers,
            broadcast_avmd,
            broadcast_accountcode,
            broadcast_toll_allow,
            broadcast_enabled,
            created_at AS insert_date,
            updated_at AS update_date
            FROM call_broadcasts;""")
    except Exception:
        pass

    # --- 3. call_centers queues ---
    cc_cols = [
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=True),
        sa.Column("queue_language", sa.String(length=32), nullable=True),
        sa.Column("queue_dialect", sa.String(length=32), nullable=True),
        sa.Column("queue_voice", sa.String(length=32), nullable=True),
        sa.Column("queue_record_template", sa.String(length=256), nullable=True),
        sa.Column("queue_time_base_score", sa.String(length=32), nullable=True),
        sa.Column("queue_time_base_score_sec", sa.Integer(), nullable=True),
        sa.Column("queue_max_wait_time", sa.Integer(), nullable=True),
        sa.Column("queue_max_wait_time_with_no_agent", sa.Integer(), nullable=True),
        sa.Column("queue_max_wait_time_with_no_agent_time_reached", sa.Integer(), nullable=True),
        sa.Column("queue_tier_rules_apply", sa.Boolean(), nullable=True),
        sa.Column("queue_tier_rule_wait_second", sa.Integer(), nullable=True),
        sa.Column("queue_tier_rule_wait_multiply_level", sa.Boolean(), nullable=True),
        sa.Column("queue_tier_rule_no_agent_no_wait", sa.Boolean(), nullable=True),
        sa.Column("queue_timeout_action", sa.String(length=512), nullable=True),
        sa.Column("queue_discard_abandoned_after", sa.Integer(), nullable=True),
        sa.Column("queue_abandoned_resume_allowed", sa.Boolean(), nullable=True),
        sa.Column("queue_cid_prefix", sa.String(length=64), nullable=True),
        sa.Column("queue_outbound_caller_id_name", sa.String(length=128), nullable=True),
        sa.Column("queue_outbound_caller_id_number", sa.String(length=32), nullable=True),
        sa.Column("queue_announce_position", sa.String(length=32), nullable=True),
        sa.Column("queue_announce_sound", sa.String(length=128), nullable=True),
        sa.Column("queue_announce_frequency", sa.Integer(), nullable=True),
        sa.Column("queue_cc_exit_keys", sa.String(length=32), nullable=True),
        sa.Column("queue_email_address", sa.String(length=255), nullable=True),
        sa.Column("queue_context", sa.String(length=128), nullable=True),
        sa.Column("queue_limit", sa.String(length=32), nullable=True),
    ]
    for col in cc_cols:
        _add_col("call_centers", col)
    # try FK for dialplan_uuid
    try:
        op.create_foreign_key("fk_cc_dialplan", "call_centers", "dialplans", ["dialplan_uuid"], ["dialplan_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_call_centers_dialplan", "call_centers", ["dialplan_uuid"])
    except Exception:
        pass

    # agents expansion
    agent_cols = [
        sa.Column("user_uuid", sa.Uuid(), nullable=True),
        sa.Column("agent_call_timeout", sa.Integer(), nullable=True),
        sa.Column("agent_id", sa.String(length=64), nullable=True),
        sa.Column("agent_password", sa.String(length=128), nullable=True),
        sa.Column("agent_logout", sa.String(length=32), nullable=True),
        sa.Column("agent_max_no_answer", sa.Integer(), nullable=True),
        sa.Column("agent_wrap_up_time", sa.Integer(), nullable=True),
        sa.Column("agent_reject_delay_time", sa.Integer(), nullable=True),
        sa.Column("agent_busy_delay_time", sa.Integer(), nullable=True),
        sa.Column("agent_no_answer_delay_time", sa.String(length=64), nullable=True),
        sa.Column("agent_record", sa.Boolean(), nullable=True),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=True),
    ]
    for col in agent_cols:
        _add_col("call_center_agents", col)
    try:
        op.create_foreign_key("fk_cca_user", "call_center_agents", "users", ["user_uuid"], ["user_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_call_center_agents_user", "call_center_agents", ["user_uuid"])
    except Exception:
        pass

    # create tiers table
    try:
        op.create_table(
            "call_center_tiers",
            sa.Column("call_center_tier_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("call_center_queue_uuid", sa.Uuid(), nullable=False),
            sa.Column("call_center_agent_uuid", sa.Uuid(), nullable=False),
            sa.Column("agent_name", sa.String(length=128), nullable=True),
            sa.Column("queue_name", sa.String(length=128), nullable=True),
            sa.Column("tier_level", sa.Integer(), nullable=True),
            sa.Column("tier_position", sa.Integer(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["call_center_queue_uuid"], ["call_centers.call_center_queue_uuid"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["call_center_agent_uuid"], ["call_center_agents.call_center_agent_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("call_center_tier_uuid"),
        )
    except Exception:
        pass
    for col in ["domain_id", "call_center_queue_uuid", "call_center_agent_uuid"]:
        try:
            op.create_index(f"ix_call_center_tiers_{col}", "call_center_tiers", [col])
        except Exception:
            pass
    _rls("call_center_tiers")
    try:
        op.execute("DROP VIEW IF EXISTS v_call_center_tiers;")
        op.execute("CREATE VIEW v_call_center_tiers AS SELECT domain_id AS domain_uuid, call_center_tier_uuid, call_center_queue_uuid, call_center_agent_uuid, agent_name, queue_name, tier_level, tier_position, created_at AS insert_date, updated_at AS update_date FROM call_center_tiers;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_center_queues;")
        op.execute("""CREATE VIEW v_call_center_queues AS SELECT domain_id AS domain_uuid, call_center_queue_uuid, dialplan_uuid, queue_name, queue_extension, queue_greeting, queue_strategy, queue_moh_sound, queue_record_template, queue_language, queue_dialect, queue_voice, queue_cid_prefix, queue_outbound_caller_id_name, queue_outbound_caller_id_number, queue_timeout_action, queue_max_wait_time, queue_description, queue_context, queue_enabled, created_at AS insert_date, updated_at AS update_date FROM call_centers;""")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_center_agents;")
        op.execute("""CREATE VIEW v_call_center_agents AS SELECT domain_id AS domain_uuid, call_center_agent_uuid, call_center_queue_uuid, user_uuid, agent_name, agent_type, agent_contact, agent_status, agent_call_timeout, agent_id, agent_password, agent_max_no_answer, agent_wrap_up_time, agent_record, created_at AS insert_date, updated_at AS update_date FROM call_center_agents;""")
    except Exception:
        pass

    # --- 4. call_flows ---
    _add_col("call_flows", sa.Column("dialplan_uuid", sa.Uuid(), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_status", sa.String(length=32), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_pin_number", sa.String(length=32), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_label", sa.String(length=128), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_app", sa.String(length=64), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_data", sa.String(length=512), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_alternate_sound", sa.String(length=128), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_alternate_app", sa.String(length=64), nullable=True))
    _add_col("call_flows", sa.Column("call_flow_alternate_data", sa.String(length=512), nullable=True))
    try:
        op.create_foreign_key("fk_cf_dialplan", "call_flows", "dialplans", ["dialplan_uuid"], ["dialplan_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_call_flows_dialplan", "call_flows", ["dialplan_uuid"])
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_flows;")
        op.execute("""CREATE VIEW v_call_flows AS SELECT
            domain_id AS domain_uuid,
            call_flow_uuid,
            dialplan_uuid,
            call_flow_name,
            call_flow_extension,
            call_flow_feature_code,
            call_flow_context,
            call_flow_status,
            call_flow_pin_number,
            call_flow_label,
            call_flow_sound,
            call_flow_app,
            call_flow_data,
            call_flow_alternate_label,
            call_flow_alternate_sound,
            call_flow_alternate_app,
            call_flow_alternate_data,
            call_flow_enabled,
            call_flow_description,
            created_at AS insert_date,
            updated_at AS update_date
            FROM call_flows;""")
    except Exception:
        pass

    # --- 5. recordings transcription/summary + call_recordings table ---
    _add_col("recordings", sa.Column("transcription", sa.Text(), nullable=True))
    _add_col("recordings", sa.Column("summary", sa.Text(), nullable=True))
    try:
        op.execute("DROP VIEW IF EXISTS v_recordings;")
        op.execute("""CREATE VIEW v_recordings AS SELECT domain_id AS domain_uuid, recording_uuid, name, file_path, transcription, summary, created_at AS insert_date FROM recordings;""")
    except Exception:
        pass
    try:
        op.create_table(
            "call_recordings",
            sa.Column("call_recording_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("cdr_uuid", sa.Uuid(), nullable=True),
            sa.Column("recording_path", sa.String(length=512), nullable=True),
            sa.Column("recording_name", sa.String(length=255), nullable=True),
            sa.Column("transcription", sa.Text(), nullable=True),
            sa.Column("summary", sa.Text(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            # NOTE: no FK cdr_uuid -> cdr(cdr_uuid): cdr PK is composite
            # (cdr_uuid, domain_id). Application-layer integrity only.
            sa.PrimaryKeyConstraint("call_recording_uuid"),
        )
    except Exception:
        pass
    for col in ["domain_id", "cdr_uuid"]:
        try:
            op.create_index(f"ix_call_recordings_{col}", "call_recordings", [col])
        except Exception:
            pass
    _rls("call_recordings")
    try:
        op.execute("DROP VIEW IF EXISTS v_call_recordings;")
        op.execute("CREATE VIEW v_call_recordings AS SELECT domain_id AS domain_uuid, call_recording_uuid, cdr_uuid, recording_path, recording_name, transcription, summary, created_at AS insert_date, updated_at AS update_date FROM call_recordings;")
    except Exception:
        pass
    # also legacy alias view call_recordings is often alias to recordings; ensure both exist

    # seed permissions (idempotent)
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'call_block_view'),
            (gen_random_uuid(), 'call_block_add'),
            (gen_random_uuid(), 'call_block_edit'),
            (gen_random_uuid(), 'call_block_delete'),
            (gen_random_uuid(), 'call_broadcast_view'),
            (gen_random_uuid(), 'call_broadcast_add'),
            (gen_random_uuid(), 'call_broadcast_edit'),
            (gen_random_uuid(), 'call_broadcast_delete'),
            (gen_random_uuid(), 'call_center_tier_view'),
            (gen_random_uuid(), 'call_center_tier_add'),
            (gen_random_uuid(), 'call_center_tier_edit'),
            (gen_random_uuid(), 'call_center_tier_delete'),
            (gen_random_uuid(), 'call_flow_view'),
            (gen_random_uuid(), 'call_flow_add'),
            (gen_random_uuid(), 'call_flow_edit'),
            (gen_random_uuid(), 'call_flow_delete'),
            (gen_random_uuid(), 'call_recording_view'),
            (gen_random_uuid(), 'call_recording_add'),
            (gen_random_uuid(), 'call_recording_edit'),
            (gen_random_uuid(), 'call_recording_delete')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN (
                'call_block_view','call_block_add','call_block_edit','call_block_delete',
                'call_broadcast_view','call_broadcast_add','call_broadcast_edit','call_broadcast_delete',
                'call_center_tier_view','call_center_tier_add','call_center_tier_edit','call_center_tier_delete',
                'call_flow_view','call_flow_add','call_flow_edit','call_flow_delete',
                'call_recording_view','call_recording_add','call_recording_edit','call_recording_delete'
            )
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_call_recordings;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON call_recordings;")
        op.execute("ALTER TABLE call_recordings DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("call_recordings")
    except Exception:
        pass
    for col in ["summary", "transcription"]:
        try:
            op.drop_column("recordings", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_flows;")
        op.execute("CREATE VIEW v_call_flows AS SELECT domain_id AS domain_uuid, call_flow_uuid, call_flow_name, call_flow_extension, call_flow_feature_code, call_flow_context, call_flow_enabled, call_flow_description, created_at AS insert_date FROM call_flows;")
    except Exception:
        pass
    for col in ["call_flow_alternate_data", "call_flow_alternate_app", "call_flow_alternate_sound", "call_flow_data", "call_flow_app", "call_flow_label", "call_flow_pin_number", "call_flow_status", "dialplan_uuid"]:
        try:
            op.drop_column("call_flows", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_center_agents;")
        op.execute("DROP VIEW IF EXISTS v_call_center_queues;")
        op.execute("DROP VIEW IF EXISTS v_call_center_tiers;")
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON call_center_tiers;")
        op.execute("ALTER TABLE call_center_tiers DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("call_center_tiers")
    except Exception:
        pass
    for col in ["updated_at", "agent_record", "agent_no_answer_delay_time", "agent_busy_delay_time", "agent_reject_delay_time", "agent_wrap_up_time", "agent_max_no_answer", "agent_logout", "agent_password", "agent_id", "agent_call_timeout", "user_uuid"]:
        try:
            op.drop_column("call_center_agents", col)
        except Exception:
            pass
    for col in ["queue_limit", "queue_context", "queue_email_address", "queue_cc_exit_keys", "queue_announce_frequency", "queue_announce_sound", "queue_announce_position", "queue_outbound_caller_id_number", "queue_outbound_caller_id_name", "queue_cid_prefix", "queue_abandoned_resume_allowed", "queue_discard_abandoned_after", "queue_timeout_action", "queue_tier_rule_no_agent_no_wait", "queue_tier_rule_wait_multiply_level", "queue_tier_rule_wait_second", "queue_tier_rules_apply", "queue_max_wait_time_with_no_agent_time_reached", "queue_max_wait_time_with_no_agent", "queue_max_wait_time", "queue_time_base_score_sec", "queue_time_base_score", "queue_record_template", "queue_voice", "queue_dialect", "queue_language", "dialplan_uuid"]:
        try:
            op.drop_column("call_centers", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_broadcasts;")
    except Exception:
        pass
    for col in ["broadcast_toll_allow", "broadcast_accountcode", "broadcast_destination_data", "broadcast_avmd", "recording_uuid", "broadcast_start_time"]:
        try:
            op.drop_column("call_broadcasts", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_call_block;")
        op.execute("CREATE VIEW v_call_block AS SELECT domain_id AS domain_uuid, call_block_uuid, call_block_name, call_block_number, call_block_enabled, call_block_count, call_block_description, created_at AS insert_date FROM call_block;")
    except Exception:
        pass
    for col in ["update_user", "insert_user", "date_added", "call_block_data", "call_block_app", "call_block_country_code", "extension_uuid", "call_block_direction"]:
        try:
            op.drop_column("call_block", col)
        except Exception:
            pass
