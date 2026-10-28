//! Batch 1 settings + batch 2a telephony data layer.
//!
//! Entities mirror Python ground truth column-for-column:
//! - `vars` — app/vars/models.py (+ var_command/var_hostname from alembic 029)
//! - `sofia_global_settings` — app/sofia_global_settings/models.py (global, no domain_id)
//! - `default_settings` — app/admin_settings/models.py (035_provision_defaults)
//! - `domain_settings` — app/domains/models.py
//! - `domains` — app/domains/models.py
//! - `extensions` — app/extensions/models.py (+ helpers in [`extensions`])
//! - `dialplans` / `dialplan_details` — app/dialplans/models.py
//! - `gateways` — app/gateways/models.py (`register` column + [`gateways`] defaults)
//! - `sip_profiles` / `sip_profile_domains` / `sip_profile_settings`
//!   — app/sip_profiles/models.py
//! - `devices` (+ lines/settings/keys/profiles) — app/devices/models.py
//! - batch 3a call-routing: `ring_groups` + `ring_group_members`,
//!   `call_centers` + `call_center_agents` + `call_center_tiers`,
//!   `ivr_menus` + `ivr_menu_options`, `voicemails` + greetings/messages,
//!   `conferences`, `fax_queue`, `cdr`, `recordings` + `call_recordings`.
//!
//! Sofia XML lives in `pbx-fs::sofia` (verbatim port of
//! `app/freeswitch/sofia_config.py::_build_sofia_xml`).
//!
//! Runtime uses Postgres (`DATABASE_URL`); tests use shared in-memory sqlite.

pub mod entities;
pub mod extensions;
pub mod gateways;
pub mod pagination;
pub mod seeds;
pub mod tenant;

pub use pagination::{paginate, Page};
pub use tenant::{connect_sqlite_memory, set_tenant, set_tenant_sql};

use sea_orm::{ConnectionTrait, DbErr};

const DDL_VARS: &str = "CREATE TABLE IF NOT EXISTS vars (
  var_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  var_category VARCHAR(64) NOT NULL,
  var_name VARCHAR(128) NOT NULL,
  var_value VARCHAR(512) NOT NULL,
  var_enabled BOOLEAN NOT NULL,
  var_description TEXT,
  var_order INTEGER,
  var_command VARCHAR(512),
  var_hostname VARCHAR(255),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (domain_id, var_name, var_category)
)";
const DDL_VARS_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_vars_domain_id ON vars (domain_id)";

