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
pub mod migration;
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

const DDL_DOMAIN_PLUGINS: &str = "CREATE TABLE IF NOT EXISTS domain_plugins (
  id TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
  plugin_name VARCHAR(64) NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
)";
const DDL_DOMAIN_PLUGINS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_domain_plugins_domain ON domain_plugins (domain_id)";

const DDL_DOMAIN_APPS: &str = "CREATE TABLE IF NOT EXISTS domain_apps (
  id TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id) ON DELETE CASCADE,
  app_name VARCHAR(64) NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  UNIQUE (domain_id, app_name)
)";
const DDL_DOMAIN_APPS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_domain_apps_domain ON domain_apps (domain_id)";

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
  billsec INTEGER,
  duration INTEGER,
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

const DDL_USERS: &str = "CREATE TABLE IF NOT EXISTS users (
  user_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  username VARCHAR(64) NOT NULL,
  email VARCHAR(255),
  password_hash VARCHAR(255) NOT NULL DEFAULT '',
  enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_USERS_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_users_domain ON users (domain_id)";

const DDL_GROUPS: &str = "CREATE TABLE IF NOT EXISTS groups (
  group_uuid TEXT PRIMARY KEY,
  domain_id TEXT REFERENCES domains (domain_id),
  name VARCHAR(64) NOT NULL,
  description TEXT,
  group_level INTEGER NOT NULL DEFAULT 10,
  group_protected BOOLEAN NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_GROUPS_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_groups_domain ON groups (domain_id)";

const DDL_USER_GROUPS: &str = "CREATE TABLE IF NOT EXISTS user_groups (
  id TEXT PRIMARY KEY,
  user_uuid TEXT NOT NULL REFERENCES users (user_uuid),
  group_uuid TEXT NOT NULL REFERENCES groups (group_uuid),
  domain_id TEXT NOT NULL
)";
const DDL_USER_GROUPS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_user_groups_group ON user_groups (group_uuid)";

const DDL_PERMISSIONS: &str = "CREATE TABLE IF NOT EXISTS permissions (
  permission_uuid TEXT PRIMARY KEY,
  name VARCHAR(64) NOT NULL UNIQUE,
  description TEXT
)";

const DDL_GROUP_PERMISSIONS: &str = "CREATE TABLE IF NOT EXISTS group_permissions (
  id TEXT PRIMARY KEY,
  group_uuid TEXT NOT NULL REFERENCES groups (group_uuid),
  permission_uuid TEXT NOT NULL REFERENCES permissions (permission_uuid),
  domain_id TEXT
)";

const DDL_CONTACTS: &str = "CREATE TABLE IF NOT EXISTS contacts (
  contact_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  name VARCHAR(128) NOT NULL,
  number VARCHAR(32) NOT NULL,
  email VARCHAR(255),
  organization VARCHAR(128),
  created_at TEXT NOT NULL
)";
const DDL_CONTACTS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_contacts_domain ON contacts (domain_id)";

const DDL_EMAIL_QUEUE: &str = "CREATE TABLE IF NOT EXISTS email_queue (
  email_queue_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL REFERENCES domains (domain_id),
  email_from VARCHAR(512) NOT NULL,
  email_to VARCHAR(512) NOT NULL,
  email_subject VARCHAR(512),
  email_body TEXT,
  email_status VARCHAR(32) NOT NULL DEFAULT 'waiting',
  created_at TEXT NOT NULL
)";
const DDL_EMAIL_QUEUE_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_email_queue_domain ON email_queue (domain_id)";

const DDL_REMAINING_RECORDS: &str = "CREATE TABLE IF NOT EXISTS remaining_records (
  id TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  resource VARCHAR(64) NOT NULL,
  data TEXT NOT NULL,
  created_at TEXT NOT NULL
)";
const DDL_REMAINING_RECORDS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_remaining_resource_domain ON remaining_records (resource, domain_id)";

// ---- spillover-replacement tables (Python models.py column parity) ----
const DDL_CALL_BLOCK: &str = "CREATE TABLE IF NOT EXISTS call_block (
  call_block_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  call_block_direction VARCHAR(32),
  extension_uuid TEXT,
  call_block_name VARCHAR(128) NOT NULL,
  call_block_country_code VARCHAR(8),
  call_block_number VARCHAR(32) NOT NULL,
  call_block_enabled BOOLEAN NOT NULL DEFAULT 1,
  call_block_description VARCHAR(512),
  call_block_action VARCHAR(32),
  call_block_count INTEGER,
  call_block_app VARCHAR(64),
  call_block_data VARCHAR(512),
  date_added TEXT,
  insert_user TEXT,
  update_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_CALL_BLOCK_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_call_block_domain ON call_block (domain_id)";

