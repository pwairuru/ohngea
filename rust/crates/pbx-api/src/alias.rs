//! PHP↔Rust field-name aliases (PHP `fusionpbx/` is ground truth).
//!
//! The React frontend uses PHP `v_*` column names (e.g. `queue_greeting`,
//! `conference_pin_number`); the Rust entities use stripped short names
//! (e.g. `greeting`, `pin_number`). `normalize` copies alias values onto
//! their canonical Rust keys (canonical wins when both present) so every
//! `PATCH`/untyped-create handler accepts both conventions. Typed creates
//! use `#[serde(alias = ...)]` for the same mapping; `ser_*` outputs emit
//! both keys so forms populate regardless of which name they read.

/// (alias, canonical) pairs. Keep in sync with `docs/php-schema-fields.txt`.
const ALIASES: &[(&str, &str)] = &[
    // call_centers queue (PHP v_call_center_queues queue_* minus the
    // six columns whose Rust names keep the prefix)
    ("queue_greeting", "greeting"),
    ("queue_moh_sound", "moh_sound"),
    ("queue_record_template", "record_template"),
    ("queue_language", "language"),
    ("queue_dialect", "dialect"),
    ("queue_voice", "voice"),
    ("queue_cid_prefix", "cid_prefix"),
    ("queue_outbound_caller_id_name", "outbound_caller_id_name"),
    (
        "queue_outbound_caller_id_number",
        "outbound_caller_id_number",
    ),
    ("queue_timeout_action", "timeout_action"),
    ("queue_max_wait_time", "max_wait_time"),
    (
        "queue_max_wait_time_with_no_agent",
        "max_wait_time_with_no_agent",
    ),
    (
        "queue_max_wait_time_with_no_agent_time_reached",
        "max_wait_time_with_no_agent_time_reached",
    ),
    ("queue_tier_rules_apply", "tier_rules_apply"),
    ("queue_tier_rule_wait_second", "tier_rule_wait_second"),
    (
        "queue_tier_rule_wait_multiply_level",
        "tier_rule_wait_multiply_level",
    ),
    (
        "queue_tier_rule_no_agent_no_wait",
        "tier_rule_no_agent_no_wait",
    ),
    ("queue_discard_abandoned_after", "discard_abandoned_after"),
    ("queue_abandoned_resume_allowed", "abandoned_resume_allowed"),
    ("queue_announce_position", "announce_position"),
    ("queue_announce_sound", "announce_sound"),
    ("queue_announce_frequency", "announce_frequency"),
    ("queue_cc_exit_keys", "cc_exit_keys"),
    ("queue_email_address", "email_address"),
    ("queue_context", "context"),
    // ring_groups
    ("ring_group_forward_enabled", "forward_enabled"),
    ("ring_group_forward_destination", "forward_destination"),
    ("timeout", "call_timeout"),
    ("destination_delay", "delay"),
    ("destination_timeout", "timeout"),
    // conferences
    ("conference_pin_number", "pin_number"),
    ("conference_email_address", "email_address"),
    ("conference_account_code", "account_code"),
    ("conference_flags", "flags"),
    ("conference_context", "context"),
    // sip_profiles (PHP v_sip_profiles prefixed names)
    ("sip_profile_name", "name"),
    ("sip_profile_enabled", "enabled"),
    ("sip_profile_hostname", "hostname"),
    ("sip_profile_description", "description"),
    // conference_rooms (frontend short names)
    ("room_name", "conference_room_name"),
    // voicemail_messages
    ("transcription", "message_transcription"),
    ("file_path", "file_name"),
    // voicemail_options
    ("order", "option_order"),
    // fax_logs
    ("result_code", "fax_result_code"),
    // fax_files
    ("file_name", "fax_file_path"),
    // recordings
    ("recording_name", "name"),
    ("recording_path", "file_path"),
    // ivr_menu_options (standalone creates)
    // (digits/action/param already match; option_order handled by callers)
    // voicemail forward copy (PHP voicemail_edit.php voicemail_destination
    // select creates v_voicemail_destinations rows with voicemail_uuid_copy)
    ("voicemail_destination", "voicemail_uuid_copy"),
    // user_logs PHP/Python superset (PHP: type/remote_address/timestamp;
    // Python: action/ip_address/created_at; canonical: log_type/remote_address)
    ("type", "log_type"),
    ("action", "log_type"),
    ("ip_address", "remote_address"),
    // contacts custom-label helpers (PHP custom text inputs map to base label)
    ("phone_label_custom", "phone_label"),
    ("email_label_custom", "email_label"),
    ("address_label_custom", "address_label"),
    ("relation_label_custom", "relation_label"),
    (
        "relation_reciprocal_label_custom",
        "relation_reciprocal_label",
    ),
    ("url_label_custom", "url_label"),
    // users PHP/Python superset (PHP user_email/user_enabled vs Python email/enabled)
    ("user_email", "email"),
    ("user_enabled", "enabled"),
];

/// Copy canonical values onto alias keys (only fills gaps) so forms
/// bound to PHP-style names populate from GET responses.
pub fn emit_aliases(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        for (alias, canon) in ALIASES {
            if obj.contains_key(*alias) {
                continue;
            }
            if let Some(val) = obj.get(*canon).cloned() {
                obj.insert((*alias).to_owned(), val);
            }
        }
    }
}

/// Copy alias values onto canonical keys (canonical wins on conflict).
/// Pure — unit-testable. Only maps objects; anything else passes through.
pub fn normalize(payload: serde_json::Value) -> serde_json::Value {
    let mut v = payload;
    if let Some(obj) = v.as_object_mut() {
        for (alias, canon) in ALIASES {
            if obj.contains_key(*canon) {
                continue;
            }
            if let Some(val) = obj.get(*alias).cloned() {
                obj.insert((*canon).to_owned(), val);
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn php_aliases_map_to_rust_keys() {
        let v = normalize(serde_json::json!({
            "queue_greeting": "greet.wav",
            "conference_pin_number": "1234",
            "result_code": "OK",
        }));
        assert_eq!(v["greeting"], "greet.wav");
        assert_eq!(v["pin_number"], "1234");
        assert_eq!(v["fax_result_code"], "OK");
    }

    #[test]
    fn canonical_wins_over_alias() {
        let v = normalize(serde_json::json!({
            "greeting": "canon.wav",
            "queue_greeting": "alias.wav",
        }));
        assert_eq!(v["greeting"], "canon.wav");
    }

    #[test]
    fn non_objects_pass_through() {
        let v = normalize(serde_json::json!([1, 2, 3]));
        assert_eq!(v, serde_json::json!([1, 2, 3]));
    }
}
