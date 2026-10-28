// Batch 1 settings first, then batch 2a telephony in dependency order:
// domains → extensions → dialplans → gateways → sip → devices.
pub mod default_settings;
pub mod device_keys;
pub mod device_lines;
pub mod device_profile_keys;
pub mod device_profile_settings;
pub mod device_profiles;
pub mod device_settings;
pub mod devices;
pub mod dialplan_details;
pub mod dialplans;
pub mod domain_settings;
pub mod domains;
pub mod extensions;
pub mod gateways;
pub mod sip_profile_domains;
pub mod sip_profile_settings;
pub mod sip_profiles;
pub mod sofia_global_settings;
pub mod vars;
