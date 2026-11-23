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
      default_setting_order INTEGER,
      default_setting_enabled BOOLEAN NOT NULL,
      default_setting_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE default_settings ADD COLUMN IF NOT EXISTS default_setting_order INTEGER",
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
    "CREATE TABLE IF NOT EXISTS domain_plugins (
      id UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      plugin_name VARCHAR(64) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_domain_plugins_domain ON domain_plugins (domain_id)",
    "CREATE TABLE IF NOT EXISTS domain_apps (
      id UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      app_name VARCHAR(64) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL,
      UNIQUE (domain_id, app_name)
    )",
    "CREATE INDEX IF NOT EXISTS ix_domain_apps_domain ON domain_apps (domain_id)",
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
      unique_id VARCHAR(64),
      dial_user VARCHAR(128),
      dial_domain VARCHAR(128),
      follow_me_destinations TEXT,
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
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
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
    "ALTER TABLE dialplans ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP dialplans parity),
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
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
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
    // Global/shared gateways (PHP `gateway_domain` parity): pre-existing
    // installs have NOT NULL; relax idempotently (no-op when already NULL-able).
    "ALTER TABLE gateways ALTER COLUMN domain_id DROP NOT NULL",
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
    // users / groups (026 + 023 parity; 023 adds user_totp_secret for TOTP)
    "CREATE TABLE IF NOT EXISTS users (
      user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      username VARCHAR(64) NOT NULL,
      email VARCHAR(255),
      user_email VARCHAR(255),
      password_hash VARCHAR(255) NOT NULL DEFAULT '',
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      user_enabled BOOLEAN,
      user_status VARCHAR(64),
      user_time_zone VARCHAR(64),
      user_language VARCHAR(16),
      user_type VARCHAR(32),
      api_key VARCHAR(64),
      contact_uuid UUID,
      contact_name_given VARCHAR(64),
      contact_name_family VARCHAR(64),
      contact_organization VARCHAR(128),
      user_totp_secret VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    // Existing postgres DBs predate the TOTP column: add it idempotently.
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_totp_secret VARCHAR(64)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_email VARCHAR(255)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_enabled BOOLEAN",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_status VARCHAR(64)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_time_zone VARCHAR(64)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_language VARCHAR(16)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS user_type VARCHAR(32)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS api_key VARCHAR(64)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS contact_uuid UUID",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS contact_name_given VARCHAR(64)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS contact_name_family VARCHAR(64)",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS contact_organization VARCHAR(128)",
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
    "CREATE TABLE IF NOT EXISTS user_settings (
      user_setting_uuid UUID PRIMARY KEY,
      user_uuid UUID NOT NULL REFERENCES users (user_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      user_setting_category VARCHAR(64) NOT NULL,
      user_setting_subcategory VARCHAR(64),
      user_setting_name VARCHAR(64),
      user_setting_value VARCHAR(512),
      user_setting_order INTEGER,
      user_setting_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      user_setting_description TEXT
    )",
    "ALTER TABLE user_settings ADD COLUMN IF NOT EXISTS user_setting_order INTEGER",
    "CREATE INDEX IF NOT EXISTS ix_user_settings_user ON user_settings (user_uuid)",
    "CREATE TABLE IF NOT EXISTS user_logs (
      user_log_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      user_uuid UUID,
      username VARCHAR(128),
      log_type VARCHAR(64),
      result VARCHAR(64),
      detail VARCHAR(1024),
      remote_address VARCHAR(64),
      user_agent VARCHAR(512),
      timestamp TIMESTAMPTZ,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_user_logs_domain ON user_logs (domain_id)",
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
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension_uuid UUID REFERENCES extensions (extension_uuid),
      device_enabled_date TIMESTAMPTZ,
      device_user_uuid UUID,
      device_uuid_alternate VARCHAR(64),
      device_provisioned_agent VARCHAR(128),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE devices ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP devices parity),
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
      greeting VARCHAR(256),
      exit_key VARCHAR(8),
      call_timeout INTEGER,
      forward_destination VARCHAR(128),
      forward_enabled BOOLEAN,
      caller_id_name VARCHAR(128),
      caller_id_number VARCHAR(32),
      cid_name_prefix VARCHAR(64),
      cid_number_prefix VARCHAR(32),
      timeout_app VARCHAR(128),
      timeout_data VARCHAR(512),
      distinctive_ring VARCHAR(128),
      ringback VARCHAR(128),
      call_screen_enabled BOOLEAN,
      call_forward_enabled BOOLEAN,
      follow_me_enabled BOOLEAN,
      missed_call_app VARCHAR(128),
      missed_call_data VARCHAR(512),
      context VARCHAR(128),
      dialplan_uuid UUID,
      forward_toll_allow VARCHAR(64),
      wait_announcement VARCHAR(256),
      wait_interval INTEGER,
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
      timeout INTEGER NOT NULL DEFAULT 30,
      destination_number VARCHAR(128),
      destination_enabled BOOLEAN,
      destination_prompt VARCHAR(8),
      destination_description VARCHAR(512)
    )",
    "CREATE INDEX IF NOT EXISTS ix_ring_group_members_parent ON ring_group_members (ring_group_uuid)",
    "CREATE TABLE IF NOT EXISTS call_centers (
      call_center_queue_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      queue_name VARCHAR(128) NOT NULL,
      queue_extension VARCHAR(32),
      queue_strategy VARCHAR(32) NOT NULL DEFAULT 'longest-idle-agent',
      queue_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      dialplan_uuid UUID,
      greeting VARCHAR(256),
      moh_sound VARCHAR(256),
      record_template VARCHAR(256),
      language VARCHAR(32),
      dialect VARCHAR(32),
      voice VARCHAR(32),
      queue_limit INTEGER,
      time_base_score VARCHAR(64),
      time_base_score_sec INTEGER,
      max_wait_time INTEGER,
      max_wait_time_with_no_agent INTEGER,
      max_wait_time_with_no_agent_time_reached INTEGER,
      tier_rules_apply BOOLEAN,
      tier_rule_wait_second INTEGER,
      tier_rule_no_agent_no_wait BOOLEAN,
      timeout_action VARCHAR(128),
      discard_abandoned_after INTEGER,
      abandoned_resume_allowed BOOLEAN,
      tier_rule_wait_multiply_level BOOLEAN,
      cid_prefix VARCHAR(32),
      outbound_caller_id_name VARCHAR(128),
      outbound_caller_id_number VARCHAR(32),
      announce_position BOOLEAN,
      announce_sound VARCHAR(256),
      announce_frequency INTEGER,
      cc_exit_keys VARCHAR(32),
      email_address VARCHAR(256),
      context VARCHAR(128),
      queue_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_call_centers_domain ON call_centers (domain_id)",
    "CREATE TABLE IF NOT EXISTS call_center_agents (
      call_center_agent_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      queue_id UUID NOT NULL REFERENCES call_centers (call_center_queue_uuid) ON DELETE CASCADE,
      user_uuid UUID,
      agent_name VARCHAR(128) NOT NULL,
      agent_type VARCHAR(32),
      agent_call_timeout INTEGER,
      agent_id VARCHAR(64),
      agent_password VARCHAR(128),
      agent_contact VARCHAR(128) NOT NULL,
      agent_status VARCHAR(32) NOT NULL DEFAULT 'available',
      agent_logout VARCHAR(32),
      agent_max_no_answer INTEGER,
      agent_wrap_up_time INTEGER,
      agent_reject_delay_time INTEGER,
      agent_busy_delay_time INTEGER,
      agent_no_answer_delay_time INTEGER,
      agent_record BOOLEAN
    )",
    "CREATE INDEX IF NOT EXISTS ix_call_center_agents_queue ON call_center_agents (queue_id)",
    "CREATE TABLE IF NOT EXISTS call_center_tiers (
      call_center_tier_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      queue_id UUID NOT NULL REFERENCES call_centers (call_center_queue_uuid) ON DELETE CASCADE,
      call_center_queue_uuid UUID,
      call_center_agent_uuid UUID,
      agent_name VARCHAR(128) NOT NULL,
      queue_name VARCHAR(128),
      tier_level INTEGER NOT NULL DEFAULT 1,
      tier_position INTEGER NOT NULL DEFAULT 1
    )",
    "CREATE TABLE IF NOT EXISTS ivr_menus (
      ivr_menu_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(128) NOT NULL,
      extension VARCHAR(32),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      greet_long VARCHAR(256),
      greet_short VARCHAR(256),
      invalid_sound VARCHAR(256),
      exit_sound VARCHAR(256),
      confirm_macro VARCHAR(128),
      confirm_key VARCHAR(8),
      tts_engine VARCHAR(32),
      tts_voice VARCHAR(32),
      confirm_attempts INTEGER,
      timeout INTEGER,
      inter_digit_timeout INTEGER,
      max_failures INTEGER,
      max_timeouts INTEGER,
      digit_len INTEGER,
      dialplan_uuid UUID,
      parent_uuid UUID,
      language VARCHAR(32),
      dialect VARCHAR(32),
      voice VARCHAR(32),
      pin_number VARCHAR(32),
      ringback VARCHAR(128),
      cid_prefix VARCHAR(32),
      context VARCHAR(128),
      direct_dial BOOLEAN,
      exit_app VARCHAR(128),
      exit_data VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE ivr_menus ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP ivr_menus parity),
    "CREATE INDEX IF NOT EXISTS ix_ivr_menus_domain ON ivr_menus (domain_id)",
    "CREATE TABLE IF NOT EXISTS ivr_menu_options (
      option_uuid UUID PRIMARY KEY,
      ivr_menu_uuid UUID NOT NULL REFERENCES ivr_menus (ivr_menu_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      digits VARCHAR(8) NOT NULL,
      action VARCHAR(64) NOT NULL,
      param VARCHAR(512),
      option_order INTEGER NOT NULL DEFAULT 0,
      ivr_menu_id VARCHAR(64),
      ivr_menu_option_enabled BOOLEAN
    )",
    "CREATE INDEX IF NOT EXISTS ix_ivr_menu_options_parent ON ivr_menu_options (ivr_menu_uuid)",
    "CREATE TABLE IF NOT EXISTS voicemails (
      voicemail_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_id VARCHAR(32) NOT NULL,
      voicemail_password VARCHAR(32) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      voicemail_tutorial BOOLEAN,
      greeting_id VARCHAR(32),
      voicemail_alternate_greet_id VARCHAR(32),
      voicemail_recording_instructions BOOLEAN,
      voicemail_recording_options VARCHAR(128),
      voicemail_mail_to VARCHAR(512),
      voicemail_sms_to VARCHAR(32),
      voicemail_transcription_enabled BOOLEAN,
      voicemail_transcription_prompt_enabled BOOLEAN,
      voicemail_transcription_prompt VARCHAR(1024),
      voicemail_attach_file BOOLEAN,
      voicemail_file VARCHAR(512),
      voicemail_local_after_email BOOLEAN,
      voicemail_local_after_forward BOOLEAN,
      alternate_voicemail_destination VARCHAR(128),
      alternate_voicemail_enabled BOOLEAN,
      voicemail_name_base64 TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE voicemails ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP voicemails parity),
    "CREATE INDEX IF NOT EXISTS ix_voicemails_domain ON voicemails (domain_id)",
    "CREATE TABLE IF NOT EXISTS voicemail_greetings (
      voicemail_greeting_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_uuid UUID NOT NULL REFERENCES voicemails (voicemail_uuid) ON DELETE CASCADE,
      greeting_name VARCHAR(128) NOT NULL,
      greeting_text VARCHAR(1024),
      greeting_id VARCHAR(32),
      greeting_voice VARCHAR(32),
      greeting_message TEXT,
      greeting_filename VARCHAR(512),
      greeting_description VARCHAR(512),
      greeting_base64 TEXT
    )",
    "CREATE TABLE IF NOT EXISTS voicemail_messages (
      voicemail_message_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_uuid UUID NOT NULL REFERENCES voicemails (voicemail_uuid) ON DELETE CASCADE,
      file_name VARCHAR(512),
      message_length INTEGER,
      created_epoch BIGINT,
      read_epoch BIGINT,
      caller_id_name VARCHAR(128),
      caller_id_number VARCHAR(32),
      message_status VARCHAR(32),
      message_priority VARCHAR(16),
      message_intro_base64 TEXT,
      message_base64 TEXT,
      message_transcription TEXT
    )",
    "CREATE TABLE IF NOT EXISTS voicemail_options (
      voicemail_option_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_uuid UUID NOT NULL REFERENCES voicemails (voicemail_uuid) ON DELETE CASCADE,
      digits VARCHAR(8),
      action VARCHAR(64),
      param VARCHAR(512),
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conference_room_users (
      conference_room_user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_room_uuid UUID NOT NULL,
      user_uuid UUID NOT NULL,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS database_transactions (
      database_transaction_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      user_uuid UUID,
      app_name VARCHAR(128),
      app_uuid UUID,
      transaction_code VARCHAR(64),
      transaction_address VARCHAR(256),
      transaction_type VARCHAR(32),
      transaction_date TIMESTAMPTZ,
      transaction_old TEXT,
      transaction_new TEXT,
      transaction_result VARCHAR(32),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_vendors (
      device_vendor_uuid UUID PRIMARY KEY,
      name VARCHAR(128) NOT NULL,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_vendor_functions (
      device_vendor_function_uuid UUID PRIMARY KEY,
      device_vendor_uuid UUID NOT NULL,
      label VARCHAR(128),
      subtype VARCHAR(64),
      value VARCHAR(128),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS device_vendor_function_groups (
      device_vendor_function_group_uuid UUID PRIMARY KEY,
      device_vendor_function_uuid UUID NOT NULL,
      device_vendor_uuid UUID NOT NULL,
      group_name VARCHAR(128),
      group_uuid UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS email_queue_attachments (
      email_queue_attachment_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      email_queue_uuid UUID NOT NULL,
      email_attachment_mime_type VARCHAR(128),
      email_attachment_type VARCHAR(32),
      email_attachment_path VARCHAR(512),
      email_attachment_name VARCHAR(256),
      email_attachment_base64 TEXT,
      email_attachment_cid VARCHAR(256),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS extension_users (
      extension_user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension_uuid UUID NOT NULL,
      user_uuid UUID NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS modules (
      module_uuid UUID PRIMARY KEY,
      module_label VARCHAR(128),
      module_name VARCHAR(128),
      module_category VARCHAR(64),
      module_order INTEGER,
      module_enabled BOOLEAN,
      module_default_enabled BOOLEAN,
      module_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_label VARCHAR(128)",
    "ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_name VARCHAR(128)",
    "ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_category VARCHAR(64)",
    "ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_enabled BOOLEAN",
    "ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_default_enabled BOOLEAN",
    "ALTER TABLE modules ADD COLUMN IF NOT EXISTS module_description VARCHAR(512)",
    "CREATE TABLE IF NOT EXISTS ring_group_users (
      ring_group_user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      ring_group_uuid UUID NOT NULL,
      user_uuid UUID NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS voicemail_destinations (
      voicemail_destination_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      voicemail_uuid UUID NOT NULL,
      voicemail_uuid_copy UUID
    )",
    "CREATE TABLE IF NOT EXISTS xml_cdr_extensions (
      xml_cdr_extension_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      xml_cdr_uuid UUID NOT NULL,
      extension_uuid UUID,
      start_stamp TIMESTAMPTZ,
      end_stamp TIMESTAMPTZ,
      duration INTEGER,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS xml_cdr_json (
      xml_cdr_json_uuid UUID PRIMARY KEY,
      xml_cdr_uuid UUID NOT NULL,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      start_stamp TIMESTAMPTZ,
      json TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS xml_cdr_flow (
      xml_cdr_flow_uuid UUID PRIMARY KEY,
      xml_cdr_uuid UUID NOT NULL,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      call_flow VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS xml_cdr_logs (
      xml_cdr_log_uuid UUID PRIMARY KEY,
      xml_cdr_uuid UUID NOT NULL,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS xml_cdr_transcripts (
      xml_cdr_transcript_uuid UUID PRIMARY KEY,
      xml_cdr_uuid UUID NOT NULL,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      transcript_json TEXT,
      transcript_summary TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conferences (
      conference_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_name VARCHAR(128) NOT NULL,
      conference_extension VARCHAR(32) NOT NULL,
      conference_profile VARCHAR(64) NOT NULL DEFAULT 'default',
      conference_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      dialplan_uuid UUID,
      pin_number VARCHAR(32),
      email_address VARCHAR(256),
      account_code VARCHAR(64),
      flags VARCHAR(128),
      conference_order INTEGER,
      conference_description VARCHAR(512),
      context VARCHAR(128),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE conferences ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP conferences parity),
    "CREATE INDEX IF NOT EXISTS ix_conferences_domain ON conferences (domain_id)",
    "CREATE TABLE IF NOT EXISTS conference_users (
      conference_user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_uuid UUID NOT NULL REFERENCES conferences (conference_uuid) ON DELETE CASCADE,
      user_uuid UUID NOT NULL,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conference_controls (
      conference_control_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      control_name VARCHAR(128) NOT NULL,
      control_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      control_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_conference_controls_domain ON conference_controls (domain_id)",
    "CREATE TABLE IF NOT EXISTS conference_profiles (
      conference_profile_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      profile_name VARCHAR(128) NOT NULL,
      profile_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      profile_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_conference_profiles_domain ON conference_profiles (domain_id)",
    "CREATE TABLE IF NOT EXISTS conference_control_details (
      conference_control_detail_uuid UUID PRIMARY KEY,
      conference_control_uuid UUID NOT NULL,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      control_digits VARCHAR(16),
      control_action VARCHAR(64),
      control_data VARCHAR(512),
      control_enabled BOOLEAN,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conference_profile_params (
      conference_profile_param_uuid UUID PRIMARY KEY,
      conference_profile_uuid UUID NOT NULL,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      profile_param_name VARCHAR(128),
      profile_param_value VARCHAR(512),
      profile_param_enabled BOOLEAN,
      profile_param_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conference_centers (
      conference_center_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_center_name VARCHAR(128) NOT NULL,
      conference_center_extension VARCHAR(32),
      conference_center_description VARCHAR(512),
      conference_center_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      dialplan_uuid UUID,
      conference_center_pin_length INTEGER,
      conference_center_greeting VARCHAR(256),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_conference_centers_domain ON conference_centers (domain_id)",
    "CREATE TABLE IF NOT EXISTS conference_rooms (
      conference_room_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_center_uuid UUID NOT NULL REFERENCES conference_centers (conference_center_uuid) ON DELETE CASCADE,
      conference_room_name VARCHAR(128) NOT NULL,
      max_members INTEGER,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      profile VARCHAR(64),
      moderator_pin VARCHAR(32),
      participant_pin VARCHAR(32),
      start_datetime TIMESTAMPTZ,
      stop_datetime TIMESTAMPTZ,
      wait_mod BOOLEAN,
      moderator_endconf BOOLEAN,
      announce_name BOOLEAN,
      announce_count BOOLEAN,
      announce_recording BOOLEAN,
      sounds VARCHAR(512),
      mute BOOLEAN,
      created VARCHAR(64),
      created_by VARCHAR(128),
      email_address VARCHAR(256),
      account_code VARCHAR(64),
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conference_sessions (
      conference_session_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_center_uuid UUID REFERENCES conference_centers (conference_center_uuid) ON DELETE CASCADE,
      conference_room_uuid UUID REFERENCES conference_rooms (conference_room_uuid) ON DELETE CASCADE,
      profile VARCHAR(64),
      meeting_uuid UUID,
      recording VARCHAR(512),
      start_epoch BIGINT,
      end_epoch BIGINT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS conference_session_details (
      conference_session_detail_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      conference_session_uuid UUID NOT NULL REFERENCES conference_sessions (conference_session_uuid) ON DELETE CASCADE,
      username VARCHAR(32),
      meeting_uuid UUID,
      caller_id_name VARCHAR(128),
      caller_id_number VARCHAR(32),
      moderator BOOLEAN,
      network_addr VARCHAR(64),
      start_epoch BIGINT,
      end_epoch BIGINT,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS fax_queue (
      fax_queue_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      fax_uuid UUID,
      fax_extension VARCHAR(32) NOT NULL,
      destination VARCHAR(32) NOT NULL,
      status VARCHAR(32) NOT NULL DEFAULT 'waiting',
      origination_uuid UUID,
      fax_log_uuid UUID,
      fax_date VARCHAR(32),
      hostname VARCHAR(128),
      fax_caller_id_name VARCHAR(128),
      fax_caller_id_number VARCHAR(32),
      fax_recipient VARCHAR(128),
      fax_number VARCHAR(32),
      fax_prefix VARCHAR(8),
      fax_email_address VARCHAR(256),
      fax_file VARCHAR(512),
      fax_retry_date VARCHAR(32),
      fax_notify_sent BOOLEAN,
      fax_notify_date VARCHAR(32),
      fax_retry_count INTEGER,
      fax_accountcode VARCHAR(64),
      fax_command VARCHAR(512),
      fax_response TEXT
    )",
    "CREATE INDEX IF NOT EXISTS ix_fax_queue_domain ON fax_queue (domain_id)",
    "CREATE TABLE IF NOT EXISTS fax (
      fax_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      fax_extension VARCHAR(32) NOT NULL,
      fax_name VARCHAR(128) NOT NULL,
      description VARCHAR(512),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      dialplan_uuid UUID,
      fax_destination_number VARCHAR(32),
      fax_prefix VARCHAR(8),
      fax_email_connection_type VARCHAR(16),
      fax_email_connection_host VARCHAR(256),
      fax_email_connection_port INTEGER,
      fax_email_connection_security VARCHAR(16),
      fax_email_connection_validate BOOLEAN,
      fax_email_connection_username VARCHAR(256),
      fax_email_connection_password VARCHAR(256),
      fax_email_connection_mailbox VARCHAR(256),
      fax_email_inbound_subject_tag VARCHAR(128),
      fax_email_outbound_subject_tag VARCHAR(128),
      fax_email_outbound_authorized_senders TEXT,
      fax_pin_number VARCHAR(32),
      fax_caller_id_name VARCHAR(128),
      fax_caller_id_number VARCHAR(32),
      fax_toll_allow VARCHAR(64),
      fax_file VARCHAR(512),
      fax_user_list TEXT,
      fax_forward_number VARCHAR(32),
      fax_send_channels INTEGER,
      accountcode VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE fax ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP fax parity),
    "CREATE INDEX IF NOT EXISTS ix_fax_domain ON fax (domain_id)",
    "ALTER TABLE fax ADD COLUMN IF NOT EXISTS fax_file VARCHAR(512)",
    "CREATE TABLE IF NOT EXISTS fax_files (
      fax_file_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      fax_uuid UUID NOT NULL REFERENCES fax (fax_uuid) ON DELETE CASCADE,
      fax_mode VARCHAR(32),
      fax_recipient VARCHAR(128),
      fax_destination VARCHAR(32),
      fax_file_type VARCHAR(32),
      fax_file_path VARCHAR(512),
      fax_caller_id_name VARCHAR(128),
      fax_caller_id_number VARCHAR(32),
      fax_date VARCHAR(32),
      fax_epoch BIGINT,
      fax_base64 TEXT,
      read_date VARCHAR(32)
    )",
    "CREATE TABLE IF NOT EXISTS fax_logs (
      fax_log_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      fax_uuid UUID NOT NULL REFERENCES fax (fax_uuid) ON DELETE CASCADE,
      fax_success BOOLEAN,
      fax_result_code VARCHAR(64),
      fax_result_text VARCHAR(512),
      fax_file VARCHAR(512),
      fax_ecm_used BOOLEAN,
      fax_local_station_id VARCHAR(128),
      fax_document_transferred_pages INTEGER,
      fax_document_total_pages INTEGER,
      fax_image_resolution VARCHAR(32),
      fax_image_size BIGINT,
      fax_bad_rows INTEGER,
      fax_transfer_rate VARCHAR(32),
      fax_retry_attempts INTEGER,
      fax_retry_limit INTEGER,
      fax_retry_sleep INTEGER,
      fax_uri VARCHAR(512),
      fax_duration INTEGER,
      fax_date VARCHAR(32),
      fax_epoch BIGINT
    )",
    "CREATE TABLE IF NOT EXISTS fax_users (
      fax_user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      fax_uuid UUID NOT NULL REFERENCES fax (fax_uuid) ON DELETE CASCADE,
      user_uuid UUID,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS cdr (
      cdr_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      caller_id_number VARCHAR(32),
      caller_id_name VARCHAR(128),
      destination_number VARCHAR(32) NOT NULL,
      start_stamp TIMESTAMPTZ,
      answer_stamp TIMESTAMPTZ,
      end_stamp TIMESTAMPTZ,
      billsec INTEGER,
      duration INTEGER,
      billmsec BIGINT,
      mduration INTEGER,
      hold_accum_seconds INTEGER,
      hangup_cause VARCHAR(64),
      direction VARCHAR(16),
      sip_call_id VARCHAR(128),
      accountcode VARCHAR(64),
      context VARCHAR(128),
      leg VARCHAR(16),
      status VARCHAR(32),
      bridge_uuid VARCHAR(128),
      conference_id VARCHAR(128),
      record_path VARCHAR(512),
      record_name VARCHAR(255),
      mos_score REAL,
      caller_extension VARCHAR(32),
      destination_extension VARCHAR(32),
      read_codec VARCHAR(32),
      write_codec VARCHAR(32),
      network_addr VARCHAR(64),
      pdd_ms INTEGER,
      record_length INTEGER,
      call_disposition VARCHAR(32),
      hangup_cause_q850 VARCHAR(16),
      sip_hangup_disposition VARCHAR(32),
      raw_xml TEXT,
      malformed BOOLEAN NOT NULL DEFAULT FALSE,
      provider_uuid UUID,
      extension_uuid UUID,
      domain_name VARCHAR(128),
      default_language VARCHAR(16),
      caller_destination VARCHAR(32),
      source_number VARCHAR(32),
      start_epoch BIGINT,
      answer_epoch BIGINT,
      end_epoch BIGINT,
      read_rate INTEGER,
      write_rate INTEGER,
      remote_media_ip VARCHAR(64),
      record_transcription TEXT,
      originating_leg_uuid UUID,
      rtp_audio_in_mos REAL,
      last_app VARCHAR(64),
      last_arg VARCHAR(512),
      voicemail_message VARCHAR(8),
      missed_call BOOLEAN,
      call_center_queue_uuid UUID,
      cc_side VARCHAR(16),
      cc_member_uuid UUID,
      cc_queue_joined_epoch BIGINT,
      cc_queue VARCHAR(128),
      cc_member_session_uuid UUID,
      cc_agent_uuid UUID,
      cc_agent VARCHAR(128),
      cc_agent_type VARCHAR(32),
      cc_agent_bridged BOOLEAN,
      cc_queue_answered_epoch BIGINT,
      cc_queue_terminated_epoch BIGINT,
      cc_queue_canceled_epoch BIGINT,
      cc_cancel_reason VARCHAR(64),
      cc_cause VARCHAR(64),
      waitsec INTEGER,
      conference_name VARCHAR(128),
      conference_uuid UUID,
      conference_member_id VARCHAR(16),
      digits_dialed VARCHAR(32),
      pin_number VARCHAR(32),
      ring_group_uuid UUID,
      ivr_menu_uuid UUID,
      processed BOOLEAN,
      call_flow VARCHAR(64),
      json TEXT
    )",
    "CREATE INDEX IF NOT EXISTS ix_cdr_domain ON cdr (domain_id)",
    "CREATE INDEX IF NOT EXISTS ix_cdr_domain_start ON cdr (domain_id, start_stamp)",
    "CREATE TABLE IF NOT EXISTS cdr_archive (
      cdr_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      caller_id_number VARCHAR(32),
      caller_id_name VARCHAR(128),
      destination_number VARCHAR(32) NOT NULL,
      start_stamp TIMESTAMPTZ,
      answer_stamp TIMESTAMPTZ,
      end_stamp TIMESTAMPTZ,
      billsec INTEGER,
      duration INTEGER,
      billmsec BIGINT,
      mduration INTEGER,
      hold_accum_seconds INTEGER,
      hangup_cause VARCHAR(64),
      direction VARCHAR(16),
      sip_call_id VARCHAR(128),
      accountcode VARCHAR(64),
      context VARCHAR(128),
      leg VARCHAR(16),
      status VARCHAR(32),
      bridge_uuid VARCHAR(128),
      conference_id VARCHAR(128),
      record_path VARCHAR(512),
      record_name VARCHAR(255),
      mos_score REAL,
      caller_extension VARCHAR(32),
      destination_extension VARCHAR(32),
      read_codec VARCHAR(32),
      write_codec VARCHAR(32),
      network_addr VARCHAR(64),
      pdd_ms INTEGER,
      record_length INTEGER,
      call_disposition VARCHAR(32),
      hangup_cause_q850 VARCHAR(16),
      sip_hangup_disposition VARCHAR(32),
      raw_xml TEXT,
      malformed BOOLEAN NOT NULL DEFAULT FALSE,
      archived_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_cdr_archive_domain ON cdr_archive (domain_id)",
    "CREATE TABLE IF NOT EXISTS recordings (
      recording_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(255) NOT NULL,
      file_path VARCHAR(512) NOT NULL,
      storage_backend VARCHAR(32) NOT NULL DEFAULT 's3',
      recording_voice VARCHAR(64),
      recording_description TEXT,
      recording_speed VARCHAR(16),
      recording_message TEXT,
      recording_base64 TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE recordings ADD COLUMN IF NOT EXISTS recording_description TEXT",
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
      contact_organization VARCHAR(128),
      contact_type VARCHAR(32),
      contact_parent_uuid UUID,
      contact_name_prefix VARCHAR(64),
      contact_name_given VARCHAR(128),
      contact_name_middle VARCHAR(128),
      contact_name_family VARCHAR(128),
      contact_name_suffix VARCHAR(64),
      contact_nickname VARCHAR(128),
      contact_title VARCHAR(128),
      contact_role VARCHAR(128),
      contact_category VARCHAR(64),
      contact_url VARCHAR(512),
      contact_time_zone VARCHAR(64),
      contact_note VARCHAR(1024),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_type VARCHAR(32)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_parent_uuid UUID",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_name_prefix VARCHAR(64)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_name_given VARCHAR(128)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_name_middle VARCHAR(128)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_name_family VARCHAR(128)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_name_suffix VARCHAR(64)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_nickname VARCHAR(128)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_title VARCHAR(128)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_role VARCHAR(128)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_category VARCHAR(64)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_url VARCHAR(512)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_time_zone VARCHAR(64)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_note VARCHAR(1024)",
    "ALTER TABLE contacts ADD COLUMN IF NOT EXISTS contact_organization VARCHAR(128)",
    "CREATE INDEX IF NOT EXISTS ix_contacts_domain ON contacts (domain_id)",
    "CREATE TABLE IF NOT EXISTS contact_addresses (
      contact_address_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      address_label VARCHAR(64),
      address_street VARCHAR(255),
      address_extended VARCHAR(255),
      address_community VARCHAR(128),
      address_locality VARCHAR(128),
      address_region VARCHAR(128),
      address_postal_code VARCHAR(32),
      address_country VARCHAR(128),
      address_type VARCHAR(32),
      address_primary BOOLEAN NOT NULL DEFAULT FALSE,
      address_latitude VARCHAR(32),
      address_longitude VARCHAR(32),
      address_description VARCHAR(255),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_extended VARCHAR(255)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_community VARCHAR(128)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_region VARCHAR(128)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_postal_code VARCHAR(32)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_country VARCHAR(128)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_type VARCHAR(32)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_primary BOOLEAN NOT NULL DEFAULT FALSE",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_latitude VARCHAR(32)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_longitude VARCHAR(32)",
    "ALTER TABLE contact_addresses ADD COLUMN IF NOT EXISTS address_description VARCHAR(255)",
    "CREATE TABLE IF NOT EXISTS contact_phones (
      contact_phone_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      phone_label VARCHAR(64),
      phone_number VARCHAR(32) NOT NULL,
      phone_country_code VARCHAR(8),
      phone_extension VARCHAR(16),
      phone_speed_dial VARCHAR(8),
      phone_description VARCHAR(255),
      phone_primary BOOLEAN NOT NULL DEFAULT FALSE,
      phone_type_voice BOOLEAN NOT NULL DEFAULT TRUE,
      phone_type_fax BOOLEAN NOT NULL DEFAULT FALSE,
      phone_type_video BOOLEAN NOT NULL DEFAULT FALSE,
      phone_type_text BOOLEAN NOT NULL DEFAULT FALSE,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_country_code VARCHAR(8)",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_extension VARCHAR(16)",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_speed_dial VARCHAR(8)",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_description VARCHAR(255)",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_type_voice BOOLEAN NOT NULL DEFAULT TRUE",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_type_fax BOOLEAN NOT NULL DEFAULT FALSE",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_type_video BOOLEAN NOT NULL DEFAULT FALSE",
    "ALTER TABLE contact_phones ADD COLUMN IF NOT EXISTS phone_type_text BOOLEAN NOT NULL DEFAULT FALSE",
    "CREATE TABLE IF NOT EXISTS contact_emails (
      contact_email_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      email_label VARCHAR(64),
      email_address VARCHAR(255) NOT NULL,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_urls (
      contact_url_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      url_label VARCHAR(64),
      url_address VARCHAR(512) NOT NULL,
      url_type VARCHAR(32),
      url_primary BOOLEAN NOT NULL DEFAULT FALSE,
      url_description VARCHAR(255),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE contact_urls ADD COLUMN IF NOT EXISTS url_type VARCHAR(32)",
    "ALTER TABLE contact_urls ADD COLUMN IF NOT EXISTS url_primary BOOLEAN NOT NULL DEFAULT FALSE",
    "ALTER TABLE contact_urls ADD COLUMN IF NOT EXISTS url_description VARCHAR(255)",
    "CREATE TABLE IF NOT EXISTS contact_notes (
      contact_note_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      contact_note VARCHAR(1024) NOT NULL,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_attachments (
      contact_attachment_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      attachment_primary BOOLEAN,
      attachment_filename VARCHAR(255),
      attachment_content TEXT,
      attachment_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_groups (
      contact_group_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      group_uuid UUID,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_relations (
      contact_relation_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      relation_label VARCHAR(128),
      relation_contact_uuid UUID,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_settings (
      contact_setting_uuid UUID PRIMARY KEY,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_setting_category VARCHAR(64),
      contact_setting_subcategory VARCHAR(64),
      contact_setting_name VARCHAR(128),
      contact_setting_value VARCHAR(1024),
      contact_setting_order INTEGER,
      contact_setting_enabled BOOLEAN,
      contact_setting_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_times (
      contact_time_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      user_uuid UUID,
      time_start VARCHAR(32),
      time_stop VARCHAR(32),
      time_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS contact_users (
      contact_user_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      contact_uuid UUID NOT NULL REFERENCES contacts (contact_uuid) ON DELETE CASCADE,
      user_uuid UUID,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS countries (
      country_uuid UUID PRIMARY KEY,
      country VARCHAR(128) NOT NULL,
      iso_a2 VARCHAR(2),
      iso_a3 VARCHAR(3),
      num INTEGER,
      country_code VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS languages (
      language_uuid UUID PRIMARY KEY,
      language VARCHAR(128) NOT NULL,
      code VARCHAR(16),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS menus (
      menu_uuid UUID PRIMARY KEY,
      menu_name VARCHAR(128) NOT NULL,
      menu_language VARCHAR(16),
      menu_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS menu_items (
      menu_item_uuid UUID PRIMARY KEY,
      menu_uuid UUID,
      menu_item_parent_uuid UUID,
      item_uuid VARCHAR(64),
      menu_item_title VARCHAR(128) NOT NULL,
      menu_item_link VARCHAR(512),
      menu_item_icon VARCHAR(64),
      menu_item_icon_color VARCHAR(32),
      menu_item_category VARCHAR(64),
      menu_item_protected BOOLEAN,
      menu_item_order INTEGER,
      menu_item_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS menu_item_groups (
      menu_item_group_uuid UUID PRIMARY KEY,
      menu_uuid UUID,
      menu_item_uuid UUID,
      group_name VARCHAR(64),
      group_uuid UUID,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS menu_languages (
      menu_language_uuid UUID PRIMARY KEY,
      menu_uuid UUID,
      menu_item_uuid UUID,
      menu_language VARCHAR(16),
      menu_item_title VARCHAR(128),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS software (
      software_uuid UUID PRIMARY KEY,
      software_name VARCHAR(128),
      software_url VARCHAR(512),
      software_version VARCHAR(64),
      created_at TIMESTAMPTZ NOT NULL
    )",
    // v_services parity (fusionpbx/core/services): global registry, no
    // domain_id — stays open (DISABLE RLS below, like modules).
    "CREATE TABLE IF NOT EXISTS services (
      service_uuid UUID PRIMARY KEY,
      service_name VARCHAR(128),
      service_category VARCHAR(64),
      service_file VARCHAR(512),
      service_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      service_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS notifications (
      notification_uuid UUID PRIMARY KEY,
      project_notifications TEXT,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS databases (
      database_uuid UUID PRIMARY KEY,
      database_driver VARCHAR(32),
      database_type VARCHAR(32),
      database_host VARCHAR(128),
      database_port VARCHAR(16),
      database_name VARCHAR(128),
      database_username VARCHAR(128),
      database_password VARCHAR(128),
      database_path VARCHAR(512),
      database_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS dashboards (
      dashboard_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      dashboard_name VARCHAR(128) NOT NULL DEFAULT '',
      dashboard_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      dashboard_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_dashboards_domain ON dashboards (domain_id)",
    "CREATE TABLE IF NOT EXISTS dashboard_widgets (
      dashboard_widget_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      dashboard_uuid UUID REFERENCES dashboards (dashboard_uuid) ON DELETE CASCADE,
      widget_name VARCHAR(128),
      widget_path VARCHAR(512),
      widget_icon VARCHAR(128),
      widget_icon_color VARCHAR(32),
      widget_url VARCHAR(512),
      widget_target VARCHAR(32),
      widget_width VARCHAR(32),
      widget_height VARCHAR(32),
      widget_content TEXT,
      widget_content_text_align VARCHAR(32),
      widget_content_details TEXT,
      widget_chart_type VARCHAR(32),
      widget_label_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      widget_label_text_color VARCHAR(32),
      widget_label_text_color_hover VARCHAR(32),
      widget_label_background_color VARCHAR(32),
      widget_label_background_color_hover VARCHAR(32),
      widget_number_text_color VARCHAR(32),
      widget_number_text_color_hover VARCHAR(32),
      widget_number_background_color VARCHAR(32),
      widget_background_gradient_style VARCHAR(32),
      widget_background_gradient_angle VARCHAR(32),
      widget_column_span INTEGER,
      widget_row_span INTEGER,
      widget_details_state VARCHAR(32),
      widget_order INTEGER,
      widget_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      widget_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_dashboard_widgets_dashboard ON dashboard_widgets (dashboard_uuid)",
    "CREATE TABLE IF NOT EXISTS email_queue (
      email_queue_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      email_date TIMESTAMPTZ,
      email_from VARCHAR(512) NOT NULL,
      email_to VARCHAR(512) NOT NULL,
      email_subject VARCHAR(512),
      email_body TEXT,
      email_status VARCHAR(32) NOT NULL DEFAULT 'waiting',
      email_retry_count INTEGER NOT NULL DEFAULT 0,
      email_attempts INTEGER NOT NULL DEFAULT 0,
      email_error VARCHAR(512),
      hostname VARCHAR(128),
      email_action_before VARCHAR(128),
      email_action_after VARCHAR(128),
      email_uuid UUID,
      email_transcription TEXT,
      email_response TEXT,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_email_queue_domain ON email_queue (domain_id)",
    "ALTER TABLE email_queue ADD COLUMN IF NOT EXISTS email_response TEXT",
    "CREATE TABLE IF NOT EXISTS email_templates (
      email_template_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      template_name VARCHAR(128) NOT NULL DEFAULT '',
      template_language VARCHAR(16),
      template_category VARCHAR(64),
      template_subcategory VARCHAR(64),
      template_subject VARCHAR(512),
      template_body TEXT,
      template_type VARCHAR(32),
      template_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      template_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_email_templates_domain ON email_templates (domain_id)",
    "CREATE TABLE IF NOT EXISTS remaining_records (
      id UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      resource VARCHAR(64) NOT NULL,
      data TEXT NOT NULL,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_remaining_resource_domain ON remaining_records (resource, domain_id)",
    // ---- gap3: call_flows + extension_settings (Python models.py parity) ----
    "CREATE TABLE IF NOT EXISTS call_flows (
      call_flow_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      dialplan_uuid UUID,
      call_flow_name VARCHAR(128) NOT NULL,
      call_flow_extension VARCHAR(32) NOT NULL,
      call_flow_feature_code VARCHAR(32),
      call_flow_context VARCHAR(128) NOT NULL DEFAULT 'public',
      call_flow_status VARCHAR(32),
      call_flow_pin_number VARCHAR(32),
      call_flow_label VARCHAR(128),
      call_flow_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      call_flow_description VARCHAR(512),
      call_flow_sound VARCHAR(128),
      call_flow_app VARCHAR(64),
      call_flow_data VARCHAR(512),
      call_flow_alternate_label VARCHAR(128),
      call_flow_alternate_destination VARCHAR(128),
      call_flow_alternate_sound VARCHAR(128),
      call_flow_alternate_app VARCHAR(64),
      call_flow_alternate_data VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE call_flows ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP call_flows parity),
    "CREATE INDEX IF NOT EXISTS ix_call_flows_domain ON call_flows (domain_id)",
    "CREATE TABLE IF NOT EXISTS extension_settings (
      extension_setting_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension_uuid UUID NOT NULL,
      extension_setting_type VARCHAR(64),
      extension_setting_name VARCHAR(128) NOT NULL,
      extension_setting_value VARCHAR(512) NOT NULL,
      extension_setting_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      extension_setting_description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_extension_settings_domain ON extension_settings (domain_id)",
    // ---- spillover replacement: per-table DDL (Python models.py parity) ----
    "CREATE TABLE IF NOT EXISTS call_block (
      call_block_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      call_block_direction VARCHAR(32),
      extension_uuid UUID,
      call_block_name VARCHAR(128) NOT NULL,
      call_block_country_code VARCHAR(8),
      call_block_number VARCHAR(32) NOT NULL,
      call_block_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      call_block_description TEXT,
      call_block_action VARCHAR(32),
      call_block_count INTEGER,
      call_block_app VARCHAR(64),
      call_block_data VARCHAR(512),
      date_added TIMESTAMPTZ,
      insert_user UUID,
      update_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE call_block ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP call_block parity),
    "CREATE INDEX IF NOT EXISTS ix_call_block_domain ON call_block (domain_id)",
    "CREATE TABLE IF NOT EXISTS call_broadcasts (
      call_broadcast_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      broadcast_name VARCHAR(128) NOT NULL,
      broadcast_description TEXT,
      broadcast_start_time BIGINT,
      broadcast_timeout INTEGER,
      broadcast_concurrent_limit INTEGER,
      recording_uuid UUID,
      broadcast_caller_id_name VARCHAR(128),
      broadcast_caller_id_number VARCHAR(32),
      broadcast_destination_type VARCHAR(32),
      broadcast_destination_data VARCHAR(512),
      broadcast_phone_numbers TEXT,
      broadcast_avmd BOOLEAN,
      broadcast_accountcode VARCHAR(64),
      broadcast_toll_allow VARCHAR(128),
      broadcast_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_call_broadcasts_domain ON call_broadcasts (domain_id)",
    "CREATE TABLE IF NOT EXISTS bridges (
      bridge_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      bridge_name VARCHAR(128) NOT NULL,
      bridge_destination VARCHAR(128),
      bridge_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      bridge_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_bridges_domain ON bridges (domain_id)",
    "CREATE TABLE IF NOT EXISTS destinations (
      destination_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      destination_type VARCHAR(32) NOT NULL,
      destination_number VARCHAR(32) NOT NULL,
      destination_prefix VARCHAR(32),
      destination_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      destination_description TEXT,
      destination_context VARCHAR(128),
      destination_app VARCHAR(64),
      destination_data VARCHAR(512),
      destination_caller_id_name VARCHAR(128),
      destination_caller_id_number VARCHAR(32),
      destination_hold_music VARCHAR(128),
      destination_record BOOLEAN,
      dialplan_uuid UUID,
      fax_uuid UUID,
      user_uuid UUID,
      group_uuid UUID,
      provider_uuid UUID,
      destination_trunk_prefix VARCHAR(32),
      trunk_prefix VARCHAR(32),
      destination_area_code VARCHAR(16),
      area_code VARCHAR(16),
      destination_condition_field VARCHAR(128),
      condition_field VARCHAR(128),
      destination_number_regex VARCHAR(512),
      number_regex VARCHAR(512),
      destination_distinctive_ring VARCHAR(128),
      distinctive_ring VARCHAR(128),
      destination_ringback VARCHAR(128),
      ringback VARCHAR(128),
      destination_accountcode VARCHAR(64),
      accountcode VARCHAR(64),
      destination_type_voice BOOLEAN,
      type_voice BOOLEAN,
      destination_type_fax BOOLEAN,
      type_fax BOOLEAN,
      destination_type_emergency BOOLEAN,
      type_emergency BOOLEAN,
      destination_type_text BOOLEAN,
      type_text BOOLEAN,
      destination_conditions VARCHAR(4096),
      conditions VARCHAR(4096),
      destination_actions VARCHAR(4096),
      actions VARCHAR(4096),
      destination_alternate_app VARCHAR(64),
      alternate_app VARCHAR(64),
      destination_alternate_data VARCHAR(512),
      alternate_data VARCHAR(512),
      destination_caller_id VARCHAR(64),
      destination_cid_name_prefix VARCHAR(128),
      destination_order INTEGER,
      destination_email VARCHAR(256),
      destination_insert_user UUID,
      insert_user UUID,
      destination_insert_date TIMESTAMPTZ,
      destination_update_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE destinations ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP destinations parity),
    "CREATE INDEX IF NOT EXISTS ix_destinations_domain ON destinations (domain_id)",
    "CREATE TABLE IF NOT EXISTS follow_me (
      follow_me_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      follow_me_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      cid_name_prefix VARCHAR(128),
      cid_number_prefix VARCHAR(32),
      call_prompt VARCHAR(32),
      dial_string VARCHAR(512),
      description TEXT,
      follow_me_ignore_busy BOOLEAN NOT NULL DEFAULT FALSE,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_follow_me_domain ON follow_me (domain_id)",
    "CREATE TABLE IF NOT EXISTS follow_me_destinations (
      follow_me_destination_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      follow_me_uuid UUID NOT NULL REFERENCES follow_me (follow_me_uuid) ON DELETE CASCADE,
      follow_me_destination VARCHAR(128) NOT NULL,
      follow_me_delay INTEGER NOT NULL DEFAULT 0,
      follow_me_timeout INTEGER NOT NULL DEFAULT 30,
      follow_me_prompt VARCHAR(32),
      follow_me_order INTEGER NOT NULL DEFAULT 0,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_follow_me_destinations_parent ON follow_me_destinations (follow_me_uuid)",
    "CREATE TABLE IF NOT EXISTS call_forwards (
      call_forward_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension_uuid UUID,
      forward_all_destination VARCHAR(128),
      forward_all_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      forward_busy_destination VARCHAR(128),
      forward_busy_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      forward_no_answer_destination VARCHAR(128),
      forward_no_answer_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      forward_user_not_registered_destination VARCHAR(128),
      forward_user_not_registered_enabled BOOLEAN NOT NULL DEFAULT FALSE,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_call_forwards_domain ON call_forwards (domain_id)",
    "CREATE TABLE IF NOT EXISTS time_conditions (
      time_condition_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      name VARCHAR(128) NOT NULL,
      extension VARCHAR(32) NOT NULL,
      context VARCHAR(128),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description TEXT,
      dialplan_uuid UUID,
      time_hour VARCHAR(64),
      time_minute VARCHAR(64),
      time_wday VARCHAR(64),
      time_mday VARCHAR(64),
      time_mon VARCHAR(64),
      time_year VARCHAR(32),
      time_yday VARCHAR(64),
      time_week VARCHAR(32),
      time_mweek VARCHAR(32),
      minute_of_day VARCHAR(64),
      time_of_day VARCHAR(64),
      date_time VARCHAR(128),
      condition VARCHAR(512),
      true_destination VARCHAR(128),
      false_destination VARCHAR(128),
      timezone VARCHAR(64),
      details JSON,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE time_conditions ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP time_conditions parity),
    "CREATE INDEX IF NOT EXISTS ix_time_conditions_domain ON time_conditions (domain_id)",
    "CREATE TABLE IF NOT EXISTS feature_codes (
      feature_code_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      feature_code_name VARCHAR(128) NOT NULL,
      feature_code_value VARCHAR(32),
      feature_code_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      feature_code_description TEXT,
      insert_user UUID,
      update_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_feature_codes_domain ON feature_codes (domain_id)",
    "CREATE TABLE IF NOT EXISTS music_on_hold (
      music_on_hold_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      music_name VARCHAR(128) NOT NULL,
      music_path VARCHAR(512) NOT NULL,
      music_rate INTEGER,
      music_shuffle VARCHAR(16),
      music_channels INTEGER,
      music_interval INTEGER,
      music_timer_name VARCHAR(64),
      music_chime_list VARCHAR(512),
      music_chime_freq VARCHAR(64),
      music_chime_max INTEGER,
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description TEXT,
      insert_user UUID,
      update_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE music_on_hold ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP music_on_hold parity),
    "CREATE INDEX IF NOT EXISTS ix_music_on_hold_domain ON music_on_hold (domain_id)",
    "CREATE TABLE IF NOT EXISTS phrases (
      phrase_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      phrase_name VARCHAR(128) NOT NULL,
      phrase_language VARCHAR(16),
      phrase_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      phrase_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE phrases ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP phrases parity),
    "CREATE INDEX IF NOT EXISTS ix_phrases_domain ON phrases (domain_id)",
    "CREATE TABLE IF NOT EXISTS phrase_details (
      phrase_detail_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      phrase_uuid UUID NOT NULL REFERENCES phrases (phrase_uuid) ON DELETE CASCADE,
      phrase_detail_group INTEGER,
      phrase_detail_tag VARCHAR(64),
      phrase_detail_pattern VARCHAR(128),
      phrase_detail_function VARCHAR(64),
      phrase_detail_data VARCHAR(512),
      phrase_detail_method VARCHAR(32),
      phrase_detail_type VARCHAR(32),
      phrase_detail_order INTEGER NOT NULL DEFAULT 0,
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_phrase_details_parent ON phrase_details (phrase_uuid)",
    "CREATE TABLE IF NOT EXISTS pin_numbers (
      pin_number_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      pin_number VARCHAR(32) NOT NULL,
      accountcode VARCHAR(64),
      enabled BOOLEAN NOT NULL DEFAULT TRUE,
      description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_pin_numbers_domain ON pin_numbers (domain_id)",
    "CREATE TABLE IF NOT EXISTS number_translations (
      number_translation_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      number_translation_name VARCHAR(128) NOT NULL,
      number_translation_regex VARCHAR(512),
      number_translation_replace VARCHAR(512),
      number_translation_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      number_translation_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_number_translations_domain ON number_translations (domain_id)",
    "CREATE TABLE IF NOT EXISTS number_translation_details (
      number_translation_detail_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      number_translation_uuid UUID NOT NULL REFERENCES number_translations (number_translation_uuid) ON DELETE CASCADE,
      number_translation_detail_regex VARCHAR(512) NOT NULL,
      number_translation_detail_replace VARCHAR(512) NOT NULL,
      number_translation_detail_order INTEGER NOT NULL DEFAULT 0,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_number_translation_details_parent ON number_translation_details (number_translation_uuid)",
    "CREATE TABLE IF NOT EXISTS streams (
      stream_uuid UUID PRIMARY KEY,
      domain_id UUID REFERENCES domains (domain_id) ON DELETE CASCADE,
      stream_name VARCHAR(128) NOT NULL,
      stream_location VARCHAR(512),
      stream_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      stream_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "ALTER TABLE streams ALTER COLUMN domain_id DROP NOT NULL", // shared/global rows (PHP streams parity),
    "CREATE INDEX IF NOT EXISTS ix_streams_domain ON streams (domain_id)",
    "CREATE TABLE IF NOT EXISTS access_controls (
      access_control_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      access_control_name VARCHAR(128) NOT NULL,
      access_control_default VARCHAR(16) NOT NULL DEFAULT 'deny',
      access_control_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_access_controls_domain ON access_controls (domain_id)",
    "CREATE TABLE IF NOT EXISTS access_control_nodes (
      access_control_node_uuid UUID PRIMARY KEY,
      access_control_uuid UUID NOT NULL REFERENCES access_controls (access_control_uuid) ON DELETE CASCADE,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      node_type VARCHAR(16) NOT NULL,
      node_cidr VARCHAR(64) NOT NULL,
      node_description TEXT,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_access_control_nodes_parent ON access_control_nodes (access_control_uuid)",
    "CREATE TABLE IF NOT EXISTS webrtc_settings (
      domain_id UUID PRIMARY KEY REFERENCES domains (domain_id) ON DELETE CASCADE,
      enabled BOOLEAN NOT NULL DEFAULT FALSE,
      auth_mode VARCHAR(32) NOT NULL DEFAULT 'extension',
      wss_url VARCHAR(256),
      sip_domain VARCHAR(128),
      ice_servers JSON,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS fifo (
      fifo_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      dialplan_uuid UUID,
      fifo_name VARCHAR(128) NOT NULL,
      fifo_extension VARCHAR(32),
      fifo_agent_queue VARCHAR(256),
      fifo_agent_status VARCHAR(128),
      fifo_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      fifo_description TEXT,
      fifo_strategy VARCHAR(32),
      fifo_members VARCHAR(4096),
      fifo_timeout_seconds VARCHAR(16),
      fifo_exit_key VARCHAR(8),
      fifo_exit_action VARCHAR(512),
      fifo_music VARCHAR(512),
      fifo_order INTEGER,
      fifo_caller_id_name VARCHAR(128),
      fifo_caller_id_number VARCHAR(32),
      insert_user UUID,
      update_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_fifo_domain ON fifo (domain_id)",
    "CREATE TABLE IF NOT EXISTS fifo_members (
      fifo_member_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      fifo_uuid UUID NOT NULL REFERENCES fifo (fifo_uuid) ON DELETE CASCADE,
      member_contact VARCHAR(512),
      member_call_timeout INTEGER,
      member_simultaneous INTEGER,
      member_wrap_up_time INTEGER,
      member_enabled BOOLEAN NOT NULL DEFAULT TRUE,
      insert_user UUID,
      update_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_fifo_members_parent ON fifo_members (fifo_uuid)",
    "CREATE TABLE IF NOT EXISTS emergency_logs (
      emergency_log_uuid UUID PRIMARY KEY,
      domain_id UUID NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
      extension VARCHAR(32),
      event VARCHAR(512),
      created_at TIMESTAMPTZ NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS ix_emergency_logs_domain ON emergency_logs (domain_id)",
    "CREATE TABLE IF NOT EXISTS event_guard_logs (
      event_guard_log_uuid UUID PRIMARY KEY,
      hostname VARCHAR(128),
      log_date TIMESTAMPTZ NOT NULL,
      filter VARCHAR(128),
      ip_address VARCHAR(64),
      extension VARCHAR(32),
      user_agent VARCHAR(512),
      log_status VARCHAR(32),
      insert_user UUID,
      created_at TIMESTAMPTZ NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL
    )",
];

/// Forward column adds for DBs created before the CDR/email expansion.
/// Executed best-effort after `DDL_TABLES` (ignored on error).
const FORWARD_ALTERS: &[&str] = &[
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS caller_id_name VARCHAR(128)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS start_stamp TIMESTAMPTZ",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS answer_stamp TIMESTAMPTZ",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS end_stamp TIMESTAMPTZ",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS billmsec BIGINT",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS mduration INTEGER",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS hold_accum_seconds INTEGER",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS sip_call_id VARCHAR(128)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS accountcode VARCHAR(64)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS context VARCHAR(128)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS leg VARCHAR(16)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS status VARCHAR(32)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS bridge_uuid VARCHAR(128)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS conference_id VARCHAR(128)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS record_path VARCHAR(512)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS record_name VARCHAR(255)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS mos_score REAL",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS caller_extension VARCHAR(32)",
    "ALTER TABLE cdr ADD COLUMN IF NOT EXISTS destination_extension VARCHAR(32)",
    "ALTER TABLE email_queue ADD COLUMN IF NOT EXISTS email_date TIMESTAMPTZ",
    "ALTER TABLE email_queue ADD COLUMN IF NOT EXISTS email_retry_count INTEGER NOT NULL DEFAULT 0",
    "ALTER TABLE email_queue ADD COLUMN IF NOT EXISTS email_attempts INTEGER NOT NULL DEFAULT 0",
    "ALTER TABLE email_queue ADD COLUMN IF NOT EXISTS email_error VARCHAR(512)",
    // NOTE (2026-11-22): countries.country_code was VARCHAR(16) but the
    // reference seed holds multi-code values (e.g. Dominican Republic
    // '1-809,1-829,1-849', 17 chars), aborting boot seed on fresh DBs.
    "ALTER TABLE countries ALTER COLUMN country_code TYPE VARCHAR(64)",
];

/// Strict tenant tables (non-nullable domain_id) — from 043_tenant_rls_all.py.
const STRICT_TABLES: &[&str] = &[
    "extensions",
    "sip_profiles",
    "sip_profile_domains",
    "sip_profile_settings",
    "cdr",
    "cdr_archive",
    "recordings",
    "contacts",
    "ring_groups",
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
    "voicemail_options",
    "conference_centers",
    "conference_rooms",
    "conference_room_users",
    "conference_sessions",
    "conference_session_details",
    "database_transactions",
    // NOTE (2026-11-22): device_vendors*, modules have NO domain_id column
    // (global lookup tables; PHP 043 keeps modules open too). They must NOT
    // be in STRICT — CREATE POLICY would fail, and inside this transactional
    // migration that single failure poisons the whole boot migrate.
    "email_queue_attachments",
    "extension_users",
    "ring_group_users",
    "voicemail_destinations",
    "xml_cdr_extensions",
    "xml_cdr_json",
    "xml_cdr_flow",
    "xml_cdr_logs",
    "xml_cdr_transcripts",
    "conference_users",
    "fax_files",
    "fax_logs",
    "fax_users",
    "contact_addresses",
    "contact_phones",
    "contact_emails",
    "contact_urls",
    "contact_notes",
    "contact_attachments",
    "contact_groups",
    "contact_relations",
    "contact_settings",
    "contact_times",
    "contact_users",
    "device_lines",
    "device_settings",
    "device_keys",
    "device_profiles",
    "device_profile_keys",
    "device_profile_settings",
    "call_center_tiers",
    "remaining_records",
    "call_broadcasts",
    "bridges",
    "follow_me",
    "follow_me_destinations",
    "call_forwards",
    "feature_codes",
    "phrase_details",
    "pin_numbers",
    "number_translations",
    "number_translation_details",
    "access_controls",
    "access_control_nodes",
    "webrtc_settings",
    "fifo",
    "fifo_members",
    "emergency_logs",
];

/// Nullable tenant tables (shared/global rows allowed).
const NULLABLE_TABLES: &[&str] = &["groups", "users", "domain_settings", "gateways", "destinations", "ivr_menus", "devices", "voicemails", "conferences", "fax", "time_conditions", "phrases", "streams", "call_block", "dialplans", "dialplan_details", "music_on_hold", "call_flows"];

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
        // `?show=all` unlock (PHP `show == 'all'` + `X_all` parity): permissive
        // OR-policy — grants reads only when the API sets transaction-local
        // `app.domain_all = '1'` after verifying the JWT holds the resource's
        // `_all` perm. Writes stay tenant-locked (no WITH CHECK here, so
        // INSERT/UPDATE still answer to `tenant_{tbl}`). Raw DB holders can
        // set the GUC themselves, but they already have full DB access.
        out.push(format!("DROP POLICY IF EXISTS tenant_all_{tbl} ON {tbl}"));
        out.push(format!(
            "CREATE POLICY tenant_all_{tbl} ON {tbl} USING (current_setting('app.domain_all', true) = '1')"
        ));
    }
    for tbl in NULLABLE_TABLES {
        out.push(format!("ALTER TABLE {tbl} ENABLE ROW LEVEL SECURITY"));
        out.push(format!("ALTER TABLE {tbl} FORCE ROW LEVEL SECURITY"));
        out.push(format!("DROP POLICY IF EXISTS tenant_{tbl} ON {tbl}"));
        out.push(format!(
            "CREATE POLICY tenant_{tbl} ON {tbl} USING (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid) WITH CHECK (domain_id IS NULL OR domain_id = NULLIF(current_setting('app.domain_uuid', true), '')::uuid)"
        ));
        out.push(format!("DROP POLICY IF EXISTS tenant_all_{tbl} ON {tbl}"));
        out.push(format!(
            "CREATE POLICY tenant_all_{tbl} ON {tbl} USING (current_setting('app.domain_all', true) = '1')"
        ));
    }
    for tbl in [
        "permissions",
        "domains",
        "sofia_global_settings",
        "modules",
        "services",
        "device_vendors",
        "device_vendor_functions",
        "device_vendor_function_groups",
    ] {
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
        // Forward alters for pre-existing DBs (CREATE IF NOT EXISTS
        // never adds columns to old tables). Best-effort: ignore
        // "already exists" / dialect errors.
        for stmt in FORWARD_ALTERS {
            let _ = conn.execute_unprepared(stmt).await;
        }
        if is_pg {
            for stmt in rls_statements() {
                // Best-effort: ignore already-exists races.
                let _ = conn.execute_unprepared(&stmt).await;
            }
            for view in VIEWS {
                let _ = conn.execute_unprepared(view).await;
            }
            // One-shot forward migration: drain JSON spillover rows into the
            // per-table entities (batch 3c → per-table replacement).
            // Best-effort: a half-migrated boot must never fail startup.
            let _ = backfill_remaining_records(conn).await;
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
            "domain_plugins",
            "domain_apps",
            "remaining_records",
            "email_queue",
            "contacts",
            "call_recordings",
            "recordings",
            "cdr",
            "fax_queue",
            "conferences",
            "voicemail_messages",
            "voicemail_options",
            "conference_centers",
            "conference_rooms",
            "conference_sessions",
            "conference_session_details",
            "conference_users",
            "fax",
            "fax_files",
            "fax_logs",
            "fax_users",
            "contact_addresses",
            "contact_phones",
            "contact_emails",
            "contact_urls",
            "contact_notes",
            "contact_attachments",
            "contact_groups",
            "contact_relations",
            "contact_settings",
            "contact_times",
            "contact_users",
            "countries",
            "languages",
            "menus",
            "menu_items",
            "menu_item_groups",
            "menu_languages",
            "software",
            "notifications",
            "databases",
            "voicemail_greetings",
            "voicemails",
            "ivr_menu_options",
            "ivr_menus",
            "call_center_tiers",
            "domain_plugins",
            "domain_apps",
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
            "user_settings",
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

// ---- one-shot spillover backfill (remaining_records → per-table entities) ----

/// Outcome of [`backfill_remaining_records`]: drained rows vs rows left behind
/// (unknown resource or missing required columns — never deleted).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BackfillCounts {
    pub migrated: usize,
    pub skipped: usize,
}

fn jstr(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(s) = v.get(k).and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_owned());
            }
        }
    }
    None
}

fn jbool(v: &serde_json::Value, keys: &[&str], default: bool) -> bool {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(b) = x.as_bool() {
                return b;
            }
            if let Some(s) = x.as_str() {
                match s.trim().to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => return true,
                    "false" | "0" | "no" | "off" => return false,
                    _ => {}
                }
            }
            if let Some(n) = x.as_i64() {
                return n != 0;
            }
        }
    }
    default
}

fn jint(v: &serde_json::Value, keys: &[&str], default: i32) -> i32 {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(n) = x.as_i64() {
                return n as i32;
            }
            if let Some(s) = x.as_str() {
                if let Ok(n) = s.trim().parse::<i32>() {
                    return n;
                }
            }
        }
    }
    default
}

fn jint64(v: &serde_json::Value, keys: &[&str]) -> Option<i64> {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(n) = x.as_i64() {
                return Some(n);
            }
            if let Some(s) = x.as_str() {
                if let Ok(n) = s.trim().parse::<i64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

use sea_orm::{ActiveModelTrait, EntityTrait, Set};

/// Drain `remaining_records` JSON rows into the per-table entities.
///
/// Idempotent: a row whose PK already exists counts as migrated and is
/// drained (deleted). Unknown resources and rows missing required columns
/// are left in place and counted as skipped. Missing `remaining_records`
/// table (fresh DB) yields zero counts, not an error.
pub async fn backfill_remaining_records<C: sea_orm::ConnectionTrait>(
    db: &C,
) -> Result<BackfillCounts, DbErr> {
    use crate::entities::*;
    use sea_orm::{ColumnTrait as _, QueryFilter as _, QueryOrder as _, QuerySelect as _};
    let mut out = BackfillCounts::default();
    // Cursor-paged drain: `data` holds large JSON payloads, so never
    // buffer the whole staging table — 500 rows per page ordered by PK.
    let mut last_id: Option<String> = None;
    loop {
        let mut sel = remaining_record::Entity::find()
            .order_by_asc(remaining_record::Column::Id)
            .limit(500);
        if let Some(ref lid) = last_id {
            sel = sel.filter(remaining_record::Column::Id.gt(lid.clone()));
        }
        let page = match sel.all(db).await {
            Ok(r) => r,
            Err(_) => return Ok(out),
        };
        if page.is_empty() {
            break;
        }
        last_id = page.last().map(|r| r.id.clone());
        for row in page {
            let data: serde_json::Value =
                serde_json::from_str(&row.data).unwrap_or(serde_json::Value::Null);
            let now = chrono::Utc::now();
            let migrated: bool = match row.resource.as_str() {
                "call_block" => {
                    let (Some(name), Some(number)) = (
                        jstr(&data, &["call_block_name", "name"])
                            .or_else(|| jstr(&data, &["call_block_number", "number"])),
                        jstr(&data, &["call_block_number", "number"]),
                    ) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = call_block::ActiveModel {
                        call_block_uuid: Set(row.id.clone()),
                        domain_id: Set(Some(row.domain_id)),
                        call_block_direction: Set(jstr(&data, &["call_block_direction"])),
                        call_block_name: Set(name),
                        call_block_country_code: Set(jstr(&data, &["call_block_country_code"])),
                        call_block_number: Set(number),
                        call_block_enabled: Set(jbool(
                            &data,
                            &["call_block_enabled", "enabled"],
                            true,
                        )),
                        call_block_description: Set(jstr(
                            &data,
                            &["call_block_description", "description"],
                        )),
                        call_block_action: Set(jstr(&data, &["call_block_action", "action"])),
                        call_block_app: Set(jstr(&data, &["call_block_app", "app"])),
                        call_block_data: Set(jstr(&data, &["call_block_data", "data"])),
                        call_block_count: Set(Some(jint(&data, &["call_block_count"], 0))),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if call_block::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "call_broadcast" => {
                    let Some(name) = jstr(&data, &["broadcast_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = call_broadcasts::ActiveModel {
                        call_broadcast_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        broadcast_name: Set(name),
                        broadcast_description: Set(jstr(
                            &data,
                            &["broadcast_description", "description"],
                        )),
                        broadcast_start_time: Set(jint64(&data, &["broadcast_start_time"])),
                        broadcast_timeout: Set(Some(jint(&data, &["broadcast_timeout"], 30))),
                        broadcast_concurrent_limit: Set(Some(jint(
                            &data,
                            &["broadcast_concurrent_limit"],
                            1,
                        ))),
                        broadcast_caller_id_name: Set(jstr(&data, &["broadcast_caller_id_name"])),
                        broadcast_caller_id_number: Set(jstr(
                            &data,
                            &["broadcast_caller_id_number"],
                        )),
                        broadcast_destination_type: Set(jstr(
                            &data,
                            &["broadcast_destination_type"],
                        )),
                        broadcast_destination_data: Set(jstr(
                            &data,
                            &["broadcast_destination_data"],
                        )),
                        broadcast_phone_numbers: Set(jstr(&data, &["broadcast_phone_numbers"])),
                        broadcast_avmd: Set(data.get("broadcast_avmd").and_then(|x| x.as_bool())),
                        broadcast_accountcode: Set(jstr(&data, &["broadcast_accountcode"])),
                        broadcast_toll_allow: Set(jstr(&data, &["broadcast_toll_allow"])),
                        broadcast_enabled: Set(jbool(
                            &data,
                            &["broadcast_enabled", "enabled"],
                            true,
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if call_broadcasts::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "bridges" => {
                    let Some(name) = jstr(&data, &["bridge_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = bridges::ActiveModel {
                        bridge_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        bridge_name: Set(name),
                        bridge_destination: Set(jstr(
                            &data,
                            &["bridge_destination", "destination"],
                        )),
                        bridge_enabled: Set(jbool(&data, &["bridge_enabled", "enabled"], true)),
                        bridge_description: Set(jstr(
                            &data,
                            &["bridge_description", "description"],
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if bridges::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "destinations" => {
                    let Some(number) =
                        jstr(&data, &["destination_number", "destination", "number"])
                    else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = destinations::ActiveModel {
                        destination_uuid: Set(row.id.clone()),
                        domain_id: Set(Some(row.domain_id)),
                        destination_type: Set(jstr(&data, &["destination_type", "type"])
                            .unwrap_or_else(|| "inbound".to_owned())),
                        destination_number: Set(number),
                        destination_prefix: Set(jstr(&data, &["destination_prefix"])),
                        destination_enabled: Set(jbool(
                            &data,
                            &["destination_enabled", "enabled"],
                            true,
                        )),
                        destination_description: Set(jstr(
                            &data,
                            &["destination_description", "description"],
                        )),
                        destination_context: Set(jstr(&data, &["destination_context"])
                            .or_else(|| Some("public".to_owned()))),
                        destination_app: Set(jstr(&data, &["destination_app", "action", "app"])),
                        destination_data: Set(jstr(&data, &["destination_data", "data"])),
                        destination_caller_id_name: Set(jstr(
                            &data,
                            &["destination_caller_id_name"],
                        )),
                        destination_caller_id_number: Set(jstr(
                            &data,
                            &["destination_caller_id_number"],
                        )),
                        destination_hold_music: Set(jstr(&data, &["destination_hold_music"])),
                        destination_record: Set(data
                            .get("destination_record")
                            .and_then(|x| x.as_bool())),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if destinations::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "follow_me" => {
                    let am = follow_me::ActiveModel {
                        follow_me_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        follow_me_enabled: Set(jbool(
                            &data,
                            &["follow_me_enabled", "enabled"],
                            true,
                        )),
                        cid_name_prefix: Set(jstr(&data, &["cid_name_prefix"])),
                        cid_number_prefix: Set(jstr(&data, &["cid_number_prefix"])),
                        call_prompt: Set(jstr(&data, &["call_prompt"])),
                        dial_string: Set(jstr(&data, &["dial_string"])),
                        description: Set(jstr(&data, &["description"])),
                        follow_me_ignore_busy: Set(jbool(&data, &["follow_me_ignore_busy"], false)),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    let ok = if follow_me::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    };
                    if ok {
                        if let Some(dests) = data.get("destinations").and_then(|x| x.as_array()) {
                            for (i, d) in dests.iter().enumerate() {
                                let num = if let Some(s) = d.as_str() {
                                    Some(s.to_owned())
                                } else {
                                    jstr(d, &["follow_me_destination", "destination", "number"])
                                };
                                let Some(num) = num else { continue };
                                let dam = follow_me_destinations::ActiveModel {
                                    follow_me_destination_uuid: Set(
                                        uuid::Uuid::now_v7().to_string()
                                    ),
                                    domain_id: Set(row.domain_id),
                                    follow_me_uuid: Set(row.id.clone()),
                                    follow_me_destination: Set(num),
                                    follow_me_delay: Set(jint(d, &["follow_me_delay", "delay"], 0)),
                                    follow_me_timeout: Set(jint(
                                        d,
                                        &["follow_me_timeout", "timeout"],
                                        30,
                                    )),
                                    follow_me_prompt: Set(jstr(d, &["follow_me_prompt", "prompt"])),
                                    follow_me_order: Set(i as i32),
                                    created_at: Set(now),
                                };
                                let _ = dam.insert(db).await;
                            }
                        }
                    }
                    ok
                }
                "call_forward" => {
                    let am = call_forwards::ActiveModel {
                        call_forward_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        forward_all_destination: Set(jstr(
                            &data,
                            &["forward_all_destination", "destination"],
                        )),
                        forward_all_enabled: Set(jbool(
                            &data,
                            &["forward_all_enabled", "enabled"],
                            false,
                        )),
                        forward_busy_destination: Set(jstr(&data, &["forward_busy_destination"])),
                        forward_busy_enabled: Set(jbool(&data, &["forward_busy_enabled"], false)),
                        forward_no_answer_destination: Set(jstr(
                            &data,
                            &["forward_no_answer_destination"],
                        )),
                        forward_no_answer_enabled: Set(jbool(
                            &data,
                            &["forward_no_answer_enabled"],
                            false,
                        )),
                        forward_user_not_registered_destination: Set(jstr(
                            &data,
                            &["forward_user_not_registered_destination"],
                        )),
                        forward_user_not_registered_enabled: Set(jbool(
                            &data,
                            &["forward_user_not_registered_enabled"],
                            false,
                        )),
                        enabled: Set(jbool(&data, &["enabled"], true)),
                        description: Set(jstr(&data, &["description"])),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if call_forwards::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "time_conditions" => {
                    let (Some(name), Some(extension)) = (
                        jstr(&data, &["name", "time_condition_name"]),
                        jstr(&data, &["extension", "time_condition_extension"]),
                    ) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = time_conditions::ActiveModel {
                        time_condition_uuid: Set(row.id.clone()),
                        domain_id: Set(Some(row.domain_id)),
                        name: Set(name),
                        extension: Set(extension),
                        context: Set(jstr(&data, &["context", "dialplan_context"])),
                        enabled: Set(jbool(&data, &["enabled", "time_condition_enabled"], true)),
                        description: Set(jstr(
                            &data,
                            &["description", "time_condition_description"],
                        )),
                        condition: Set(jstr(&data, &["condition"])),
                        true_destination: Set(jstr(&data, &["true_destination", "true_dest"])),
                        false_destination: Set(jstr(&data, &["false_destination", "false_dest"])),
                        timezone: Set(jstr(&data, &["timezone"])),
                        details: Set(data.get("details").cloned()),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if time_conditions::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "feature_codes" => {
                    let Some(name) = jstr(&data, &["feature_code_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = feature_codes::ActiveModel {
                        feature_code_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        feature_code_name: Set(name),
                        feature_code_value: Set(
                            jstr(&data, &["feature_code_value", "value"]).unwrap_or_default()
                        ),
                        feature_code_enabled: Set(jbool(
                            &data,
                            &["feature_code_enabled", "enabled"],
                            true,
                        )),
                        feature_code_description: Set(jstr(
                            &data,
                            &["feature_code_description", "description"],
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if feature_codes::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "music_on_hold" => {
                    let Some(name) = jstr(&data, &["music_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = music_on_hold::ActiveModel {
                        music_on_hold_uuid: Set(row.id.clone()),
                        domain_id: Set(Some(row.domain_id)),
                        music_name: Set(name),
                        music_path: Set(jstr(&data, &["music_path", "path"]).unwrap_or_default()),
                        music_rate: Set(Some(jint(&data, &["music_rate"], 8000))),
                        music_shuffle: Set(jstr(&data, &["music_shuffle"])),
                        music_channels: Set(Some(jint(&data, &["music_channels"], 1))),
                        music_interval: Set(Some(jint(&data, &["music_interval"], 20))),
                        music_timer_name: Set(jstr(&data, &["music_timer_name"])),
                        music_chime_list: Set(jstr(&data, &["music_chime_list"])),
                        music_chime_freq: Set(jstr(&data, &["music_chime_freq"])),
                        music_chime_max: Set(data
                            .get("music_chime_max")
                            .and_then(|x| x.as_i64())
                            .map(|n| n as i32)),
                        enabled: Set(jbool(&data, &["enabled"], true)),
                        description: Set(jstr(&data, &["description"])),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if music_on_hold::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "phrases" => {
                    let Some(name) = jstr(&data, &["phrase_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = phrases::ActiveModel {
                        phrase_uuid: Set(row.id.clone()),
                        domain_id: Set(Some(row.domain_id)),
                        phrase_name: Set(name),
                        phrase_language: Set(jstr(&data, &["phrase_language"])),
                        phrase_enabled: Set(jbool(&data, &["phrase_enabled", "enabled"], true)),
                        phrase_description: Set(jstr(
                            &data,
                            &["phrase_description", "description"],
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if phrases::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "pin_numbers" => {
                    let Some(number) = jstr(&data, &["pin_number", "number"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = pin_numbers::ActiveModel {
                        pin_number_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        pin_number: Set(number),
                        accountcode: Set(jstr(&data, &["accountcode"])),
                        enabled: Set(jbool(&data, &["enabled"], true)),
                        description: Set(jstr(&data, &["description"])),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if pin_numbers::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "number_translations" => {
                    let Some(name) = jstr(&data, &["number_translation_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = number_translations::ActiveModel {
                        number_translation_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        number_translation_name: Set(name),
                        number_translation_regex: Set(jstr(
                            &data,
                            &["number_translation_regex", "regex"],
                        )),
                        number_translation_replace: Set(jstr(
                            &data,
                            &["number_translation_replace", "replace"],
                        )),
                        number_translation_enabled: Set(jbool(
                            &data,
                            &["number_translation_enabled", "enabled"],
                            true,
                        )),
                        number_translation_description: Set(jstr(
                            &data,
                            &["number_translation_description", "description"],
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if number_translations::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "streams" => {
                    let Some(name) = jstr(&data, &["stream_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = streams::ActiveModel {
                        stream_uuid: Set(row.id.clone()),
                        domain_id: Set(Some(row.domain_id)),
                        stream_name: Set(name),
                        stream_location: Set(jstr(&data, &["stream_location", "location"])),
                        stream_enabled: Set(jbool(&data, &["stream_enabled", "enabled"], true)),
                        stream_description: Set(jstr(
                            &data,
                            &["stream_description", "description"],
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if streams::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "access_controls" => {
                    let Some(name) = jstr(&data, &["access_control_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = access_controls::ActiveModel {
                        access_control_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        access_control_name: Set(name),
                        access_control_default: Set(jstr(&data, &["access_control_default"])
                            .unwrap_or_else(|| "deny".to_owned())),
                        access_control_description: Set(jstr(
                            &data,
                            &["access_control_description", "description"],
                        )),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if access_controls::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "webrtc_settings" => {
                    let am = webrtc_settings::ActiveModel {
                        domain_id: Set(row.domain_id),
                        enabled: Set(jbool(&data, &["enabled"], false)),
                        auth_mode: Set(
                            jstr(&data, &["auth_mode"]).unwrap_or_else(|| "extension".to_owned())
                        ),
                        wss_url: Set(jstr(&data, &["wss_url"])),
                        sip_domain: Set(jstr(&data, &["sip_domain"])),
                        ice_servers: Set(data.get("ice_servers").cloned()),
                        created_at: Set(now),
                        updated_at: Set(now),
                    };
                    if webrtc_settings::Entity::find_by_id(row.domain_id)
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "fifo_list" | "fifo" => {
                    let Some(name) = jstr(&data, &["fifo_name", "name"]) else {
                        out.skipped += 1;
                        continue;
                    };
                    let am = fifo::ActiveModel {
                        fifo_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        fifo_name: Set(name),
                        fifo_extension: Set(jstr(&data, &["fifo_extension", "extension"])),
                        fifo_enabled: Set(jbool(&data, &["fifo_enabled", "enabled"], true)),
                        fifo_description: Set(jstr(&data, &["fifo_description", "description"])),
                        fifo_strategy: Set(jstr(&data, &["fifo_strategy", "strategy"])),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if fifo::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "emergency_logs" => {
                    let am = emergency_logs::ActiveModel {
                        emergency_log_uuid: Set(row.id.clone()),
                        domain_id: Set(row.domain_id),
                        extension: Set(jstr(&data, &["extension"])),
                        event: Set(jstr(&data, &["event"])),
                        created_at: Set(now),
                    };
                    if emergency_logs::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                "event_guard" => {
                    let am = event_guard_logs::ActiveModel {
                        event_guard_log_uuid: Set(row.id.clone()),
                        hostname: Set(jstr(&data, &["hostname"])),
                        log_date: Set(now),
                        filter: Set(jstr(&data, &["filter"])),
                        ip_address: Set(jstr(&data, &["ip_address"])),
                        extension: Set(jstr(&data, &["extension"])),
                        user_agent: Set(jstr(&data, &["user_agent"])),
                        log_status: Set(jstr(&data, &["log_status", "status"])),
                        created_at: Set(now),
                        updated_at: Set(now),
                        ..Default::default()
                    };
                    if event_guard_logs::Entity::find_by_id(row.id.clone())
                        .one(db)
                        .await?
                        .is_some()
                    {
                        true
                    } else {
                        am.insert(db).await.map(|_| true).unwrap_or(false)
                    }
                }
                _ => {
                    out.skipped += 1;
                    continue;
                }
            };
            if migrated {
                let am: remaining_record::ActiveModel = row.into();
                let _ = am.delete(db).await;
                out.migrated += 1;
            } else {
                out.skipped += 1;
            }
        }
    }
    Ok(out)
}