const DDL_CALL_BROADCASTS: &str = "CREATE TABLE IF NOT EXISTS call_broadcasts (
  call_broadcast_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  broadcast_name VARCHAR(128) NOT NULL,
  broadcast_description VARCHAR(512),
  broadcast_start_time INTEGER,
  broadcast_timeout INTEGER,
  broadcast_concurrent_limit INTEGER,
  recording_uuid TEXT,
  broadcast_caller_id_name VARCHAR(128),
  broadcast_caller_id_number VARCHAR(32),
  broadcast_destination_type VARCHAR(32),
  broadcast_destination_data VARCHAR(512),
  broadcast_phone_numbers TEXT,
  broadcast_avmd BOOLEAN,
  broadcast_accountcode VARCHAR(64),
  broadcast_toll_allow VARCHAR(128),
  broadcast_enabled BOOLEAN NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_CALL_BROADCASTS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_call_broadcasts_domain ON call_broadcasts (domain_id)";

const DDL_BRIDGES: &str = "CREATE TABLE IF NOT EXISTS bridges (
  bridge_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  bridge_name VARCHAR(128) NOT NULL,
  bridge_destination VARCHAR(128),
  bridge_enabled BOOLEAN NOT NULL DEFAULT 1,
  bridge_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_BRIDGES_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_bridges_domain ON bridges (domain_id)";

const DDL_DESTINATIONS: &str = "CREATE TABLE IF NOT EXISTS destinations (
  destination_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  destination_type VARCHAR(32) NOT NULL,
  destination_number VARCHAR(32) NOT NULL,
  destination_prefix VARCHAR(32),
  destination_enabled BOOLEAN NOT NULL DEFAULT 1,
  destination_description VARCHAR(512),
  destination_context VARCHAR(128),
  destination_app VARCHAR(64),
  destination_data VARCHAR(512),
  destination_caller_id_name VARCHAR(128),
  destination_caller_id_number VARCHAR(32),
  destination_hold_music VARCHAR(128),
  destination_record BOOLEAN,
  dialplan_uuid TEXT,
  fax_uuid TEXT,
  user_uuid TEXT,
  group_uuid TEXT,
  provider_uuid TEXT,
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
  destination_insert_user TEXT,
  insert_user TEXT,
  destination_insert_date TEXT,
  destination_update_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_DESTINATIONS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_destinations_domain ON destinations (domain_id)";

const DDL_FOLLOW_ME: &str = "CREATE TABLE IF NOT EXISTS follow_me (
  follow_me_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  follow_me_enabled BOOLEAN NOT NULL DEFAULT 1,
  cid_name_prefix VARCHAR(128),
  cid_number_prefix VARCHAR(32),
  call_prompt VARCHAR(32),
  dial_string VARCHAR(512),
  description VARCHAR(512),
  follow_me_ignore_busy BOOLEAN NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_FOLLOW_ME_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_follow_me_domain ON follow_me (domain_id)";

const DDL_FOLLOW_ME_DESTINATIONS: &str = "CREATE TABLE IF NOT EXISTS follow_me_destinations (
  follow_me_destination_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  follow_me_uuid TEXT NOT NULL REFERENCES follow_me (follow_me_uuid),
  follow_me_destination VARCHAR(128) NOT NULL,
  follow_me_delay INTEGER NOT NULL DEFAULT 0,
  follow_me_timeout INTEGER NOT NULL DEFAULT 30,
  follow_me_prompt VARCHAR(32),
  follow_me_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
)";
const DDL_FOLLOW_ME_DESTINATIONS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_follow_me_destinations_parent ON follow_me_destinations (follow_me_uuid)";

const DDL_CALL_FORWARDS: &str = "CREATE TABLE IF NOT EXISTS call_forwards (
  call_forward_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  extension_uuid TEXT,
  forward_all_destination VARCHAR(128),
  forward_all_enabled BOOLEAN NOT NULL DEFAULT 0,
  forward_busy_destination VARCHAR(128),
  forward_busy_enabled BOOLEAN NOT NULL DEFAULT 0,
  forward_no_answer_destination VARCHAR(128),
  forward_no_answer_enabled BOOLEAN NOT NULL DEFAULT 0,
  forward_user_not_registered_destination VARCHAR(128),
  forward_user_not_registered_enabled BOOLEAN NOT NULL DEFAULT 0,
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_CALL_FORWARDS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_call_forwards_domain ON call_forwards (domain_id)";

