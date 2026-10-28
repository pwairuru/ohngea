//! Postgres Migrator — mirrors alembic chain (postgres dialect).
//!
//! Ground truth:
//! - `app/db.py:12-18` pool parity (size 5, timeout 30, pre_ping)
//! - `app/db.py:58-82` `set_tenant` GUC `app.domain_uuid`
//! - `alembic/versions/*.py` tables/indexes/RLS/views
//! - `alembic/versions/043_tenant_rls_all.py` STRICT/NULLABLE policy lists
//!
//! Postgres types: UUID PK, TIMESTAMPTZ, FK CASCADE, RLS FORCE + policies.
//! SQLite accepts same DDL (typeless) — RLS/views run on Postgres only.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(M20260101_000001PgParity)]
    }
}

struct M20260101_000001PgParity;

impl MigrationName for M20260101_000001PgParity {
    fn name(&self) -> &str {
        "m20260101_000001_pg_parity"
    }
}

const EXT_PGCRYPTO: &str = "CREATE EXTENSION IF NOT EXISTS pgcrypto";

/// Core tables with postgres types. Mirrors alembic 001 + batch deltas.
/// Column-for-column with `crate::entities` (which mirror Python models).
const DDL_TABLES: &[&str] = &[
    // domains — global, no RLS
    "CREATE TABLE IF NOT EXISTS domains (
      domain_id UUID PRIMARY KEY,
      domain_name VARCHAR(255) NOT NULL UNIQUE,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_domains_domain_name ON domains (domain_name)",
    // vars (029 adds var_command/var_hostname)
    "CREATE TABLE IF NOT EXISTS vars (
      var_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      var_category VARCHAR(64) NOT NULL,
      var_name VARCHAR(128) NOT NULL,
      var_value VARCHAR(512) NOT NULL,
      var_enabled BOOLEAN NOT NULL,
      var_description TEXT,
      var_order INTEGER,
      var_command VARCHAR(512),
      var_hostname VARCHAR(255),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL,
      UNIQUE (domain_id, var_name, var_category)
    )",
    "CREATE INDEX IF NOT EXISTS ix_vars_domain_id ON vars (domain_id)",
    // sofia global — no domain_id, no RLS
    "CREATE TABLE IF NOT EXISTS sofia_global_settings (
      sofia_global_setting_uuid UUID PRIMARY KEY,
      global_setting_name VARCHAR(128) NOT NULL UNIQUE,
      global_setting_value VARCHAR(512) NOT NULL,
      global_setting_enabled BOOLEAN NOT NULL,
      global_setting_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    // default_settings (global provision defaults)
    "CREATE TABLE IF NOT EXISTS default_settings (
      default_setting_uuid UUID PRIMARY KEY,
      default_setting_category VARCHAR(64) NOT NULL,
      default_setting_subcategory VARCHAR(64) NOT NULL,
      default_setting_name VARCHAR(64) NOT NULL,
      default_setting_value TEXT,
      default_setting_enabled BOOLEAN NOT NULL,
      default_setting_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_default_settings_category_sub
     ON default_settings (default_setting_category, default_setting_subcategory)",
    "CREATE TABLE IF NOT EXISTS domain_settings (
      id UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      category VARCHAR(64) NOT NULL,
      subcategory VARCHAR(64) NOT NULL,
      \"key\" VARCHAR(64) NOT NULL,
      value VARCHAR(1024) NOT NULL,
      enabled BOOLEAN NOT NULL,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_domain_settings_domain ON domain_settings (domain_id)",
    // extensions — full parity columns
    "CREATE TABLE IF NOT EXISTS extensions (
      extension_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension VARCHAR(32) NOT NULL,
      number_alias VARCHAR(32),
      password VARCHAR(64),
      accountcode VARCHAR(64),
      effective_caller_id_name VARCHAR(128) NOT NULL,
      effective_caller_id_number VARCHAR(32),
      outbound_caller_id_name VARCHAR(128),
      outbound_caller_id_number VARCHAR(32),
      emergency_caller_id_name VARCHAR(128),
      emergency_caller_id_number VARCHAR(32),
      directory_first_name VARCHAR(128),
      directory_last_name VARCHAR(128),
      directory_visible BOOLEAN NOT NULL DEFAULT FALSE,
      directory_exten_visible BOOLEAN NOT NULL DEFAULT FALSE,
      max_registrations VARCHAR(32),
      limit_max VARCHAR(32),
      limit_destination VARCHAR(128),
      missed_call_app VARCHAR(32),
      missed_call_data VARCHAR(128),
      user_context VARCHAR(128),
      toll_allow VARCHAR(128),
      call_timeout INTEGER,
      call_group VARCHAR(32),
      call_screen_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      user_record VARCHAR(32),
      hold_music VARCHAR(128),
      auth_acl VARCHAR(128),
      cidr VARCHAR(512),
      sip_force_contact VARCHAR(128),
      sip_force_expires INTEGER,
      nibble_account VARCHAR(128),
      mwi_account VARCHAR(128),
      sip_bypass_media VARCHAR(32),
      absolute_codec_string VARCHAR(256),
      force_ping BOOLEAN NOT NULL DEFAULT FALSE,
      dial_string VARCHAR(1024),
      do_not_disturb BOOLEAN NOT NULL DEFAULT FALSE,
      forward_all_destination VARCHAR(128),
      forward_all_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      forward_busy_destination VARCHAR(128),
      forward_busy_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      forward_no_answer_destination VARCHAR(128),
      forward_no_answer_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      forward_user_not_registered_destination VARCHAR(128),
      forward_user_not_registered_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      follow_me_uuid UUID,
      follow_me_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      extension_language VARCHAR(32),
      extension_dialect VARCHAR(32),
      extension_voice VARCHAR(32),
      extension_type VARCHAR(32),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_extensions_domain ON extensions (domain_id)",
    "CREATE INDEX IF NOT EXISTS ix_extensions_domain_extension ON extensions (domain_id, extension)",
    // dialplans + details
    "CREATE TABLE IF NOT EXISTS dialplans (
      dialplan_uuid UUID PRIMARY KEY,
      name VARCHAR(128) NOT NULL,
      context VARCHAR(128) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      description TEXT,
      dialplan_continue BOOLEAN NOT NULL DEFAULT FALSE,
      dialplan_order INTEGER,
      hostname VARCHAR(128),
      app_uuid VARCHAR(64),
      dialplan_number VARCHAR(128),
      dialplan_destination BOOLEAN,
      dialplan_xml TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_dialplans_domain_name ON dialplans (domain_id, name)",
    "CREATE TABLE IF NOT EXISTS dialplan_details (
      detail_uuid UUID PRIMARY KEY,
      dialplan_uuid UUID NOT NULL REFERENCES dialplans (dialplan_uuid) ON DELETE CASCADE,
      tag VARCHAR(32) NOT NULL,
      \"type\" VARCHAR(64) NOT NULL,
      data VARCHAR(512) NOT NULL,
      \"order\" INTEGER NOT NULL DEFAULT 0,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      dialplan_detail_break VARCHAR(32),
      dialplan_detail_inline VARCHAR(32),
      dialplan_detail_group INTEGER,
      detail_enabled BOOLEAN,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_dialplan_details_parent ON dialplan_details (dialplan_uuid)",
    // gateways
    "CREATE TABLE IF NOT EXISTS gateways (
      gateway_uuid UUID PRIMARY KEY,
      gateway VARCHAR(64) NOT NULL,
      username VARCHAR(64),
      password VARCHAR(64),
      proxy VARCHAR(128),
      \"register\" BOOLEAN NOT NULL DEFAULT TRUE,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      distinct_to BOOLEAN,
      auth_username VARCHAR(64),
      realm VARCHAR(128),
      from_user VARCHAR(64),
      from_domain VARCHAR(128),
      register_proxy VARCHAR(128),
      outbound_proxy VARCHAR(128),
      expire_seconds INTEGER,
      register_transport VARCHAR(32),
      contact_params VARCHAR(256),
      retry_seconds INTEGER,
      extension VARCHAR(64),
      ping VARCHAR(32),
      ping_min INTEGER,
      ping_max INTEGER,
      contact_in_ping BOOLEAN,
      caller_id_in_from BOOLEAN,
      supress_cng BOOLEAN,
      sip_cid_type VARCHAR(32),
      codec_prefs VARCHAR(256),
      channels INTEGER,
      extension_in_contact VARCHAR(64),
      context VARCHAR(64),
      profile VARCHAR(64),
      hostname VARCHAR(255),
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_gateways_domain ON gateways (domain_id)",
    // sip profiles + domains + settings
    "CREATE TABLE IF NOT EXISTS sip_profiles (
      sip_profile_uuid UUID PRIMARY KEY,
      name VARCHAR(64) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      sip_port INTEGER NOT NULL DEFAULT 5060,
      sip_ip VARCHAR(64),
      ext_sip_ip VARCHAR(64),
      ext_rtp_ip VARCHAR(64),
      realm VARCHAR(128),
      context VARCHAR(128),
      challenge_realm VARCHAR(128),
      wss_port INTEGER,
      wss_binding VARCHAR(128),
      tls_cert_path VARCHAR(256),
      tls_key_path VARCHAR(256),
      sip_secure BOOLEAN,
      rtp_ip VARCHAR(64),
      rtp_port_range VARCHAR(32),
      hold_music VARCHAR(128),
      apply_inbound_acl VARCHAR(128),
      local_network_acl VARCHAR(128),
      record_path VARCHAR(256),
      record_template VARCHAR(128),
      description VARCHAR(512),
      hostname VARCHAR(128),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_sip_profiles_domain_name ON sip_profiles (domain_id, name)",
    "CREATE TABLE IF NOT EXISTS sip_profile_domains (
      sip_profile_domain_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      sip_profile_uuid UUID NOT NULL REFERENCES sip_profiles (sip_profile_uuid) ON DELETE CASCADE,
      domain_name VARCHAR(128) NOT NULL,
      domain_alias VARCHAR(16),
      domain_parse VARCHAR(16),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_sip_profile_domains_parent ON sip_profile_domains (sip_profile_uuid)",
    "CREATE TABLE IF NOT EXISTS sip_profile_settings (
      sip_profile_setting_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      sip_profile_uuid UUID NOT NULL REFERENCES sip_profiles (sip_profile_uuid) ON DELETE CASCADE,
      setting_name VARCHAR(128) NOT NULL,
      setting_value VARCHAR(512),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_sip_profile_settings_parent ON sip_profile_settings (sip_profile_uuid)",
    // users / groups (026 + 023 parity)
    "CREATE TABLE IF NOT EXISTS users (
      user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      username VARCHAR(64) NOT NULL,
      email VARCHAR(255),
      password_hash VARCHAR(255) NOT NULL DEFAULT '',
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_users_domain ON users (domain_id)",
    "CREATE INDEX IF NOT EXISTS ix_users_username ON users (username)",
    "CREATE TABLE IF NOT EXISTS groups (
      group_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(64) NOT NULL,
      description TEXT,
      group_level INTEGER NOT NULL DEFAULT 10,
      group_protected BOOLEAN NOT NULL DEFAULT FALSE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_groups_domain ON groups (domain_id)",
    "CREATE TABLE IF NOT EXISTS user_groups (
      id UUID PRIMARY KEY,
      user_uuid UUID NOT NULL REFERENCES users (user_uuid) ON DELETE CASCADE,
      group_uuid UUID NOT NULL REFERENCES groups (group_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE
    )",
    "CREATE INDEX IF NOT EXISTS ix_user_groups_group ON user_groups (group_uuid)",
    "CREATE TABLE IF NOT EXISTS permissions (
      permission_uuid UUID PRIMARY KEY,
      name VARCHAR(64) NOT NULL UNIQUE,
      description TEXT
    )",
    "CREATE TABLE IF NOT EXISTS group_permissions (
      id UUID PRIMARY KEY,
      group_uuid UUID NOT NULL REFERENCES groups (group_uuid) ON DELETE CASCADE,
      permission_uuid UUID NOT NULL REFERENCES permissions (permission_uuid) ON DELETE CASCADE,
      domain_id UUID
    )",
    // devices + lines/settings/keys/profiles (032 parity, trimmed)
    "CREATE TABLE IF NOT EXISTS devices (
      device_uuid UUID PRIMARY KEY,
      mac VARCHAR(17) NOT NULL,
      vendor VARCHAR(32) NOT NULL,
      model VARCHAR(64),
      label VARCHAR(128),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension_uuid UUID REFERENCES extensions (extension_uuid),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_devices_domain ON devices (domain_id)",
    "CREATE TABLE IF NOT EXISTS device_lines (
      device_line_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      device_uuid UUID NOT NULL REFERENCES devices (device_uuid) ON DELETE CASCADE,
      line_number VARCHAR(8) NOT NULL DEFAULT '1',
      server_address VARCHAR(128),
      label VARCHAR(128),
      display_name VARCHAR(128),
      user_id VARCHAR(64),
      auth_id VARCHAR(64),
      password VARCHAR(64),
      sip_port INTEGER,
      sip_transport VARCHAR(32),
      register_expires INTEGER,
      shared_line VARCHAR(32),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_device_lines_device ON device_lines (device_uuid)",
    "CREATE TABLE IF NOT EXISTS device_settings (
      device_setting_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      device_uuid UUID NOT NULL REFERENCES devices (device_uuid) ON DELETE CASCADE,
      device_profile_uuid UUID,
      device_setting_category VARCHAR(64),
      device_setting_subcategory VARCHAR(64),
      device_setting_name VARCHAR(128) NOT NULL,
      device_setting_value VARCHAR(512),
      device_setting_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      device_setting_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_keys (
      device_key_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      device_uuid UUID NOT NULL REFERENCES devices (device_uuid) ON DELETE CASCADE,
      device_profile_uuid UUID,
      device_key_id INTEGER,
      device_key_category VARCHAR(64),
      device_key_vendor VARCHAR(64),
      device_key_type VARCHAR(64),
      device_key_subtype VARCHAR(64),
      device_key_line INTEGER,
      device_key_value VARCHAR(128),
      device_key_extension VARCHAR(64),
      device_key_protected VARCHAR(16),
      device_key_label VARCHAR(128),
      device_key_icon VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_profiles (
      device_profile_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      device_profile_name VARCHAR(128) NOT NULL,
      device_profile_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      device_profile_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_profile_keys (
      device_profile_key_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      device_profile_uuid UUID NOT NULL REFERENCES device_profiles (device_profile_uuid) ON DELETE CASCADE,
      profile_key_id INTEGER,
      profile_key_category VARCHAR(64),
      profile_key_vendor VARCHAR(64),
      profile_key_type VARCHAR(64),
      profile_key_subtype VARCHAR(64),
      profile_key_line INTEGER,
      profile_key_value VARCHAR(128),
      profile_key_extension VARCHAR(64),
      profile_key_protected BOOLEAN NOT NULL DEFAULT FALSE,
      profile_key_label VARCHAR(128),
      profile_key_icon VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_profile_settings (
      device_profile_setting_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      device_profile_uuid UUID NOT NULL REFERENCES device_profiles (device_profile_uuid) ON DELETE CASCADE,
      profile_setting_name VARCHAR(128) NOT NULL,
      profile_setting_value VARCHAR(512),
      profile_setting_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      profile_setting_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    // batch 3a call-routing
    "CREATE TABLE IF NOT EXISTS ring_groups (
      ring_group_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(128) NOT NULL,
      extension VARCHAR(32) NOT NULL,
      strategy VARCHAR(32) NOT NULL DEFAULT 'simultaneous',
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_ring_groups_domain ON ring_groups (domain_id)",
    "CREATE TABLE IF NOT EXISTS ring_group_members (
      ring_group_member_uuid UUID PRIMARY KEY,
      ring_group_uuid UUID NOT NULL REFERENCES ring_groups (ring_group_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      destination VARCHAR(128) NOT NULL,
      delay INTEGER NOT NULL DEFAULT 0,
      timeout INTEGER NOT NULL DEFAULT 30
    )",
    "CREATE INDEX IF NOT EXISTS ix_ring_group_members_parent ON ring_group_members (ring_group_uuid)",
    "CREATE TABLE IF NOT EXISTS call_centers (
      call_center_queue_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      queue_name VARCHAR(128) NOT NULL,
      queue_extension VARCHAR(32),
      queue_strategy VARCHAR(32) NOT NULL DEFAULT 'longest-idle-agent',
      queue_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_call_centers_domain ON call_centers (domain_id)",
    "CREATE TABLE IF NOT EXISTS call_center_agents (
      call_center_agent_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      queue_id UUID NOT NULL REFERENCES call_centers (call_center_queue_uuid) ON DELETE CASCADE,
      agent_name VARCHAR(128) NOT NULL,
      agent_contact VARCHAR(128) NOT NULL,
      agent_status VARCHAR(32) NOT NULL DEFAULT 'available'
    )",
    "CREATE INDEX IF NOT EXISTS ix_call_center_agents_queue ON call_center_agents (queue_id)",
    "CREATE TABLE IF NOT EXISTS call_center_tiers (
      call_center_tier_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      queue_id UUID NOT NULL REFERENCES call_centers (call_center_queue_uuid) ON DELETE CASCADE,
      agent_name VARCHAR(128) NOT NULL,
      tier_level INTEGER NOT NULL DEFAULT 1,
      tier_position INTEGER NOT NULL DEFAULT 1
    )",
    "CREATE TABLE IF NOT EXISTS ivr_menus (
      ivr_menu_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(128) NOT NULL,
      extension VARCHAR(32),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_ivr_menus_domain ON ivr_menus (domain_id)",
    "CREATE TABLE IF NOT EXISTS ivr_menu_options (
      option_uuid UUID PRIMARY KEY,
      ivr_menu_uuid UUID NOT NULL REFERENCES ivr_menus (ivr_menu_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      digits VARCHAR(8) NOT NULL,
      action VARCHAR(64) NOT NULL,
      param VARCHAR(512),
      option_order INTEGER NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS ix_ivr_menu_options_parent ON ivr_menu_options (ivr_menu_uuid)",
    "CREATE TABLE IF NOT EXISTS voicemails (
      voicemail_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_id VARCHAR(32) NOT NULL,
      voicemail_password VARCHAR(32) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_voicemails_domain ON voicemails (domain_id)",
    "CREATE TABLE IF NOT EXISTS voicemail_greetings (
      voicemail_greeting_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_uuid UUID NOT NULL REFERENCES voicemails (voicemail_uuid) ON DELETE CASCADE,
      greeting_name VARCHAR(128) NOT NULL,
      greeting_text VARCHAR(1024)
    )",
    "CREATE TABLE IF NOT EXISTS voicemail_messages (
      voicemail_message_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_uuid UUID NOT NULL REFERENCES voicemails (voicemail_uuid) ON DELETE CASCADE,
      file_name VARCHAR(512),
      message_length INTEGER
    )",
    "CREATE TABLE IF NOT EXISTS conferences (
      conference_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_name VARCHAR(128) NOT NULL,
      conference_extension VARCHAR(32) NOT NULL,
      conference_profile VARCHAR(64) NOT NULL DEFAULT 'default',
      conference_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_conferences_domain ON conferences (domain_id)",
    "CREATE TABLE IF NOT EXISTS fax_queue (
      fax_queue_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      fax_extension VARCHAR(32) NOT NULL,
      destination VARCHAR(32) NOT NULL,
      status VARCHAR(32) NOT NULL DEFAULT 'waiting'
    )",
    "CREATE INDEX IF NOT EXISTS ix_fax_queue_domain ON fax_queue (domain_id)",
    "CREATE TABLE IF NOT EXISTS cdr (
      cdr_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      caller_id_number VARCHAR(32),
      destination_number VARCHAR(32) NOT NULL,
      billsec INTEGER,
      duration INTEGER,
      hangup_cause VARCHAR(64),
      direction VARCHAR(16),
      raw_xml TEXT,
      malformed BOOLEAN NOT NULL DEFAULT FALSE
    )",
    "CREATE INDEX IF NOT EXISTS ix_cdr_domain ON cdr (domain_id)",
    "CREATE TABLE IF NOT EXISTS recordings (
      recording_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(255) NOT NULL,
      file_path VARCHAR(512) NOT NULL,
      storage_backend VARCHAR(32) NOT NULL DEFAULT 's3',
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS call_recordings (
      call_recording_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      cdr_uuid UUID,
      recording_path VARCHAR(512),
      recording_name VARCHAR(255)
    )",
    "CREATE TABLE IF NOT EXISTS contacts (
      contact_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(128) NOT NULL,
      number VARCHAR(32) NOT NULL,
      email VARCHAR(255),
      organization VARCHAR(128),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_contacts_domain ON contacts (domain_id)",
    "CREATE TABLE IF NOT EXISTS email_queue (
      email_queue_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      email_from VARCHAR(512) NOT NULL,
      email_to VARCHAR(512) NOT NULL,
      email_subject VARCHAR(512),
      email_body TEXT,
      email_status VARCHAR(32) NOT NULL DEFAULT 'waiting',
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_email_queue_domain ON email_queue (domain_id)",
    "CREATE TABLE IF NOT EXISTS remaining_records (
      id UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      resource VARCHAR(64) NOT NULL,
      data TEXT NOT NULL,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_remaining_resource_domain ON remaining_records (resource, domain_id)",
];

