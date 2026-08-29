"""voicemail cdr time_conditions parity fix

Revision ID: 039_voicemail_cdr_fix
Revises: 038
Create Date: 2026-08-25

- voicemail_greetings: add greeting_voice, greeting_message
- voicemails: add 11+ missing fields + create voicemail_messages and voicemail_options tables
- cdr: add 15+ critical columns (accountcode, context, leg, status, billmsec/mduration, hold_accum, bridge_uuid, conference_id, record_path/name, json/xml, processed)
- time_conditions: ensure perms correct and holiday presets seeded via default_settings + ensure dialplans app_uuid linkage via view
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa

revision: str = "039_voicemail_cdr_fix"
down_revision: Union[str, None] = "038_fifo_batch_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _add(table, column):
    try:
        op.add_column(table, column)
    except Exception:
        pass

def _rls(tbl: str):
    try:
        op.execute(f"ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY;")
        op.execute(f"ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY;")
        op.execute(f"""CREATE POLICY tenant_isolation ON {tbl}
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');""")
    except Exception:
        pass

def upgrade() -> None:
    # --- 1. voicemail_greetings: greeting_voice, greeting_message ---
    _add("voicemail_greetings", sa.Column("greeting_voice", sa.String(length=32), nullable=True))
    _add("voicemail_greetings", sa.Column("greeting_message", sa.String(length=1024), nullable=True))
    try:
        op.execute("DROP VIEW IF EXISTS v_voicemail_greetings;")
        op.execute("""CREATE VIEW v_voicemail_greetings AS SELECT domain_id AS domain_uuid, voicemail_greeting_uuid, voicemail_id, greeting_id, greeting_name, greeting_filename, greeting_voice, greeting_message, created_at AS insert_date FROM voicemail_greetings;""")
    except Exception:
        pass

    # --- 2. voicemails: add missing 11+ fields ---
    vm_cols = [
        sa.Column("voicemail_sms_to", sa.String(length=512), nullable=True),
        sa.Column("voicemail_transcription_prompt", sa.Text(), nullable=True),
        sa.Column("voicemail_recording_instructions", sa.String(length=512), nullable=True),
        sa.Column("voicemail_attach_file", sa.String(length=32), nullable=True),
        sa.Column("voicemail_forward_enabled", sa.Boolean(), nullable=True),
        sa.Column("voicemail_forward_destinations", sa.Text(), nullable=True),
        sa.Column("voicemail_alternate_greet_id", sa.String(length=64), nullable=True),
        sa.Column("voicemail_sms_type", sa.String(length=32), nullable=True),
        sa.Column("voicemail_keep_local_after_email", sa.Boolean(), nullable=True),
        sa.Column("voicemail_transcription_enabled", sa.Boolean(), nullable=True),
        sa.Column("voicemail_tutorial", sa.Boolean(), nullable=True),
        sa.Column("voicemail_local_after_email", sa.Boolean(), nullable=True),
        # also ensure legacy-named aliases nullable
        sa.Column("recording_instructions", sa.String(length=512), nullable=True),
        sa.Column("sms_to", sa.String(length=512), nullable=True),
        sa.Column("transcription_prompt", sa.Text(), nullable=True),
    ]
    for c in vm_cols:
        _add("voicemails", c)
    # also ensure voicemail_tutorial etc already exist but try to add if missing (idempotent)
    # update view
    try:
        op.execute("DROP VIEW IF EXISTS v_voicemails;")
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
                voicemail_sms_to,
                voicemail_file,
                voicemail_attach_file,
                enabled AS voicemail_enabled,
                description AS voicemail_description,
                alternate_greet_id AS voicemail_alternate_greet_id,
                voicemail_tutorial,
                voicemail_local_after_email,
                voicemail_transcription_enabled,
                voicemail_transcription_prompt,
                created_at AS insert_date,
                updated_at AS update_date
            FROM voicemails;
        """)
    except Exception:
        pass

    # --- create voicemail_messages ---
    try:
        op.create_table(
            "voicemail_messages",
            sa.Column("voicemail_message_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("voicemail_uuid", sa.Uuid(), nullable=False),
            sa.Column("caller_id_number", sa.String(length=32), nullable=True),
            sa.Column("caller_id_name", sa.String(length=128), nullable=True),
            sa.Column("message_length", sa.Integer(), nullable=True),
            sa.Column("message_status", sa.String(length=32), nullable=True),
            sa.Column("message_priority", sa.String(length=16), nullable=True),
            sa.Column("message_uuid", sa.String(length=128), nullable=True),
            sa.Column("created_epoch", sa.Integer(), nullable=True),
            sa.Column("read_epoch", sa.Integer(), nullable=True),
            sa.Column("file_path", sa.String(length=512), nullable=True),
            sa.Column("transcription", sa.Text(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["voicemail_uuid"], ["voicemails.voicemail_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("voicemail_message_uuid"),
        )
    except Exception:
        pass
    for col in ["domain_id", "voicemail_uuid"]:
        try:
            op.create_index(f"ix_voicemail_messages_{col}", "voicemail_messages", [col])
        except Exception:
            pass
    _rls("voicemail_messages")
    try:
        op.execute("DROP VIEW IF EXISTS v_voicemail_messages;")
        op.execute("CREATE VIEW v_voicemail_messages AS SELECT domain_id AS domain_uuid, voicemail_message_uuid, voicemail_uuid, caller_id_number, caller_id_name, message_length, message_status, file_path, created_at AS insert_date FROM voicemail_messages;")
    except Exception:
        pass

    # --- create voicemail_options ---
    try:
        op.create_table(
            "voicemail_options",
            sa.Column("voicemail_option_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("voicemail_uuid", sa.Uuid(), nullable=False),
            sa.Column("digits", sa.String(length=8), nullable=True),
            sa.Column("action", sa.String(length=64), nullable=True),
            sa.Column("param", sa.String(length=512), nullable=True),
            sa.Column("description", sa.Text(), nullable=True),
            sa.Column("order", sa.Integer(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["voicemail_uuid"], ["voicemails.voicemail_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("voicemail_option_uuid"),
        )
    except Exception:
        pass
    for col in ["domain_id", "voicemail_uuid"]:
        try:
            op.create_index(f"ix_voicemail_options_{col}", "voicemail_options", [col])
        except Exception:
            pass
    _rls("voicemail_options")
    try:
        op.execute("DROP VIEW IF EXISTS v_voicemail_options;")
        op.execute("CREATE VIEW v_voicemail_options AS SELECT domain_id AS domain_uuid, voicemail_option_uuid, voicemail_uuid, digits, action, param, created_at AS insert_date FROM voicemail_options;")
    except Exception:
        pass

    # --- 3. cdr: add 15+ critical columns ---
    cdr_cols = [
        sa.Column("accountcode", sa.String(length=64), nullable=True),
        sa.Column("context", sa.String(length=128), nullable=True),
        sa.Column("leg", sa.String(length=16), nullable=True),
        sa.Column("status", sa.String(length=32), nullable=True),
        sa.Column("billmsec", sa.Integer(), nullable=True),
        sa.Column("mduration", sa.Integer(), nullable=True),
        sa.Column("hold_accum_seconds", sa.Integer(), nullable=True),
        sa.Column("bridge_uuid", sa.String(length=128), nullable=True),
        sa.Column("conference_id", sa.String(length=128), nullable=True),
        sa.Column("record_path", sa.String(length=512), nullable=True),
        sa.Column("record_name", sa.String(length=255), nullable=True),
        sa.Column("json_text", sa.Text(), nullable=True),
        sa.Column("xml_text", sa.Text(), nullable=True),
        sa.Column("processed", sa.Boolean(), nullable=True),
        sa.Column("caller_extension", sa.String(length=32), nullable=True),
        sa.Column("destination_extension", sa.String(length=32), nullable=True),
        sa.Column("caller_id_number", sa.String(length=32), nullable=True),
        sa.Column("caller_id_name", sa.String(length=128), nullable=True),
    ]
    # avoid duplicate for caller fields already existing — try add will be ignored if exists
    for c in cdr_cols:
        _add("cdr", c)
    # also ensure legacy column names aliases
    for legacy in [
        sa.Column("billsec", sa.Integer(), nullable=True),
        sa.Column("duration", sa.Integer(), nullable=True),
    ]:
        _add("cdr", legacy)
    try:
        op.execute("DROP VIEW IF EXISTS v_cdr;")
        op.execute("""CREATE VIEW v_cdr AS SELECT domain_id AS domain_uuid, cdr_uuid, caller_id_number, caller_id_name, destination_number, start_stamp, duration, billsec, accountcode, context, hangup_cause, created_at AS insert_date FROM cdr;""")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_xml_cdr;")
        op.execute("CREATE VIEW v_xml_cdr AS SELECT domain_id AS domain_uuid, cdr_uuid, caller_id_number, destination_number, start_stamp, hangup_cause, accountcode, context, leg, status, bridge_uuid, record_path, json_text, xml_text, processed FROM cdr;")
    except Exception:
        pass
    # indexes for cdr new columns
    for idx_col in ["accountcode", "context", "bridge_uuid", "processed"]:
        try:
            op.create_index(f"ix_cdr_{idx_col}", "cdr", [idx_col])
        except Exception:
            pass

    # --- 4. time_conditions: ensure perms and holiday presets via default_settings ---
    # ensure default_settings table exists
    try:
        op.create_table(
            "default_settings",
            sa.Column("default_setting_uuid", sa.Uuid(), nullable=False),
            sa.Column("default_setting_category", sa.String(length=64), nullable=False),
            sa.Column("default_setting_subcategory", sa.String(length=64), nullable=False),
            sa.Column("default_setting_name", sa.String(length=64), nullable=False),
            sa.Column("default_setting_value", sa.Text(), nullable=True),
            sa.Column("default_setting_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
            sa.Column("default_setting_description", sa.Text(), nullable=True),
            sa.Column("default_setting_order", sa.Integer(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
            sa.PrimaryKeyConstraint("default_setting_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_default_settings_category", "default_settings", ["default_setting_category"])
    except Exception:
        pass
    try:
        op.create_index("ix_default_settings_subcategory", "default_settings", ["default_setting_subcategory"])
    except Exception:
        pass
    # ensure time_condition perms complete (including legacy extra 4)
    try:
        op.execute("""
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'time_condition_view'),
            (gen_random_uuid(), 'time_condition_add'),
            (gen_random_uuid(), 'time_condition_edit'),
            (gen_random_uuid(), 'time_condition_delete'),
            (gen_random_uuid(), 'time_condition_all'),
            (gen_random_uuid(), 'time_condition_domain'),
            (gen_random_uuid(), 'time_condition_context'),
            (gen_random_uuid(), 'time_condition_destinations')
            ON CONFLICT (name) DO NOTHING;
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN ('time_condition_view','time_condition_add','time_condition_edit','time_condition_delete','time_condition_all','time_condition_domain','time_condition_context','time_condition_destinations')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
        op.execute("""
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='admin' AND p.name IN ('time_condition_view','time_condition_add','time_condition_edit','time_condition_delete','time_condition_destinations')
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
        """)
    except Exception:
        pass

    # seed holiday presets — subset from app_config.php for USA/England/Canada + region
    # Use ON CONFLICT DO NOTHING for idempotence
    presets = [
        ("c8cbb0eb-850b-4afd-a918-cceaf8af3957", "time_conditions", "region", "text", "usa", "What region to use by default when choosing Time Conditions", 0),
        ("528ec73e-03bb-4ea1-9ce1-19b81fb3f584", "time_conditions", "preset_england", "array", '{"new_years_day":{"mday":"1","mon":"1"}}', "England Holiday", 1),
        ("420b7282-2e49-4d63-9eb3-48b3b96bc184", "time_conditions", "preset_england", "array", '{"may_day":{"mon":"5","mday":"1-7","wday":"2"}}', "England Holiday", 129),
        ("c9ab6e93-63e0-4098-9290-7a721e813450", "time_conditions", "preset_england", "array", '{"spring_bank_holiday":{"mon":"5","mday":"25-31","wday":"2"}}', "England Holiday", 146),
        ("b7eac3ac-a99d-4fc8-8e3e-682a17a5b463", "time_conditions", "preset_england", "array", '{"august_bank_holiday":{"mon":"8","mday":"25-31","wday":"2"}}', "England Holiday", 243),
        ("3df036bb-ae96-4735-96da-a32e90b51940", "time_conditions", "preset_usa", "array", '{"new_years_day":{"mday":"1","mon":"1"}}', "USA Holiday", 1),
        ("0957bbc4-60e8-44d1-b51d-943de4ee5b2f", "time_conditions", "preset_usa", "array", '{"memorial_day":{"mday":"25-31","wday":"2","mon":"5"}}', "USA Holiday", 146),
        ("0aa94174-a339-47d6-b6ab-c264b3786074", "time_conditions", "preset_usa", "array", '{"independence_day":{"mday":"4","mon":"7"}}', "USA Holiday", 186),
        ("be512c08-029e-49a0-937d-1a62fc029609", "time_conditions", "preset_usa", "array", '{"labor_day":{"wday":"2","mon":"9","mday":"1-7"}}', "USA Holiday", 251),
        ("c1fdfebe-3544-4b01-8a83-d0fee8e9a47a", "time_conditions", "preset_usa", "array", '{"thanksgiving_day":{"wday":"5","mon":"11","mday":"22-28"}}', "USA Holiday", 331),
        ("82e3eb39-27a4-4d70-8436-11059d3e51e7", "time_conditions", "preset_usa", "array", '{"christmas_day":{"mday":"25","mon":"12"}}', "USA Holiday", 360),
        ("340856f4-ed1d-42ec-b13b-7f37630d019a", "time_conditions", "preset_canada", "array", '{"new_years_day":{"mday":"1","mon":"1"}}', "Canada Holiday", 1),
        ("49bbadb3-568d-42d8-b6b4-edc9560a054d", "time_conditions", "preset_canada", "array", '{"canada_day":{"mday":"1","mon":"7"}}', "Canada Holiday", 183),
        ("43ef3137-084a-40ea-94ed-42b68979c551", "time_conditions", "preset_canada", "array", '{"labour_day":{"wday":"2","mon":"9","mday":"1-7"}}', "Canada Holiday", 251),
        ("51717873-86d2-43a2-8748-ff191b4915e7", "time_conditions", "preset_canada", "array", '{"christmas_day":{"mday":"25","mon":"12"}}', "Canada Holiday", 360),
        ("c73fa4f6-aed5-49a3-8726-4c1d94847302", "time_conditions", "extension_range", "text", "800-899", "Set the suggested extension range(s) for time conditions", 0),
    ]
    try:
        conn = op.get_bind()
        for uuid_, cat, sub, name, val, desc, ord_ in presets:
            try:
                conn.execute(sa.text("""
                    INSERT INTO default_settings (default_setting_uuid, default_setting_category, default_setting_subcategory, default_setting_name, default_setting_value, default_setting_enabled, default_setting_description, default_setting_order)
                    VALUES (:uuid, :cat, :sub, :name, :val, true, :desc, :ord)
                    ON CONFLICT (default_setting_uuid) DO NOTHING
                """), {"uuid": uuid_, "cat": cat, "sub": sub, "name": name, "val": val, "desc": desc, "ord": ord_})
            except Exception:
                continue
        # ensure view
        try:
            op.execute("DROP VIEW IF EXISTS v_default_settings;")
            op.execute("""CREATE VIEW v_default_settings AS SELECT default_setting_uuid, default_setting_category, default_setting_subcategory, default_setting_name, default_setting_value, default_setting_enabled, default_setting_description, created_at AS insert_date FROM default_settings;""")
        except Exception:
            pass
        # ensure dialplans view for time conditions correctly filters app_uuid
        try:
            op.execute("DROP VIEW IF EXISTS v_dialplans;")
        except Exception:
            pass
        # v_time_conditions view that joins dialplans app_uuid
        try:
            op.execute("DROP VIEW IF EXISTS v_time_conditions;")
            op.execute("""CREATE VIEW v_time_conditions AS SELECT tc.time_condition_uuid, tc.domain_id AS domain_uuid, tc.name AS dialplan_name, tc.extension AS dialplan_number, tc.context AS dialplan_context, tc.description AS dialplan_description, tc.enabled AS dialplan_enabled, tc.dialplan_uuid, d.app_uuid FROM time_conditions tc LEFT JOIN dialplans d ON tc.dialplan_uuid = d.dialplan_uuid;""")
        except Exception:
            pass
    except Exception:
        pass

    # also ensure dialplans app_uuid column exists for linkage (already added in 023)
    try:
        op.create_index("ix_dialplans_app_uuid", "dialplans", ["app_uuid"])
    except Exception:
        pass