const DDL_SOFIA: &str = "CREATE TABLE IF NOT EXISTS sofia_global_settings (
  sofia_global_setting_uuid TEXT PRIMARY KEY,
  global_setting_name VARCHAR(128) NOT NULL UNIQUE,
  global_setting_value VARCHAR(512) NOT NULL,
  global_setting_enabled BOOLEAN NOT NULL,
  global_setting_description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_DEFAULT_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS default_settings (
  default_setting_uuid TEXT PRIMARY KEY,
  default_setting_category VARCHAR(64) NOT NULL,
  default_setting_subcategory VARCHAR(64) NOT NULL,
  default_setting_name VARCHAR(64) NOT NULL,
  default_setting_value TEXT,
  default_setting_enabled BOOLEAN NOT NULL,
  default_setting_description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_DEFAULT_SETTINGS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_default_settings_category_sub \
     ON default_settings (default_setting_category, default_setting_subcategory)";

const DDL_DOMAIN_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS domain_settings (
  id TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  category VARCHAR(64) NOT NULL,
  subcategory VARCHAR(64) NOT NULL,
  key VARCHAR(64) NOT NULL,
  value VARCHAR(1024) NOT NULL,
  enabled BOOLEAN NOT NULL,
  created_at TEXT NOT NULL
)";
const DDL_DOMAIN_SETTINGS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_domain_settings_domain ON domain_settings (domain_id)";

const DDL_DOMAINS: &str = "CREATE TABLE IF NOT EXISTS domains (
  domain_id TEXT PRIMARY KEY,
  domain_name VARCHAR(255) NOT NULL UNIQUE,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_EXTENSIONS: &str = "CREATE TABLE IF NOT EXISTS extensions (
  extension_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
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
  directory_visible BOOLEAN NOT NULL DEFAULT 0,
  directory_exten_visible BOOLEAN NOT NULL DEFAULT 0,
  max_registrations VARCHAR(32),
  limit_max VARCHAR(32),
  limit_destination VARCHAR(128),
  missed_call_app VARCHAR(32),
  missed_call_data VARCHAR(128),
  user_context VARCHAR(128),
  toll_allow VARCHAR(128),
  call_timeout INTEGER,
  call_group VARCHAR(32),
  call_screen_enabled BOOLEAN NOT NULL DEFAULT 0,
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
  force_ping BOOLEAN NOT NULL DEFAULT 0,
  dial_string VARCHAR(1024),
  do_not_disturb BOOLEAN NOT NULL DEFAULT 0,
  forward_all_destination VARCHAR(128),
  forward_all_enabled BOOLEAN NOT NULL DEFAULT 0,
  forward_busy_destination VARCHAR(128),
  forward_busy_enabled BOOLEAN NOT NULL DEFAULT 0,
  forward_no_answer_destination VARCHAR(128),
  forward_no_answer_enabled BOOLEAN NOT NULL DEFAULT 0,
  forward_user_not_registered_destination VARCHAR(128),
  forward_user_not_registered_enabled BOOLEAN NOT NULL DEFAULT 0,
  follow_me_uuid TEXT,
  follow_me_enabled BOOLEAN NOT NULL DEFAULT 0,
  extension_language VARCHAR(32),
  extension_dialect VARCHAR(32),
  extension_voice VARCHAR(32),
  extension_type VARCHAR(32),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_EXTENSIONS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_extensions_domain ON extensions (domain_id)";

const DDL_DIALPLANS: &str = "CREATE TABLE IF NOT EXISTS dialplans (
  dialplan_uuid TEXT PRIMARY KEY,
  name VARCHAR(128) NOT NULL,
  context VARCHAR(128) NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  description TEXT,
  dialplan_continue BOOLEAN NOT NULL DEFAULT 0,
  dialplan_order INTEGER,
  hostname VARCHAR(128),
  app_uuid VARCHAR(64),
  dialplan_number VARCHAR(128),
  dialplan_destination BOOLEAN,
  dialplan_xml TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_DIALPLANS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_dialplans_domain_name ON dialplans (domain_id, name)";

const DDL_DIALPLAN_DETAILS: &str = "CREATE TABLE IF NOT EXISTS dialplan_details (
  detail_uuid TEXT PRIMARY KEY,
  dialplan_uuid TEXT NOT NULL REFERENCES dialplans (dialplan_uuid),
  tag VARCHAR(32) NOT NULL,
  \"type\" VARCHAR(64) NOT NULL,
  data VARCHAR(512) NOT NULL,
  \"order\" INTEGER NOT NULL DEFAULT 0,
  domain_id TEXT REFERENCES domains (domain_id),
  dialplan_detail_break VARCHAR(32),
  dialplan_detail_inline VARCHAR(32),
  dialplan_detail_group INTEGER,
  detail_enabled BOOLEAN,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_DIALPLAN_DETAILS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_dialplan_details_parent ON dialplan_details (dialplan_uuid)";

const DDL_GATEWAYS: &str = "CREATE TABLE IF NOT EXISTS gateways (
  gateway_uuid TEXT PRIMARY KEY,
  gateway VARCHAR(64) NOT NULL,
  username VARCHAR(64),
  password VARCHAR(64),
  proxy VARCHAR(128),
  \"register\" BOOLEAN NOT NULL DEFAULT 1,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
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
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_GATEWAYS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_gateways_domain ON gateways (domain_id)";

const DDL_SIP_PROFILES: &str = "CREATE TABLE IF NOT EXISTS sip_profiles (
  sip_profile_uuid TEXT PRIMARY KEY,
  name VARCHAR(64) NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
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
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_SIP_PROFILES_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_sip_profiles_domain_name ON sip_profiles (domain_id, name)";

const DDL_SIP_PROFILE_DOMAINS: &str = "CREATE TABLE IF NOT EXISTS sip_profile_domains (
  sip_profile_domain_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  sip_profile_uuid TEXT NOT NULL REFERENCES sip_profiles (sip_profile_uuid),
  domain_name VARCHAR(128) NOT NULL,
  domain_alias VARCHAR(16),
  domain_parse VARCHAR(16),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_SIP_PROFILE_DOMAINS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_sip_profile_domains_parent ON sip_profile_domains (sip_profile_uuid)";

const DDL_SIP_PROFILE_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS sip_profile_settings (
  sip_profile_setting_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  sip_profile_uuid TEXT NOT NULL REFERENCES sip_profiles (sip_profile_uuid),
  setting_name VARCHAR(128) NOT NULL,
  setting_value VARCHAR(512),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_SIP_PROFILE_SETTINGS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_sip_profile_settings_parent ON sip_profile_settings (sip_profile_uuid)";

const DDL_DEVICES: &str = "CREATE TABLE IF NOT EXISTS devices (
  device_uuid TEXT PRIMARY KEY,
  mac VARCHAR(17) NOT NULL,
  vendor VARCHAR(32) NOT NULL,
  model VARCHAR(64),
  label VARCHAR(128),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  extension_uuid TEXT,
  device_profile_uuid TEXT,
  device_address VARCHAR(32),
  device_label VARCHAR(128),
  device_vendor VARCHAR(32),
  device_model VARCHAR(64),
  device_firmware_version VARCHAR(64),
  device_template VARCHAR(128),
  device_location VARCHAR(128),
  device_serial_number VARCHAR(128),
  device_description VARCHAR(512),
  device_username VARCHAR(64),
  device_password VARCHAR(64),
  device_provisioned_date TEXT,
  device_provisioned_ip VARCHAR(64),
  device_provisioned_method VARCHAR(32),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_DEVICES_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_devices_domain ON devices (domain_id)";

const DDL_DEVICE_LINES: &str = "CREATE TABLE IF NOT EXISTS device_lines (
  device_line_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  device_uuid TEXT NOT NULL REFERENCES devices (device_uuid),
  line_number VARCHAR(8) NOT NULL DEFAULT '1',
  server_address VARCHAR(128),
  server_address_primary VARCHAR(128),
  server_address_secondary VARCHAR(128),
  outbound_proxy_primary VARCHAR(128),
  outbound_proxy_secondary VARCHAR(128),
  label VARCHAR(128),
  display_name VARCHAR(128),
  user_id VARCHAR(64),
  auth_id VARCHAR(64),
  password VARCHAR(64),
  sip_port INTEGER,
  sip_transport VARCHAR(32),
  register_expires INTEGER,
  shared_line VARCHAR(32),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
)";
const DDL_DEVICE_LINES_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_device_lines_device ON device_lines (device_uuid)";

const DDL_DEVICE_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS device_settings (
  device_setting_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  device_uuid TEXT NOT NULL REFERENCES devices (device_uuid),
  device_profile_uuid TEXT,
  device_setting_category VARCHAR(64),
  device_setting_subcategory VARCHAR(64),
  device_setting_name VARCHAR(128) NOT NULL,
  device_setting_value VARCHAR(512),
  device_setting_enabled BOOLEAN NOT NULL DEFAULT 1,
  device_setting_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_DEVICE_KEYS: &str = "CREATE TABLE IF NOT EXISTS device_keys (
  device_key_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  device_uuid TEXT NOT NULL REFERENCES devices (device_uuid),
  device_profile_uuid TEXT,
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
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_DEVICE_PROFILES: &str = "CREATE TABLE IF NOT EXISTS device_profiles (
  device_profile_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  device_profile_name VARCHAR(128) NOT NULL,
  device_profile_enabled BOOLEAN NOT NULL DEFAULT 1,
  device_profile_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_DEVICE_PROFILE_KEYS: &str = "CREATE TABLE IF NOT EXISTS device_profile_keys (
  device_profile_key_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  device_profile_uuid TEXT NOT NULL REFERENCES device_profiles (device_profile_uuid),
  profile_key_id INTEGER,
  profile_key_category VARCHAR(64),
  profile_key_vendor VARCHAR(64),
  profile_key_type VARCHAR(64),
  profile_key_subtype VARCHAR(64),
  profile_key_line INTEGER,
  profile_key_value VARCHAR(128),
  profile_key_extension VARCHAR(64),
  profile_key_protected BOOLEAN NOT NULL DEFAULT 0,
  profile_key_label VARCHAR(128),
  profile_key_icon VARCHAR(64),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_DEVICE_PROFILE_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS device_profile_settings (
  device_profile_setting_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  device_profile_uuid TEXT NOT NULL REFERENCES device_profiles (device_profile_uuid),
  profile_setting_name VARCHAR(128) NOT NULL,
  profile_setting_value VARCHAR(512),
  profile_setting_enabled BOOLEAN NOT NULL DEFAULT 1,
  profile_setting_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_RING_GROUPS: &str = "CREATE TABLE IF NOT EXISTS ring_groups (
  ring_group_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  name VARCHAR(128) NOT NULL,
  extension VARCHAR(32) NOT NULL,
  strategy VARCHAR(32) NOT NULL DEFAULT 'simultaneous',
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_RING_GROUPS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_ring_groups_domain ON ring_groups (domain_id)";

const DDL_RING_GROUP_MEMBERS: &str = "CREATE TABLE IF NOT EXISTS ring_group_members (
  ring_group_member_uuid TEXT PRIMARY KEY,
  ring_group_uuid TEXT NOT NULL REFERENCES ring_groups (ring_group_uuid),
  domain_id TEXT NOT NULL,
  destination VARCHAR(128) NOT NULL,
  delay INTEGER NOT NULL DEFAULT 0,
  timeout INTEGER NOT NULL DEFAULT 30
)";
const DDL_RING_GROUP_MEMBERS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_ring_group_members_parent ON ring_group_members (ring_group_uuid)";

const DDL_CALL_CENTERS: &str = "CREATE TABLE IF NOT EXISTS call_centers (
  call_center_queue_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  queue_name VARCHAR(128) NOT NULL,
  queue_extension VARCHAR(32),
  queue_strategy VARCHAR(32) NOT NULL DEFAULT 'longest-idle-agent',
  queue_enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_CALL_CENTERS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_call_centers_domain ON call_centers (domain_id)";

const DDL_CALL_CENTER_AGENTS: &str = "CREATE TABLE IF NOT EXISTS call_center_agents (
  call_center_agent_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  queue_id TEXT NOT NULL REFERENCES call_centers (call_center_queue_uuid),
  agent_name VARCHAR(128) NOT NULL,
  agent_contact VARCHAR(128) NOT NULL,
  agent_status VARCHAR(32) NOT NULL DEFAULT 'available'
)";
const DDL_CALL_CENTER_AGENTS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_call_center_agents_queue ON call_center_agents (queue_id)";

const DDL_CALL_CENTER_TIERS: &str = "CREATE TABLE IF NOT EXISTS call_center_tiers (
  call_center_tier_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  queue_id TEXT NOT NULL REFERENCES call_centers (call_center_queue_uuid),
  agent_name VARCHAR(128) NOT NULL,
  tier_level INTEGER NOT NULL DEFAULT 1,
  tier_position INTEGER NOT NULL DEFAULT 1
)";