const DDL_TIME_CONDITIONS: &str = "CREATE TABLE IF NOT EXISTS time_conditions (
  time_condition_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  name VARCHAR(128) NOT NULL,
  extension VARCHAR(32) NOT NULL,
  context VARCHAR(128),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  dialplan_uuid TEXT,
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
  details TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_TIME_CONDITIONS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_time_conditions_domain ON time_conditions (domain_id)";

const DDL_FEATURE_CODES: &str = "CREATE TABLE IF NOT EXISTS feature_codes (
  feature_code_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  feature_code_name VARCHAR(128) NOT NULL,
  feature_code_value VARCHAR(32),
  feature_code_enabled BOOLEAN NOT NULL DEFAULT 1,
  feature_code_description VARCHAR(512),
  insert_user TEXT,
  update_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_FEATURE_CODES_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_feature_codes_domain ON feature_codes (domain_id)";

const DDL_MUSIC_ON_HOLD: &str = "CREATE TABLE IF NOT EXISTS music_on_hold (
  music_on_hold_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
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
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  insert_user TEXT,
  update_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_MUSIC_ON_HOLD_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_music_on_hold_domain ON music_on_hold (domain_id)";

const DDL_PHRASES: &str = "CREATE TABLE IF NOT EXISTS phrases (
  phrase_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  phrase_name VARCHAR(128) NOT NULL,
  phrase_language VARCHAR(16),
  phrase_enabled BOOLEAN NOT NULL DEFAULT 1,
  phrase_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_PHRASES_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_phrases_domain ON phrases (domain_id)";

const DDL_PHRASE_DETAILS: &str = "CREATE TABLE IF NOT EXISTS phrase_details (
  phrase_detail_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  phrase_uuid TEXT NOT NULL REFERENCES phrases (phrase_uuid),
  phrase_detail_group INTEGER,
  phrase_detail_tag VARCHAR(64),
  phrase_detail_pattern VARCHAR(128),
  phrase_detail_function VARCHAR(64),
  phrase_detail_data VARCHAR(512),
  phrase_detail_method VARCHAR(32),
  phrase_detail_type VARCHAR(32),
  phrase_detail_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
)";
const DDL_PHRASE_DETAILS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_phrase_details_parent ON phrase_details (phrase_uuid)";

const DDL_PIN_NUMBERS: &str = "CREATE TABLE IF NOT EXISTS pin_numbers (
  pin_number_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  pin_number VARCHAR(32) NOT NULL,
  accountcode VARCHAR(64),
  enabled BOOLEAN NOT NULL DEFAULT 1,
  description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_PIN_NUMBERS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_pin_numbers_domain ON pin_numbers (domain_id)";

const DDL_NUMBER_TRANSLATIONS: &str = "CREATE TABLE IF NOT EXISTS number_translations (
  number_translation_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  number_translation_name VARCHAR(128) NOT NULL,
  number_translation_regex VARCHAR(512),
  number_translation_replace VARCHAR(512),
  number_translation_enabled BOOLEAN NOT NULL DEFAULT 1,
  number_translation_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_NUMBER_TRANSLATIONS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_number_translations_domain ON number_translations (domain_id)";

const DDL_NUMBER_TRANSLATION_DETAILS: &str =
    "CREATE TABLE IF NOT EXISTS number_translation_details (
  number_translation_detail_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  number_translation_uuid TEXT NOT NULL REFERENCES number_translations (number_translation_uuid),
  number_translation_detail_regex VARCHAR(512) NOT NULL,
  number_translation_detail_replace VARCHAR(512) NOT NULL,
  number_translation_detail_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_NUMBER_TRANSLATION_DETAILS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_number_translation_details_parent ON number_translation_details (number_translation_uuid)";

const DDL_STREAMS: &str = "CREATE TABLE IF NOT EXISTS streams (
  stream_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  stream_name VARCHAR(128) NOT NULL,
  stream_location VARCHAR(512),
  stream_enabled BOOLEAN NOT NULL DEFAULT 1,
  stream_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_STREAMS_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_streams_domain ON streams (domain_id)";

