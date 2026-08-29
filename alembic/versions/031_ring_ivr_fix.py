"""ring_groups + ivr_menus parity fix

Revision ID: 031_ring_ivr_fix
Revises: 029_vars_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "031_ring_ivr_fix"
down_revision: Union[str, None] = "030_gateways_dialplans_fix"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ring_groups — add nullable columns for legacy parity (15+)
    for col, typ in [
        ("greeting", sa.String(length=256)),
        ("exit_key", sa.String(length=8)),
        ("call_timeout", sa.Integer()),
        ("caller_id_name", sa.String(length=128)),
        ("caller_id_number", sa.String(length=32)),
        ("cid_name_prefix", sa.String(length=32)),
        ("cid_number_prefix", sa.String(length=32)),
        ("timeout_app", sa.String(length=128)),
        ("timeout_data", sa.String(length=512)),
        ("distinctive_ring", sa.String(length=128)),
        ("ringback", sa.String(length=128)),
        ("call_screen_enabled", sa.Boolean()),
        ("call_forward_enabled", sa.Boolean()),
        ("follow_me_enabled", sa.Boolean()),
        ("missed_call_app", sa.String(length=128)),
        ("missed_call_data", sa.String(length=512)),
        ("context", sa.String(length=128)),
        ("dialplan_uuid", sa.Uuid()),
        ("forward_toll_allow", sa.String(length=128)),
    ]:
        try:
            op.add_column("ring_groups", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    # FK for dialplan_uuid
    try:
        op.create_foreign_key("fk_ring_groups_dialplan", "ring_groups", "dialplans", ["dialplan_uuid"], ["dialplan_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    # indices for ring_groups new cols
    try:
        op.create_index("ix_ring_groups_context", "ring_groups", ["context"])
    except Exception:
        pass
    try:
        op.create_index("ix_ring_groups_dialplan_uuid", "ring_groups", ["dialplan_uuid"])
    except Exception:
        pass

    # ring_group_users table (user mapping)
    try:
        op.create_table(
            "ring_group_users",
            sa.Column("ring_group_user_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("ring_group_uuid", sa.Uuid(), nullable=False),
            sa.Column("user_uuid", sa.Uuid(), nullable=False),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["ring_group_uuid"], ["ring_groups.ring_group_uuid"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("ring_group_user_uuid"),
        )
    except Exception:
        pass
    try:
        op.create_index("ix_ring_group_users_domain_id", "ring_group_users", ["domain_id"])
    except Exception:
        pass
    try:
        op.create_index("ix_ring_group_users_ring_group_uuid", "ring_group_users", ["ring_group_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_ring_group_users_user_uuid", "ring_group_users", ["user_uuid"])
    except Exception:
        pass
    # RLS for ring_group_users
    try:
        op.execute("ALTER TABLE ring_group_users ENABLE ROW LEVEL SECURITY;")
        op.execute("ALTER TABLE ring_group_users FORCE ROW LEVEL SECURITY;")
        op.execute(
            """CREATE POLICY tenant_isolation ON ring_group_users
            USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '')
            WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid OR current_setting('app.domain_uuid', true) = '');"""
        )
    except Exception:
        pass
    # legacy compat view v_ring_groups
    try:
        op.execute("DROP VIEW IF EXISTS v_ring_groups;")
        op.execute(
            """CREATE VIEW v_ring_groups AS SELECT
            domain_id AS domain_uuid, ring_group_uuid, name AS ring_group_name, extension AS ring_group_extension,
            greeting AS ring_group_greeting, exit_key AS ring_group_exit_key, call_timeout AS ring_group_call_timeout,
            ring_group_forward_destination, ring_group_forward_enabled, caller_id_name AS ring_group_caller_id_name,
            caller_id_number AS ring_group_caller_id_number, cid_name_prefix AS ring_group_cid_name_prefix,
            cid_number_prefix AS ring_group_cid_number_prefix, strategy AS ring_group_strategy,
            timeout_app AS ring_group_timeout_app, timeout_data AS ring_group_timeout_data,
            distinctive_ring AS ring_group_distinctive_ring, ringback AS ring_group_ringback,
            call_screen_enabled AS ring_group_call_screen_enabled, call_forward_enabled AS ring_group_call_forward_enabled,
            follow_me_enabled AS ring_group_follow_me_enabled, missed_call_app AS ring_group_missed_call_app,
            missed_call_data AS ring_group_missed_call_data, context AS ring_group_context, enabled AS ring_group_enabled,
            description AS ring_group_description, dialplan_uuid, forward_toll_allow AS ring_group_forward_toll_allow,
            created_at AS insert_date, updated_at AS update_date FROM ring_groups;"""
        )
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_ring_group_users;")
        op.execute("CREATE VIEW v_ring_group_users AS SELECT domain_id AS domain_uuid, ring_group_user_uuid, ring_group_uuid, user_uuid, created_at AS insert_date FROM ring_group_users;")
    except Exception:
        pass

    # ivr_menus — add nullable columns
    for col, typ in [
        ("dialplan_uuid", sa.Uuid()),
        ("parent_uuid", sa.Uuid()),
        ("language", sa.String(length=32)),
        ("dialect", sa.String(length=32)),
        ("voice", sa.String(length=32)),
        ("pin_number", sa.String(length=32)),
        ("ringback", sa.String(length=128)),
        ("cid_prefix", sa.String(length=32)),
        ("context", sa.String(length=128)),
        ("direct_dial", sa.Boolean()),
        ("exit_app", sa.String(length=128)),
        ("exit_data", sa.String(length=512)),
    ]:
        try:
            op.add_column("ivr_menus", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    try:
        op.create_foreign_key("fk_ivr_menus_dialplan", "ivr_menus", "dialplans", ["dialplan_uuid"], ["dialplan_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_foreign_key("fk_ivr_menus_parent", "ivr_menus", "ivr_menus", ["parent_uuid"], ["ivr_menu_uuid"], ondelete="SET NULL")
    except Exception:
        pass
    try:
        op.create_index("ix_ivr_menus_dialplan_uuid", "ivr_menus", ["dialplan_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_ivr_menus_parent_uuid", "ivr_menus", ["parent_uuid"])
    except Exception:
        pass
    try:
        op.create_index("ix_ivr_menus_context", "ivr_menus", ["context"])
    except Exception:
        pass
    # refresh v_ivr_menus view
    try:
        op.execute("DROP VIEW IF EXISTS v_ivr_menus;")
        op.execute(
            """CREATE VIEW v_ivr_menus AS SELECT
            domain_id AS domain_uuid, ivr_menu_uuid, dialplan_uuid, name AS ivr_menu_name, extension AS ivr_menu_extension,
            parent_uuid AS ivr_menu_parent_uuid, language AS ivr_menu_language, dialect AS ivr_menu_dialect, voice AS ivr_menu_voice,
            greet_long AS ivr_menu_greet_long, greet_short AS ivr_menu_greet_short, invalid_sound AS ivr_menu_invalid_sound,
            exit_sound AS ivr_menu_exit_sound, pin_number AS ivr_menu_pin_number, confirm_macro AS ivr_menu_confirm_macro,
            confirm_key AS ivr_menu_confirm_key, tts_engine AS ivr_menu_tts_engine, tts_voice AS ivr_menu_tts_voice,
            confirm_attempts AS ivr_menu_confirm_attempts, timeout AS ivr_menu_timeout, exit_app AS ivr_menu_exit_app,
            exit_data AS ivr_menu_exit_data, inter_digit_timeout AS ivr_menu_inter_digit_timeout,
            max_failures AS ivr_menu_max_failures, max_timeouts AS ivr_menu_max_timeouts, digit_len AS ivr_menu_digit_len,
            direct_dial AS ivr_menu_direct_dial, ringback AS ivr_menu_ringback, cid_prefix AS ivr_menu_cid_prefix,
            context AS ivr_menu_context, enabled AS ivr_menu_enabled, description AS ivr_menu_description,
            created_at AS insert_date, updated_at AS update_date FROM ivr_menus;"""
        )
    except Exception:
        pass


def downgrade() -> None:
    try:
        op.execute("DROP VIEW IF EXISTS v_ivr_menus;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_ring_group_users;")
    except Exception:
        pass
    try:
        op.execute("DROP VIEW IF EXISTS v_ring_groups;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON ring_group_users;")
        op.execute("ALTER TABLE ring_group_users DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("ring_group_users")
    except Exception:
        pass
    for col in ["exit_data", "exit_app", "direct_dial", "context", "cid_prefix", "ringback", "pin_number", "voice", "dialect", "language", "parent_uuid", "dialplan_uuid"]:
        try:
            op.drop_column("ivr_menus", col)
        except Exception:
            pass
    try:
        op.drop_constraint("fk_ivr_menus_parent", "ivr_menus", type_="foreignkey")
    except Exception:
        pass
    try:
        op.drop_constraint("fk_ivr_menus_dialplan", "ivr_menus", type_="foreignkey")
    except Exception:
        pass
    for col in ["forward_toll_allow", "dialplan_uuid", "context", "missed_call_data", "missed_call_app", "follow_me_enabled", "call_forward_enabled", "call_screen_enabled", "ringback", "distinctive_ring", "timeout_data", "timeout_app", "cid_number_prefix", "cid_name_prefix", "caller_id_number", "caller_id_name", "call_timeout", "exit_key", "greeting"]:
        try:
            op.drop_column("ring_groups", col)
        except Exception:
            pass
    try:
        op.drop_constraint("fk_ring_groups_dialplan", "ring_groups", type_="foreignkey")
    except Exception:
        pass