const DDL_IVR_MENUS: &str = "CREATE TABLE IF NOT EXISTS ivr_menus (
  ivr_menu_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  name VARCHAR(128) NOT NULL,
  extension VARCHAR(32),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_IVR_MENUS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_ivr_menus_domain ON ivr_menus (domain_id)";

const DDL_IVR_MENU_OPTIONS: &str = "CREATE TABLE IF NOT EXISTS ivr_menu_options (
  option_uuid TEXT PRIMARY KEY,
  ivr_menu_uuid TEXT NOT NULL REFERENCES ivr_menus (ivr_menu_uuid),
  domain_id TEXT NOT NULL,
  digits VARCHAR(8) NOT NULL,
  action VARCHAR(64) NOT NULL,
  param VARCHAR(512),
  option_order INTEGER NOT NULL DEFAULT 0
)";
const DDL_IVR_MENU_OPTIONS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_ivr_menu_options_parent ON ivr_menu_options (ivr_menu_uuid)";

const DDL_VOICEMAILS: &str = "CREATE TABLE IF NOT EXISTS voicemails (
  voicemail_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  voicemail_id VARCHAR(32) NOT NULL,
  voicemail_password VARCHAR(32) NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_VOICEMAILS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_voicemails_domain ON voicemails (domain_id)";