const DDL_ACCESS_CONTROLS: &str = "CREATE TABLE IF NOT EXISTS access_controls (
  access_control_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  access_control_name VARCHAR(128) NOT NULL,
  access_control_default VARCHAR(16) NOT NULL DEFAULT 'deny',
  access_control_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_ACCESS_CONTROLS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_access_controls_domain ON access_controls (domain_id)";

const DDL_ACCESS_CONTROL_NODES: &str = "CREATE TABLE IF NOT EXISTS access_control_nodes (
  access_control_node_uuid TEXT PRIMARY KEY,
  access_control_uuid TEXT NOT NULL REFERENCES access_controls (access_control_uuid),
  domain_id TEXT NOT NULL,
  node_type VARCHAR(16) NOT NULL,
  node_cidr VARCHAR(64) NOT NULL,
  node_description VARCHAR(512),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_ACCESS_CONTROL_NODES_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_access_control_nodes_parent ON access_control_nodes (access_control_uuid)";

const DDL_WEBRTC_SETTINGS: &str = "CREATE TABLE IF NOT EXISTS webrtc_settings (
  domain_id TEXT PRIMARY KEY,
  enabled BOOLEAN NOT NULL DEFAULT 0,
  auth_mode VARCHAR(32) NOT NULL DEFAULT 'extension',
  wss_url VARCHAR(256),
  sip_domain VARCHAR(128),
  ice_servers TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const DDL_FIFO: &str = "CREATE TABLE IF NOT EXISTS fifo (
  fifo_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  dialplan_uuid TEXT,
  fifo_name VARCHAR(128) NOT NULL,
  fifo_extension VARCHAR(32),
  fifo_agent_queue VARCHAR(256),
  fifo_agent_status VARCHAR(128),
  fifo_enabled BOOLEAN NOT NULL DEFAULT 1,
  fifo_description VARCHAR(512),
  fifo_strategy VARCHAR(32),
  fifo_members VARCHAR(4096),
  fifo_timeout_seconds VARCHAR(16),
  fifo_exit_key VARCHAR(8),
  fifo_exit_action VARCHAR(512),
  fifo_music VARCHAR(512),
  fifo_order INTEGER,
  fifo_caller_id_name VARCHAR(128),
  fifo_caller_id_number VARCHAR(32),
  insert_user TEXT,
  update_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_FIFO_IDX: &str = "CREATE INDEX IF NOT EXISTS ix_fifo_domain ON fifo (domain_id)";

const DDL_FIFO_MEMBERS: &str = "CREATE TABLE IF NOT EXISTS fifo_members (
  fifo_member_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  fifo_uuid TEXT NOT NULL REFERENCES fifo (fifo_uuid),
  member_contact VARCHAR(512),
  member_call_timeout INTEGER,
  member_simultaneous INTEGER,
  member_wrap_up_time INTEGER,
  member_enabled BOOLEAN NOT NULL DEFAULT 1,
  insert_user TEXT,
  update_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
const DDL_FIFO_MEMBERS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_fifo_members_parent ON fifo_members (fifo_uuid)";

const DDL_EMERGENCY_LOGS: &str = "CREATE TABLE IF NOT EXISTS emergency_logs (
  emergency_log_uuid TEXT PRIMARY KEY,
  domain_id TEXT NOT NULL,
  extension VARCHAR(32),
  event VARCHAR(512),
  created_at TEXT NOT NULL
)";
const DDL_EMERGENCY_LOGS_IDX: &str =
    "CREATE INDEX IF NOT EXISTS ix_emergency_logs_domain ON emergency_logs (domain_id)";

const DDL_EVENT_GUARD_LOGS: &str = "CREATE TABLE IF NOT EXISTS event_guard_logs (
  event_guard_log_uuid TEXT PRIMARY KEY,
  hostname VARCHAR(128),
  log_date TEXT NOT NULL,
  filter VARCHAR(128),
  ip_address VARCHAR(64),
  extension VARCHAR(32),
  user_agent VARCHAR(512),
  log_status VARCHAR(32),
  insert_user TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";
