"""fifo batch parity: fifo 10 cols + fifo_members, fax_queue 17 cols, feature_codes perms, moh perms fix, operator_panel 16 perms + 32 defaults

Revision ID: 038_fifo_batch_fix
Revises: 037_dest_db_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "038_fifo_batch_fix"
down_revision: Union[str, None] = "037_dest_db_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _add(table, col):
    try:
        op.add_column(table, col)
    except Exception:
        pass


def _rls(tbl):
    try:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(f"""CREATE POLICY tenant_isolation ON {tbl} USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '') WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');""")
    except Exception:
        pass


def upgrade() -> None:
    # --- 1. fifo: add missing 11 cols ---
    fifo_cols = [
        sa.Column("dialplan_uuid", sa.Uuid(), nullable=True),
        sa.Column("fifo_agent_queue", sa.Text(), nullable=True),
        sa.Column("fifo_agent_status", sa.Text(), nullable=True),
        sa.Column("fifo_members", sa.Text(), nullable=True),
        sa.Column("fifo_timeout_seconds", sa.Text(), nullable=True),
        sa.Column("fifo_exit_key", sa.String(length=8), nullable=True),
        sa.Column("fifo_exit_action", sa.Text(), nullable=True),
        sa.Column("fifo_music", sa.Text(), nullable=True),
        sa.Column("fifo_order", sa.Integer(), nullable=True),
        sa.Column("insert_user", sa.Uuid(), nullable=True),
        sa.Column("update_user", sa.Uuid(), nullable=True),
    ]
    for c in fifo_cols:
        _add("fifo", c)
    try:
        op.create_foreign_key("fk_fifo_dialplan", "fifo", "dialplans", ["dialplan_uuid"], ["dialplan_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_fifo_dialplan_uuid", "fifo", ["dialplan_uuid"])
    except Exception:
        pass

    # fifo_members table
    try:
        op.create_table(
            "fifo_members",
            sa.Column("fifo_member_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("fifo_uuid", sa.Uuid(), nullable=False),
            sa.Column("member_contact", sa.Text(), nullable=True),
            sa.Column("member_call_timeout", sa.Integer(), nullable=True),
            sa.Column("member_simultaneous", sa.Integer(), nullable=True),
            sa.Column("member_wrap_up_time", sa.Integer(), nullable=True),
            sa.Column("member_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
            sa.Column("insert_user", sa.Uuid(), nullable=True),
            sa.Column("update_user", sa.Uuid(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["fifo_uuid"], ["fifo.fifo_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("fifo_member_uuid"),
        )
    except Exception:
        pass
    for idx in ["domain_id", "fifo_uuid"]:
        try:
            op.create_index(f"ix_fifo_members_{idx}", "fifo_members", [idx])
        except Exception:
            pass
    _rls("fifo_members")
    try:
        op.execute("DROP VIEW IF EXISTS v_fifo;")
        op.execute("""
            CREATE VIEW v_fifo AS SELECT
                domain_id AS domain_uuid,
                fifo_uuid,
                dialplan_uuid,
                fifo_name,
                fifo_extension,
                fifo_agent_queue,
                fifo_agent_status,
                fifo_strategy,
                fifo_members,
                fifo_timeout_seconds,
                fifo_exit_key,
                fifo_exit_action,
                fifo_music,
                fifo_order,
                fifo_enabled,
                fifo_description,
                created_at AS insert_date,
                insert_user,
                updated_at AS update_date,
                update_user
            FROM fifo;
        """)
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fifo_members;")
        op.execute("""
            CREATE VIEW v_fifo_members AS SELECT
                domain_id AS domain_uuid,
                fifo_member_uuid,
                fifo_uuid,
                member_contact,
                member_call_timeout,
                member_simultaneous,
                member_wrap_up_time,
                member_enabled,
                created_at AS insert_date,
                insert_user,
                updated_at AS update_date,
                update_user
            FROM fifo_members;
        """)
    except Exception:
        pass
    # fifo perms
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'fifo_all'),
            (gen_random_uuid(), 'fifo_member_view'),
            (gen_random_uuid(), 'fifo_member_add'),
            (gen_random_uuid(), 'fifo_member_edit'),
            (gen_random_uuid(), 'fifo_member_delete'),
            (gen_random_uuid(), 'fifo_member_all'),
            (gen_random_uuid(), 'fifo_destinations')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('fifo_all','fifo_member_view','fifo_member_add','fifo_member_edit','fifo_member_delete','fifo_member_all','fifo_destinations')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass

    # --- 2. feature_codes: ensure insert_user/update_user cols + perms ---
    _add("feature_codes", sa.Column("insert_user", sa.Uuid(), nullable=True))
    _add("feature_codes", sa.Column("update_user", sa.Uuid(), nullable=True))
    try:
        op.execute("DROP VIEW IF EXISTS v_feature_codes;")
        op.execute("""
            CREATE VIEW v_feature_codes AS SELECT
                domain_id AS domain_uuid,
                feature_code_uuid,
                feature_code_name,
                feature_code_value,
                feature_code_enabled,
                feature_code_description,
                created_at AS insert_date,
                insert_user,
                updated_at AS update_date,
                update_user
            FROM feature_codes;
        """)
    except Exception:
        pass
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'feature_code_view'),
            (gen_random_uuid(), 'feature_code_add'),
            (gen_random_uuid(), 'feature_code_edit'),
            (gen_random_uuid(), 'feature_code_delete'),
            (gen_random_uuid(), 'feature_code_all')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('feature_code_view','feature_code_add','feature_code_edit','feature_code_delete','feature_code_all')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass

    # --- 3. fax_queue: 17 missing cols ---
    fq_cols = [
        sa.Column("origination_uuid", sa.Uuid(), nullable=True),
        sa.Column("fax_log_uuid", sa.Uuid(), nullable=True),
        sa.Column("fax_date", sa.DateTime(timezone=True), nullable=True),
        sa.Column("hostname", sa.Text(), nullable=True),
        sa.Column("fax_caller_id_name", sa.Text(), nullable=True),
        sa.Column("fax_caller_id_number", sa.Text(), nullable=True),
        sa.Column("fax_recipient", sa.Text(), nullable=True),
        sa.Column("fax_prefix", sa.Text(), nullable=True),
        sa.Column("fax_email_address", sa.Text(), nullable=True),
        sa.Column("fax_retry_date", sa.DateTime(timezone=True), nullable=True),
        sa.Column("fax_notify_sent", sa.Boolean(), nullable=True),
        sa.Column("fax_notify_date", sa.DateTime(timezone=True), nullable=True),
        sa.Column("fax_accountcode", sa.Text(), nullable=True),
        sa.Column("fax_command", sa.Text(), nullable=True),
        sa.Column("fax_response", sa.Text(), nullable=True),
        sa.Column("insert_user", sa.Uuid(), nullable=True),
        sa.Column("update_user", sa.Uuid(), nullable=True),
    ]
    for c in fq_cols:
        _add("fax_queue", c)
    try:
        op.create_index("ix_fax_queue_origination_uuid", "fax_queue", ["origination_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_fax_queue_fax_log_uuid", "fax_queue", ["fax_log_uuid"])
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_queue;")
        op.execute("""
            CREATE VIEW v_fax_queue AS SELECT
                domain_id AS domain_uuid,
                fax_queue_uuid,
                fax_uuid,
                origination_uuid,
                fax_log_uuid,
                fax_date,
                hostname,
                fax_caller_id_name,
                fax_caller_id_number,
                fax_recipient,
                fax_number,
                fax_prefix,
                fax_email_address,
                fax_file,
                fax_status,
                fax_retry_date,
                fax_notify_sent,
                fax_notify_date,
                fax_retry_count,
                fax_accountcode,
                fax_command,
                fax_response,
                created_at AS insert_date,
                insert_user,
                updated_at AS update_date,
                update_user
            FROM fax_queue;
        """)
    except Exception:
        pass
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'fax_queue_domain'),
            (gen_random_uuid(), 'fax_queue_edit'),
            (gen_random_uuid(), 'fax_queue_delete'),
            (gen_random_uuid(), 'fax_queue_all')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('fax_queue_domain','fax_queue_edit','fax_queue_delete','fax_queue_all')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass

    # --- 4. music_on_hold: chime_max + insert_user/update_user ---
    _add("music_on_hold", sa.Column("music_chime_max", sa.Integer(), nullable=True))
    _add("music_on_hold", sa.Column("insert_user", sa.Uuid(), nullable=True))
    _add("music_on_hold", sa.Column("update_user", sa.Uuid(), nullable=True))
    try:
        op.execute("DROP VIEW IF EXISTS v_music_on_hold;")
        op.execute("""
            CREATE VIEW v_music_on_hold AS SELECT
                domain_id AS domain_uuid,
                music_on_hold_uuid,
                music_name AS music_on_hold_name,
                music_path AS music_on_hold_path,
                music_rate AS music_on_hold_rate,
                music_shuffle AS music_on_hold_shuffle,
                music_channels AS music_on_hold_channels,
                music_interval AS music_on_hold_interval,
                music_timer_name AS music_on_hold_timer_name,
                music_chime_list AS music_on_hold_chime_list,
                music_chime_freq AS music_on_hold_chime_freq,
                music_chime_max AS music_on_hold_chime_max,
                enabled AS music_on_hold_enabled,
                description AS music_on_hold_description,
                created_at AS insert_date,
                insert_user,
                updated_at AS update_date,
                update_user
            FROM music_on_hold;
        """)
    except Exception:
        pass
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'music_on_hold_all'),
            (gen_random_uuid(), 'music_on_hold_name'),
            (gen_random_uuid(), 'music_on_hold_domain'),
            (gen_random_uuid(), 'music_on_hold_global'),
            (gen_random_uuid(), 'music_on_hold_path'),
            (gen_random_uuid(), 'music_on_hold_map')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('music_on_hold_all','music_on_hold_name','music_on_hold_domain','music_on_hold_global','music_on_hold_path','music_on_hold_map')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass

    # --- 5. operator_panel: 16 perms + 32 defaults ---
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'operator_panel_view'),
            (gen_random_uuid(), 'operator_panel_manage'),
            (gen_random_uuid(), 'operator_panel_originate'),
            (gen_random_uuid(), 'operator_panel_hangup'),
            (gen_random_uuid(), 'operator_panel_eavesdrop'),
            (gen_random_uuid(), 'operator_panel_coach'),
            (gen_random_uuid(), 'operator_panel_record'),
            (gen_random_uuid(), 'operator_panel_call_details'),
            (gen_random_uuid(), 'operator_panel_on_demand'),
            (gen_random_uuid(), 'operator_panel_transfer_attended'),
            (gen_random_uuid(), 'active_call_whisper'),
            (gen_random_uuid(), 'active_call_barge'),
            (gen_random_uuid(), 'operator_panel_extensions'),
            (gen_random_uuid(), 'operator_panel_calls'),
            (gen_random_uuid(), 'operator_panel_conferences'),
            (gen_random_uuid(), 'operator_panel_agents')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name IN ('superadmin','admin') AND p.name IN (
                'operator_panel_view','operator_panel_manage','operator_panel_originate','operator_panel_hangup',
                'operator_panel_eavesdrop','operator_panel_coach','operator_panel_record','operator_panel_call_details',
                'operator_panel_transfer_attended','operator_panel_extensions','operator_panel_calls','operator_panel_conferences','operator_panel_agents'
            )
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
        # on_demand / whisper / barge are typically superadmin only but also allow admin via legacy, keep superadmin
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('operator_panel_on_demand','active_call_whisper','active_call_barge')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass
    # operator_panel defaults — 32 entries from app_config.php
    defaults = [
        ("e150a7ae-e06c-4e48-98a2-fec35b469895","operator_panel","reconnect_delay","numeric","500","Delay before reconnect"),
        ("0dfc9e80-7451-46c3-b89d-0c14679d655e","operator_panel","ping_interval","numeric","5000","Keepalive ping interval"),
        ("eeca82cf-3743-4728-9a4d-207a647788e0","operator_panel","auth_timeout","numeric","5000","WS auth timeout"),
        ("50b08093-19f6-450f-b534-b4aff88fdd38","operator_panel","pong_timeout","numeric","1500","Pong timeout"),
        ("cb357440-9d16-40b3-ba99-ea7973a21610","operator_panel","pong_timeout_max_retries","numeric","2","Max pong retries"),
        ("10057611-f272-45a4-a23f-144633277596","operator_panel","max_reconnect_delay","numeric","5000","Max reconnect delay"),
        ("0cfc12b2-c9fd-4a45-99ef-fc7ed3616954","operator_panel","refresh_interval","numeric","0","Refresh interval 0=WS only"),
        ("b4d6fbcb-0a6e-4e92-bbe6-d8c68776a4b5","operator_panel","agent_stats_interval","numeric","10","Agent stats interval"),
        ("a2482295-3a47-4188-a4b8-6c303f2e62e8","operator_panel","card_label_position","text","left","Card label position"),
        ("9a25fc04-4fc8-4027-97fe-9e4723346892","operator_panel","my_extensions_button_visible","boolean","false","My Extensions button"),
        ("c4a7db2a-ec69-4aef-a95b-1a8f2d7d2de1","operator_panel","registrations_reconcile_enabled","boolean","false","Registration reconcile"),
        ("f3be5aa5-2b7e-4e8c-a00a-ae9e001ec646","operator_panel","debug_show_permissions_mode","text","off","Debug permissions mode"),
        ("ef6bb923-21cd-4c0d-acdd-60646bfac3ab","theme","operator_panel_status_connected","text","#28a745","Status connected"),
        ("2749a174-80ec-475f-a3a3-40be46ce524f","theme","operator_panel_status_warning","text","#ffc107","Status warning"),
        ("dad10272-120f-42c3-8da6-34f17315dc6a","theme","operator_panel_status_disconnected","text","#dc3545","Status disconnected"),
        ("01be0a26-8ad4-4558-93e5-41979a2e84d7","theme","operator_panel_status_connecting","text","#6c757d","Status connecting"),
        ("b6a02563-e1d8-4f7e-9732-68e2f1266d6c","theme","operator_panel_status_icon_connected","text","fa-solid fa-plug-circle-check","Icon connected"),
        ("d06a9ef7-d589-4267-920f-351b9255b6d0","theme","operator_panel_status_icon_warning","text","fa-solid fa-plug-circle-exclamation","Icon warning"),
        ("7f747305-b93d-467f-9d27-44ab76c77c16","theme","operator_panel_status_icon_disconnected","text","fa-solid fa-plug-circle-xmark","Icon disconnected"),
        ("f01ebd98-461b-4644-9946-38f3b8fd7fca","theme","operator_panel_status_icon_connecting","text","fa-solid fa-plug fa-fade","Icon connecting"),
        ("ec8f0281-5d2b-4a3b-9481-f2c9880db101","theme","operator_panel_conference_icon_mute","text","fas fa-microphone","Conf mute"),
        ("3d4dbe79-f1af-4bea-804e-27ee0ac31302","theme","operator_panel_conference_icon_unmute","text","fas fa-microphone-slash","Conf unmute"),
        ("b15d1ae4-6087-48cf-a46f-148fb5236103","theme","operator_panel_conference_icon_deaf","text","fas fa-headphones","Conf deaf"),
        ("2b560cff-58ef-42c2-b4e5-08d77c339a04","theme","operator_panel_conference_icon_undeaf","text","fas fa-deaf","Conf undeaf"),
        ("21f8af30-d08f-4342-bddb-bb68804e1005","theme","operator_panel_conference_icon_energy_up","text","fas fa-plus","Conf energy up"),
        ("4fdc30b2-7db8-4c9f-9b7e-4c78fa757706","theme","operator_panel_conference_icon_energy_down","text","fas fa-minus","Conf energy down"),
        ("2b706528-8a6b-409c-84bb-95f905344707","theme","operator_panel_conference_icon_volume_down","text","fas fa-volume-down","Conf vol down"),
        ("9220ec76-6760-4b75-a29e-503b94ca7508","theme","operator_panel_conference_icon_volume_up","text","fas fa-volume-up","Conf vol up"),
        ("866dc078-68cb-43aa-b70b-b73d213dbd09","theme","operator_panel_conference_icon_gain_down","text","fas fa-sort-amount-down","Conf gain down"),
        ("d4bde1f1-b0d8-4836-a78b-f8c4bb510a10","theme","operator_panel_conference_icon_gain_up","text","fas fa-sort-amount-up","Conf gain up"),
        ("f55d2796-46b5-4ca2-998f-256184489b11","theme","operator_panel_conference_icon_kick","text","fas fa-ban","Conf kick"),
        ("b367ce0d-beb0-424a-9fc9-0c83690001ee","theme","operator_panel_status_show_icon","boolean","true","Show status icon"),
    ]
    for uuid, cat, sub, name, val, desc in defaults:
        try:
            op.execute(sa.text("""
                INSERT INTO default_settings (default_setting_uuid, default_setting_category, default_setting_subcategory, default_setting_name, default_setting_value, default_setting_enabled, default_setting_description)
                VALUES (:uuid, :cat, :sub, :name, :val, true, :desc)
                ON CONFLICT (default_setting_uuid) DO NOTHING
            """), {"uuid": uuid, "cat": cat, "sub": sub, "name": name, "val": val, "desc": desc})
        except Exception:
            pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_fifo_members;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fifo;")
        op.execute("CREATE VIEW v_fifo AS SELECT domain_id AS domain_uuid, fifo_uuid, fifo_name, fifo_extension, fifo_enabled, fifo_description, created_at AS insert_date FROM fifo;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON fifo_members;")
        op.execute("ALTER TABLE fifo_members DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("fifo_members")
    except Exception:
        pass
    for col in ["update_user","insert_user","fifo_order","fifo_music","fifo_exit_action","fifo_exit_key","fifo_timeout_seconds","fifo_members","fifo_agent_status","fifo_agent_queue","dialplan_uuid"]:
        try:
            op.drop_column("fifo", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_feature_codes;")
    except Exception:
        pass
    for col in ["update_user","insert_user"]:
        try:
            op.drop_column("feature_codes", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_fax_queue;")
        op.execute("CREATE VIEW v_fax_queue AS SELECT domain_id AS domain_uuid, fax_queue_uuid, fax_number, fax_status, created_at AS insert_date FROM fax_queue;")
    except Exception:
        pass
    for col in ["update_user","insert_user","fax_response","fax_command","fax_accountcode","fax_notify_date","fax_notify_sent","fax_retry_date","fax_email_address","fax_prefix","fax_recipient","fax_caller_id_number","fax_caller_id_name","hostname","fax_date","fax_log_uuid","origination_uuid"]:
        try:
            op.drop_column("fax_queue", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_music_on_hold;")
        op.execute("CREATE VIEW v_music_on_hold AS SELECT domain_id AS domain_uuid, music_on_hold_uuid, music_name, music_path, music_rate, music_shuffle, enabled AS music_on_hold_enabled, description AS music_on_hold_description, created_at AS insert_date, updated_at AS update_date FROM music_on_hold;")
    except Exception:
        pass
    for col in ["update_user","insert_user","music_chime_max"]:
        try:
            op.drop_column("music_on_hold", col)
        except Exception:
            pass