const DDL_VOICEMAIL_GREETINGS: &str = "CREATE TABLE IF NOT EXISTS voicemail_greetings (
  voicemail_greeting_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  voicemail_uuid TEXT NOT NULL REFERENCES voicemails (voicemail_uuid),
  greeting_name VARCHAR(128) NOT NULL,
  greeting_text VARCHAR(1024)
)";

const DDL_VOICEMAIL_MESSAGES: &str = "CREATE TABLE IF NOT EXISTS voicemail_messages (
  voicemail_message_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  voicemail_uuid TEXT NOT NULL REFERENCES voicemails (voicemail_uuid),
  file_name VARCHAR(512),
  message_length INTEGER
)";

const DDL_CONFERENCES: &str = "CREATE TABLE IF NOT EXISTS conferences (
  conference_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  conference_name VARCHAR(128) NOT NULL,
  conference_extension VARCHAR(32) NOT NULL,
  conference_profile VARCHAR(64) NOT NULL DEFAULT 'default',
  conference_enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_CONFERENCES_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_conferences_domain ON conferences (domain_id)";

const DDL_FAX_QUEUE: &str = "CREATE TABLE IF NOT EXISTS fax_queue (
  fax_queue_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  fax_extension VARCHAR(32) NOT NULL,
  destination VARCHAR(32) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'waiting'
)";
const DDL_FAX_QUEUE_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_fax_queue_domain ON fax_queue (domain_id)";