/// Strict tenant tables (non-nullable domain_id) — from 043_tenant_rls_all.py.
const STRICT_TABLES: &[&str] = &[
    "extensions",
    "dialplans",
    "gateways",
    "sip_profiles",
    "sip_profile_domains",
    "sip_profile_settings",
    "ivr_menus",
    "devices",
    "cdr",
    "recordings",
    "contacts",
    "ring_groups",
    "voicemails",
    "conferences",
    "fax_queue",
    "email_queue",
    "call_centers",
    "vars",
    "call_recordings",
    "call_center_agents",
    "ring_group_members",
    "ivr_menu_options",
    "voicemail_greetings",
    "voicemail_messages",
    "dialplan_details",
    "device_lines",
    "device_settings",
    "device_keys",
    "device_profiles",
    "device_profile_keys",
    "device_profile_settings",
    "call_center_tiers",
    "remaining_records",
];

/// Nullable tenant tables (shared/global rows allowed).
const NULLABLE_TABLES: &[&str] = &["groups", "users", "domain_settings"];

/// Key FS views (alembic 003/005/006/011/013 parity, trimmed to ported tables).
const VIEWS: &[&str] = &[
    "CREATE OR REPLACE VIEW v_vars AS SELECT domain_id, var_uuid, var_name, var_category, var_value FROM vars",
    "CREATE OR REPLACE VIEW ring_group_destinations AS SELECT domain_id, ring_group_uuid, destination, delay, timeout FROM ring_group_members",
    "CREATE OR REPLACE VIEW v_voicemails AS SELECT domain_id, voicemail_uuid, voicemail_id FROM voicemails",
    "CREATE OR REPLACE VIEW v_conferences AS SELECT domain_id, conference_uuid, conference_name, conference_extension FROM conferences",
    "CREATE OR REPLACE VIEW v_call_centers AS SELECT domain_id, call_center_queue_uuid, queue_name, queue_extension FROM call_centers",
];