/// Create all tables (sqlite bootstrap for tests / local dev).
/// Production schema comes from alembic migrations.
/// Production Postgres uses `migration::Migrator` (sea-orm-migration).
pub async fn create_schema(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    for ddl in [
        DDL_VARS,
        DDL_VARS_IDX,
        DDL_SOFIA,
        DDL_DEFAULT_SETTINGS,
        DDL_DEFAULT_SETTINGS_IDX,
        DDL_DOMAIN_SETTINGS,
        DDL_DOMAIN_SETTINGS_IDX,
        DDL_DOMAIN_PLUGINS,
        DDL_DOMAIN_PLUGINS_IDX,
        DDL_DOMAIN_APPS,
        DDL_DOMAIN_APPS_IDX,
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
        DDL_USERS,
        DDL_USERS_IDX,
        DDL_GROUPS,
        DDL_GROUPS_IDX,
        DDL_USER_GROUPS,
        DDL_USER_GROUPS_IDX,
        DDL_PERMISSIONS,
        DDL_GROUP_PERMISSIONS,
        DDL_CONTACTS,
        DDL_CONTACTS_IDX,
        DDL_EMAIL_QUEUE,
        DDL_EMAIL_QUEUE_IDX,
        DDL_REMAINING_RECORDS,
        DDL_REMAINING_RECORDS_IDX,
        DDL_CALL_BLOCK,
        DDL_CALL_BLOCK_IDX,
        DDL_CALL_BROADCASTS,
        DDL_CALL_BROADCASTS_IDX,
        DDL_BRIDGES,
        DDL_BRIDGES_IDX,
        DDL_DESTINATIONS,
        DDL_DESTINATIONS_IDX,
        DDL_FOLLOW_ME,
        DDL_FOLLOW_ME_IDX,
        DDL_FOLLOW_ME_DESTINATIONS,
        DDL_FOLLOW_ME_DESTINATIONS_IDX,
        DDL_CALL_FORWARDS,
        DDL_CALL_FORWARDS_IDX,
        DDL_TIME_CONDITIONS,
        DDL_TIME_CONDITIONS_IDX,
        DDL_FEATURE_CODES,
        DDL_FEATURE_CODES_IDX,
        DDL_MUSIC_ON_HOLD,
        DDL_MUSIC_ON_HOLD_IDX,
        DDL_PHRASES,
        DDL_PHRASES_IDX,
        DDL_PHRASE_DETAILS,
        DDL_PHRASE_DETAILS_IDX,
        DDL_PIN_NUMBERS,
        DDL_PIN_NUMBERS_IDX,
        DDL_NUMBER_TRANSLATIONS,
        DDL_NUMBER_TRANSLATIONS_IDX,
        DDL_NUMBER_TRANSLATION_DETAILS,
        DDL_NUMBER_TRANSLATION_DETAILS_IDX,
        DDL_STREAMS,
        DDL_STREAMS_IDX,
        DDL_ACCESS_CONTROLS,
        DDL_ACCESS_CONTROLS_IDX,
        DDL_ACCESS_CONTROL_NODES,
        DDL_ACCESS_CONTROL_NODES_IDX,
        DDL_WEBRTC_SETTINGS,
        DDL_FIFO,
        DDL_FIFO_IDX,
        DDL_FIFO_MEMBERS,
        DDL_FIFO_MEMBERS_IDX,
        DDL_EMERGENCY_LOGS,
        DDL_EMERGENCY_LOGS_IDX,
        DDL_EVENT_GUARD_LOGS,
    ] {
        db.execute_unprepared(ddl).await?;
    }
    Ok(())
}

/// Postgres pool parity with `app/db.py:12-18`.
/// pool_size 5, max_overflow 10 (sqlx single max_connections 5 — overflow
/// folded in; pg backs pressure via acquire timeout), timeout 30, pre_ping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresPoolConfig {
    pub max_connections: u32,
    pub timeout_secs: u64,
    pub pre_ping: bool,
}

/// Pure pool knobs — unit-testable without a DB.
#[must_use]
pub fn postgres_connect_options(_url: &str) -> PostgresPoolConfig {
    PostgresPoolConfig {
        max_connections: 5,
        timeout_secs: 30,
        pre_ping: true,
    }
}

fn postgres_sea_options(url: &str) -> sea_orm::ConnectOptions {
    let cfg = postgres_connect_options(url);
    let mut opt = sea_orm::ConnectOptions::new(url.to_owned());
    opt.max_connections(cfg.max_connections)
        .connect_timeout(std::time::Duration::from_secs(cfg.timeout_secs))
        .test_before_acquire(cfg.pre_ping);
    opt
}

/// Connect to Postgres with parity pool opts. Caller runs `Migrator::up`.
pub async fn connect_postgres(url: &str) -> Result<sea_orm::DatabaseConnection, DbErr> {
    sea_orm::Database::connect(postgres_sea_options(url)).await
}
