"""combined fix: contacts child tables, number_translation_details, phrase_details, email_queue_attachments, modules order

Revision ID: 034_combined_fix
Revises: 033_fax_recordings_fix
Create Date: 2026-08-25
"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "034_combined_fix"
down_revision: Union[str, None] = "033_fax_recordings_fix"
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


def upgrade() -> None:
    # --- contacts child tables ---
    tables = [
        (
            "contact_addresses",
            [
                sa.Column("contact_address_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("address_type", sa.String(length=32), nullable=True),
                sa.Column("address_label", sa.String(length=64), nullable=True),
                sa.Column("address_primary", sa.Boolean(), nullable=False, server_default=sa.text("false")),
                sa.Column("address_street", sa.String(length=255), nullable=True),
                sa.Column("address_extended", sa.String(length=255), nullable=True),
                sa.Column("address_community", sa.String(length=128), nullable=True),
                sa.Column("address_locality", sa.String(length=128), nullable=True),
                sa.Column("address_region", sa.String(length=128), nullable=True),
                sa.Column("address_postal_code", sa.String(length=32), nullable=True),
                sa.Column("address_country", sa.String(length=64), nullable=True),
                sa.Column("address_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_address_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_phones",
            [
                sa.Column("contact_phone_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("phone_label", sa.String(length=64), nullable=True),
                sa.Column("phone_number", sa.String(length=32), nullable=False),
                sa.Column("phone_extension", sa.String(length=32), nullable=True),
                sa.Column("phone_type", sa.String(length=32), nullable=True),
                sa.Column("phone_primary", sa.Boolean(), nullable=False, server_default=sa.text("false")),
                sa.Column("phone_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_phone_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_emails",
            [
                sa.Column("contact_email_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("email_label", sa.String(length=64), nullable=True),
                sa.Column("email_address", sa.String(length=255), nullable=False),
                sa.Column("email_primary", sa.Boolean(), nullable=False, server_default=sa.text("false")),
                sa.Column("email_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_email_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_urls",
            [
                sa.Column("contact_url_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("url_type", sa.String(length=32), nullable=True),
                sa.Column("url_label", sa.String(length=64), nullable=True),
                sa.Column("url_address", sa.String(length=512), nullable=False),
                sa.Column("url_primary", sa.Boolean(), nullable=False, server_default=sa.text("false")),
                sa.Column("url_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_url_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_notes",
            [
                sa.Column("contact_note_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("contact_note", sa.Text(), nullable=False),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_note_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_groups",
            [
                sa.Column("contact_group_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("group_uuid", sa.Uuid(), nullable=False),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["group_uuid"], ["groups.group_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_group_uuid"),
            ],
            ["domain_id", "contact_uuid", "group_uuid"],
        ),
        (
            "contact_users",
            [
                sa.Column("contact_user_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("user_uuid", sa.Uuid(), nullable=False),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_user_uuid"),
            ],
            ["domain_id", "contact_uuid", "user_uuid"],
        ),
        (
            "contact_settings",
            [
                sa.Column("contact_setting_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("contact_setting_category", sa.String(length=64), nullable=True),
                sa.Column("contact_setting_subcategory", sa.String(length=64), nullable=True),
                sa.Column("contact_setting_name", sa.String(length=128), nullable=False),
                sa.Column("contact_setting_value", sa.String(length=512), nullable=True),
                sa.Column("contact_setting_order", sa.Integer(), nullable=True),
                sa.Column("contact_setting_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
                sa.Column("contact_setting_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_setting_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_relations",
            [
                sa.Column("contact_relation_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("relation_label", sa.String(length=64), nullable=True),
                sa.Column("relation_contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["relation_contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_relation_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_times",
            [
                sa.Column("contact_time_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("user_uuid", sa.Uuid(), nullable=True),
                sa.Column("time_start", sa.DateTime(timezone=True), nullable=True),
                sa.Column("time_stop", sa.DateTime(timezone=True), nullable=True),
                sa.Column("time_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["user_uuid"], ["users.user_uuid"], ondelete="SET NULL"),
                sa.PrimaryKeyConstraint("contact_time_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
        (
            "contact_attachments",
            [
                sa.Column("contact_attachment_uuid", sa.Uuid(), nullable=False),
                sa.Column("domain_id", sa.Uuid(), nullable=False),
                sa.Column("contact_uuid", sa.Uuid(), nullable=False),
                sa.Column("attachment_primary", sa.Boolean(), nullable=False, server_default=sa.text("false")),
                sa.Column("attachment_filename", sa.String(length=255), nullable=True),
                sa.Column("attachment_content", sa.Text(), nullable=True),
                sa.Column("attachment_description", sa.String(length=512), nullable=True),
                sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
                sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
                sa.ForeignKeyConstraint(["contact_uuid"], ["contacts.contact_uuid"], ondelete="CASCADE"),
                sa.PrimaryKeyConstraint("contact_attachment_uuid"),
            ],
            ["domain_id", "contact_uuid"],
        ),
    ]

    for tbl_name, cols, idx_cols in tables:
        try:
            op.create_table(tbl_name, *cols)
        except Exception:
            pass
        for col in idx_cols:
            try:
                op.create_index(f"ix_{tbl_name}_{col}", tbl_name, [col])
            except Exception:
                pass
        _rls(tbl_name)
        # legacy view
        try:
            v = f"v_{tbl_name}"
            op.execute(f"DROP VIEW IF EXISTS {v};")
            # generic view maps domain_id -> domain_uuid
            op.execute(f"CREATE VIEW {v} AS SELECT domain_id AS domain_uuid, * FROM {tbl_name};")
        except Exception:
            pass

    # --- number_translation_details ---
    try:
        op.create_table(
            "number_translation_details",
            sa.Column("number_translation_detail_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("number_translation_uuid", sa.Uuid(), nullable=False),
            sa.Column("number_translation_detail_regex", sa.String(length=512), nullable=False),
            sa.Column("number_translation_detail_replace", sa.String(length=512), nullable=False),
            sa.Column("number_translation_detail_order", sa.Integer(), nullable=False, server_default=sa.text("0")),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["number_translation_uuid"], ["number_translations.number_translation_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("number_translation_detail_uuid"),
        )
    except Exception:
        pass
    for col in ["domain_id", "number_translation_uuid"]:
        try:
            op.create_index(f"ix_number_translation_details_{col}", "number_translation_details", [col])
        except Exception:
            pass
    _rls("number_translation_details")
    try:
        op.execute("DROP VIEW IF EXISTS v_number_translation_details;")
        op.execute("CREATE VIEW v_number_translation_details AS SELECT domain_id AS domain_uuid, * FROM number_translation_details;")
    except Exception:
        pass
    # refresh parent view
    try:
        op.execute("DROP VIEW IF EXISTS v_number_translations;")
        op.execute("CREATE VIEW v_number_translations AS SELECT domain_id AS domain_uuid, number_translation_uuid, number_translation_name, number_translation_regex, number_translation_replace, number_translation_enabled, number_translation_description, created_at AS insert_date FROM number_translations;")
    except Exception:
        pass

    # --- phrase_details add 5 columns ---
    for col, typ in [
        ("phrase_detail_group", sa.Integer()),
        ("phrase_detail_tag", sa.String(length=64)),
        ("phrase_detail_pattern", sa.String(length=128)),
        ("phrase_detail_method", sa.String(length=32)),
        ("phrase_detail_type", sa.String(length=32)),
    ]:
        try:
            op.add_column("phrase_details", sa.Column(col, typ, nullable=True))
        except Exception:
            pass
    try:
        op.execute("DROP VIEW IF EXISTS v_phrase_details;")
        op.execute("CREATE VIEW v_phrase_details AS SELECT domain_id AS domain_uuid, phrase_detail_uuid, phrase_uuid, phrase_detail_group, phrase_detail_tag, phrase_detail_pattern, phrase_detail_function, phrase_detail_data, phrase_detail_method, phrase_detail_type, phrase_detail_order, created_at AS insert_date FROM phrase_details;")
    except Exception:
        pass

    # --- email_queue_attachments ---
    try:
        op.create_table(
            "email_queue_attachments",
            sa.Column("email_queue_attachment_uuid", sa.Uuid(), nullable=False),
            sa.Column("domain_id", sa.Uuid(), nullable=False),
            sa.Column("email_queue_uuid", sa.Uuid(), nullable=False),
            sa.Column("email_attachment_mime_type", sa.String(length=128), nullable=True),
            sa.Column("email_attachment_type", sa.String(length=32), nullable=True),
            sa.Column("email_attachment_path", sa.String(length=512), nullable=True),
            sa.Column("email_attachment_name", sa.String(length=255), nullable=True),
            sa.Column("email_attachment_base64", sa.Text(), nullable=True),
            sa.Column("email_attachment_cid", sa.String(length=255), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["domain_id"], ["domains.domain_id"], ondelete="CASCADE"),
            sa.ForeignKeyConstraint(["email_queue_uuid"], ["email_queue.email_queue_uuid"], ondelete="CASCADE"),
            sa.PrimaryKeyConstraint("email_queue_attachment_uuid"),
        )
    except Exception:
        pass
    for col in ["domain_id", "email_queue_uuid"]:
        try:
            op.create_index(f"ix_email_queue_attachments_{col}", "email_queue_attachments", [col])
        except Exception:
            pass
    _rls("email_queue_attachments")
    try:
        op.execute("DROP VIEW IF EXISTS v_email_queue_attachments;")
        op.execute("CREATE VIEW v_email_queue_attachments AS SELECT domain_id AS domain_uuid, * FROM email_queue_attachments;")
    except Exception:
        pass

    # --- modules: add module_order ---
    try:
        op.add_column("modules", sa.Column("module_order", sa.Integer(), nullable=True))
    except Exception:
        pass
    try:
        op.create_index("ix_modules_order", "modules", ["module_order"])
    except Exception:
        pass

    # seed permissions for new tables (idempotent)
    try:
        op.execute(
            """
            INSERT INTO permissions (permission_uuid, name) VALUES
            (gen_random_uuid(), 'contact_address_view'),
            (gen_random_uuid(), 'contact_address_add'),
            (gen_random_uuid(), 'contact_address_edit'),
            (gen_random_uuid(), 'contact_address_delete'),
            (gen_random_uuid(), 'contact_phone_view'),
            (gen_random_uuid(), 'contact_phone_add'),
            (gen_random_uuid(), 'contact_phone_edit'),
            (gen_random_uuid(), 'contact_phone_delete'),
            (gen_random_uuid(), 'contact_email_view'),
            (gen_random_uuid(), 'contact_email_add'),
            (gen_random_uuid(), 'contact_email_edit'),
            (gen_random_uuid(), 'contact_email_delete'),
            (gen_random_uuid(), 'contact_url_view'),
            (gen_random_uuid(), 'contact_url_add'),
            (gen_random_uuid(), 'contact_url_edit'),
            (gen_random_uuid(), 'contact_url_delete'),
            (gen_random_uuid(), 'number_translation_detail_view'),
            (gen_random_uuid(), 'number_translation_detail_add'),
            (gen_random_uuid(), 'number_translation_detail_edit'),
            (gen_random_uuid(), 'number_translation_detail_delete'),
            (gen_random_uuid(), 'email_queue_attachment_view'),
            (gen_random_uuid(), 'email_queue_attachment_add'),
            (gen_random_uuid(), 'email_queue_attachment_edit'),
            (gen_random_uuid(), 'email_queue_attachment_delete')
            ON CONFLICT (name) DO NOTHING;
            """
        )
        op.execute(
            """
            INSERT INTO group_permissions (id, group_uuid, permission_uuid)
            SELECT gen_random_uuid(), g.group_uuid, p.permission_uuid
            FROM groups g, permissions p
            WHERE g.name='superadmin' AND p.name IN (
                'contact_address_view','contact_address_add','contact_address_edit','contact_address_delete',
                'contact_phone_view','contact_phone_add','contact_phone_edit','contact_phone_delete',
                'contact_email_view','contact_email_add','contact_email_edit','contact_email_delete',
                'contact_url_view','contact_url_add','contact_url_edit','contact_url_delete',
                'number_translation_detail_view','number_translation_detail_add','number_translation_detail_edit','number_translation_detail_delete',
                'email_queue_attachment_view','email_queue_attachment_add','email_queue_attachment_edit','email_queue_attachment_delete'
            )
            AND NOT EXISTS (SELECT 1 FROM group_permissions gp2 WHERE gp2.group_uuid=g.group_uuid AND gp2.permission_uuid=p.permission_uuid);
            """
        )
    except Exception:
        pass


def downgrade() -> None:
    # modules order
    try:
        op.drop_index("ix_modules_order", table_name="modules")
    except Exception:
        pass
    try:
        op.drop_column("modules", "module_order")
    except Exception:
        pass
    # email attachments
    try:
        op.execute("DROP VIEW IF EXISTS v_email_queue_attachments;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON email_queue_attachments;")
        op.execute("ALTER TABLE email_queue_attachments DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("email_queue_attachments")
    except Exception:
        pass
    # phrase_details columns
    try:
        op.execute("DROP VIEW IF EXISTS v_phrase_details;")
    except Exception:
        pass
    for col in ["phrase_detail_type", "phrase_detail_method", "phrase_detail_pattern", "phrase_detail_tag", "phrase_detail_group"]:
        try:
            op.drop_column("phrase_details", col)
        except Exception:
            pass
    # number_translation_details
    try:
        op.execute("DROP VIEW IF EXISTS v_number_translation_details;")
        op.execute("DROP VIEW IF EXISTS v_number_translations;")
        op.execute("CREATE VIEW v_number_translations AS SELECT domain_id AS domain_uuid, number_translation_uuid, number_translation_name, number_translation_regex, number_translation_replace, number_translation_enabled, number_translation_description, created_at AS insert_date FROM number_translations;")
    except Exception:
        pass
    try:
        op.execute("DROP POLICY IF EXISTS tenant_isolation ON number_translation_details;")
        op.execute("ALTER TABLE number_translation_details DISABLE ROW LEVEL SECURITY;")
    except Exception:
        pass
    try:
        op.drop_table("number_translation_details")
    except Exception:
        pass
    # contacts child tables
    for tbl in ["contact_attachments", "contact_times", "contact_relations", "contact_settings", "contact_users", "contact_groups", "contact_notes", "contact_urls", "contact_emails", "contact_phones", "contact_addresses"]:
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