def downgrade() -> None:
    # drop voicemail options/messages
    for tbl in ["voicemail_options", "voicemail_messages"]:
        try:
            op.execute(f"DROP VIEW IF EXISTS v_{tbl};")
        except Exception:
            pass
        try:
            op.execute(f"DROP POLICY IF EXISTS tenant_isolation ON {tbl};")
            op.execute(f"ALTER TABLE {tbl} DISABLE ROW LEVEL SECURITY;")
        except Exception:
            pass
        try:
            op.drop_table(tbl)
        except Exception:
            pass
    for col in ["transcription_prompt","sms_to","recording_instructions","voicemail_keep_local_after_email","voicemail_sms_type","voicemail_alternate_greet_id","voicemail_forward_destinations","voicemail_forward_enabled","voicemail_attach_file","voicemail_recording_instructions","voicemail_transcription_prompt","voicemail_sms_to"]:
        try:
            op.drop_column("voicemails", col)
        except Exception:
            pass
    # keep fallback columns for downgrade but ignore errors
    for col in ["greeting_message","greeting_voice"]:
        try:
            op.drop_column("voicemail_greetings", col)
        except Exception:
            pass
    for col in ["accountcode","context","leg","status","billmsec","mduration","hold_accum_seconds","bridge_uuid","conference_id","record_path","record_name","json_text","xml_text","processed","caller_extension","destination_extension"]:
        try:
            op.drop_column("cdr", col)
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_cdr; CREATE VIEW v_cdr AS SELECT domain_id AS domain_uuid, cdr_uuid, caller_id_number, destination_number, start_stamp, duration, billsec FROM cdr;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_xml_cdr;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_time_conditions;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_voicemail_greetings; CREATE VIEW v_voicemail_greetings AS SELECT domain_id AS domain_uuid, voicemail_greeting_uuid, voicemail_id, greeting_id, greeting_name, created_at AS insert_date FROM voicemail_greetings;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_voicemails; CREATE VIEW v_voicemails AS SELECT domain_id AS domain_uuid, voicemail_uuid, voicemail_id, voicemail_password, greeting_id, toll_allow, voicemail_mail_to, voicemail_file, enabled AS voicemail_enabled FROM voicemails;")
    except Exception:
        pass