fn rls_statements() -> Vec<String> {
    let mut out = Vec::new();
    for tbl in STRICT_TABLES {
        out.push(format!("ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY"));
        out.push(format!("ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY"));
        out.push(format!("DROP POLICY IF EXISTS tenant_{tbl} ON {tbl}"));
        out.push(format!(
            "CREATE POLICY tenant_{tbl} ON {tbl} USING (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid) WITH CHECK (domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)"
        ));
    }
    for tbl in NULLABLE_TABLES {
        out.push(format!("ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY"));
        out.push(format!("ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY"));
        out.push(format!("DROP POLICY IF EXISTS tenant_{tbl} ON {tbl}"));
        out.push(format!(
            "CREATE POLICY tenant_{tbl} ON {tbl} USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid) WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)"
        ));
    }
    for tbl in ["permissions", "domains", "sofia_global_settings"] {
        out.push(format!("ALTER TABLE {tbl} DISABLE ROW LEVEL SECURITY"));
    }
    out
}

#[async_trait::async_trait]
impl MigrationTrait for M20260101_000001PgParity {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let backend = manager.get_database_backend();
        let is_pg = matches!(backend, DbBackend::Postgres);
        if is_pg {
            let _ = conn.execute_unprepared(EXT_PGCRYPTO).await;
        }
        for ddl in DDL_TABLES {
            conn.execute_unprepared(ddl).await?;
        }
        if is_pg {
            for stmt in rls_statements() {
                // Best-effort: ignore already-exists races.
                let _ = conn.execute_unprepared(&stmt).await;
            }
            for view in VIEWS {
                let _ = conn.execute_unprepared(view).await;
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let backend = manager.get_database_backend();
        if matches!(backend, DbBackend::Postgres) {
            for view in [
                "v_vars",
                "ring_group_destinations",
                "v_voicemails",
                "v_conferences",
                "v_call_centers",
            ] {
                let _ = conn
                    .execute_unprepared(&format!("DROP VIEW IF EXISTS {view}"))
                    .await;
            }
            for tbl in STRICT_TABLES.iter().chain(NULLABLE_TABLES.iter()) {
                let _ = conn
                    .execute_unprepared(&format!("DROP POLICY IF EXISTS tenant_{tbl} ON {tbl}"))
                    .await;
                let _ = conn
                    .execute_unprepared(&format!("ALTER TABLE {tbl} DISABLE ROW LEVEL SECURITY"))
                    .await;
            }
        }
        // Drop in reverse dependency order (children first).
        let tables = [
            "remaining_records",
            "email_queue",
            "contacts",
            "call_recordings",
            "recordings",
            "cdr",
            "fax_queue",
            "conferences",
            "voicemail_messages",
            "voicemail_greetings",
            "voicemails",
            "ivr_menu_options",
            "ivr_menus",
            "call_center_tiers",
            "call_center_agents",
            "call_centers",
            "ring_group_members",
            "ring_groups",
            "device_profile_settings",
            "device_profile_keys",
            "device_profiles",
            "device_keys",
            "device_settings",
            "device_lines",
            "devices",
            "group_permissions",
            "permissions",
            "user_groups",
            "groups",
            "users",
            "sip_profile_settings",
            "sip_profile_domains",
            "sip_profiles",
            "gateways",
            "dialplan_details",
            "dialplans",
            "extensions",
            "domain_settings",
            "default_settings",
            "sofia_global_settings",
            "vars",
            "domains",
        ];
        for tbl in tables {
            let _ = conn
                .execute_unprepared(&format!("DROP TABLE IF EXISTS {tbl}"))
                .await;
        }
        Ok(())
    }
}