const DDL_CDR: &str = "CREATE TABLE IF NOT EXISTS cdr (
  cdr_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  caller_id_number VARCHAR(32),
  destination_number VARCHAR(32) NOT NULL,
  hangup_cause VARCHAR(64),
  direction VARCHAR(16),
  raw_xml TEXT,
  malformed BOOLEAN NOT NULL DEFAULT 0
)";
const DDL_CDR_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_cdr_domain ON cdr (domain_id)";

const DDL_RECORDINGS: &str = "CREATE TABLE IF NOT EXISTS recordings (
  recording_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  name VARCHAR(255) NOT NULL,
  file_path VARCHAR(512) NOT NULL,
  storage_backend VARCHAR(32) NOT NULL DEFAULT 's3',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_CALL_RECORDINGS: &str = "CREATE TABLE IF NOT EXISTS call_recordings (
  call_recording_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  cdr_uuid TEXT,
  recording_path VARCHAR(512),
  recording_name VARCHAR(255)
)";

/// Create all tables (sqlite bootstrap for tests / local dev).
/// Production schema comes from alembic migrations.
pub async fn create_schema(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    for ddl in [
        DDL_VARS,
        DDL_VARS_IDX,
        DDL_SOFIA,
        DDL_DEFAULT_SETTINGS,
        DDL_DEFAULT_SETTINGS_IDX,
        DDL_DOMAIN_SETTINGS,
        DDL_DOMAIN_SETTINGS_IDX,
        DDL_DOMAINS,
        DDL_EXTENSIONS,
        DDL_EXTENSIONS_IDX,
        DDL_DIALPLANS,
        DDL_DIALPLANS_IDX,
        DDL_DIALPLAN_DETAILS,
        DDL_DIALPLAN_DETAILS_IDX,
        DDL_GATEWAYS,
        DDL_GATEWAYS_IDX,
        DDL_SIP_PROFILES,
        DDL_SIP_PROFILES_IDX,
        DDL_SIP_PROFILE_DOMAINS,
        DDL_SIP_PROFILE_DOMAINS_IDX,
        DDL_SIP_PROFILE_SETTINGS,
        DDL_SIP_PROFILE_SETTINGS_IDX,
        DDL_DEVICES,
        DDL_DEVICES_IDX,
        DDL_DEVICE_LINES,
        DDL_DEVICE_LINES_IDX,
        DDL_DEVICE_SETTINGS,
        DDL_DEVICE_KEYS,
        DDL_DEVICE_PROFILES,
        DDL_DEVICE_PROFILE_KEYS,
        DDL_DEVICE_PROFILE_SETTINGS,
        DDL_RING_GROUPS,
        DDL_RING_GROUPS_IDX,
        DDL_RING_GROUP_MEMBERS,
        DDL_RING_GROUP_MEMBERS_IDX,
        DDL_CALL_CENTERS,
        DDL_CALL_CENTERS_IDX,
        DDL_CALL_CENTER_AGENTS,
        DDL_CALL_CENTER_AGENTS_IDX,
        DDL_CALL_CENTER_TIERS,
        DDL_IVR_MENUS,
        DDL_IVR_MENUS_IDX,
        DDL_IVR_MENU_OPTIONS,
        DDL_IVR_MENU_OPTIONS_IDX,
        DDL_VOICEMAILS,
        DDL_VOICEMAILS_IDX,
        DDL_VOICEMAIL_GREETINGS,
        DDL_VOICEMAIL_MESSAGES,
        DDL_CONFERENCES,
        DDL_CONFERENCES_IDX,
        DDL_FAX_QUEUE,
        DDL_FAX_QUEUE_IDX,
        DDL_CDR,
        DDL_CDR_IDX,
        DDL_RECORDINGS,
        DDL_CALL_RECORDINGS,
    ] {
        db.execute_unprepared(ddl).await?;
    }
    Ok(())
}
