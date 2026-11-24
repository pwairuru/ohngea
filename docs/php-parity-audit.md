# PHP parity audit (generated — see /tmp/opencode/gapnorm.py)
# PHP ground truth: fusionpbx/**/ *_edit.php name='...' inputs
# React: fieldConfigs T()/C()/name: keys + custom route form.* keys
# API: Rust struct fields + payload.get()/s(&payload) keys + alias layer
# Names normalized: alias.rs pairs + prefix strip + verified semantic map.
# uuid/domain system keys excluded. Re-run: python3 /tmp/opencode/gapnorm.py
access_controls: php=3 react-miss=0 api-miss=1
   A: ['access_control_default']
bridges: php=7 react-miss=3 api-miss=3
   R: ['bridge_action', 'bridge_profile', 'destination_number']
   A: ['bridge_action', 'bridge_description', 'bridge_profile']
call_block: php=8 react-miss=0 api-miss=5
   A: ['action', 'call_block_action', 'call_block_country_code', 'call_block_description', 'call_block_direction']
call_broadcast: php=13 react-miss=0 api-miss=12
   A: ['broadcast_accountcode', 'broadcast_avmd', 'broadcast_caller_id_name', 'broadcast_caller_id_number', 'broadcast_concurrent_limit', 'broadcast_description', 'broadcast_destination_data', 'broadcast_destination_type', 'broadcast_phone_numbers', 'broadcast_start_time', 'broadcast_timeout', 'broadcast_toll_allow']
call_centers: php=39 react-miss=2 api-miss=1
   R: ['queue_limit', 'queue_record_enabled']
   A: ['queue_record_enabled']
call_flows: php=10 react-miss=0 api-miss=7
   A: ['call_flow_alternate_label', 'call_flow_context', 'call_flow_extension', 'call_flow_feature_code', 'call_flow_label', 'call_flow_pin_number', 'call_flow_status']
call_forward: php=14 react-miss=6 api-miss=5
   R: ['cid_name_prefix', 'cid_number_prefix', 'do_not_disturb', 'follow_me_enabled', 'follow_me_ignore_busy', 'id']
   A: ['cid_name_prefix', 'cid_number_prefix', 'do_not_disturb', 'follow_me_ignore_busy', 'id']
conference_centers: php=23 react-miss=2 api-miss=9
   R: ['announce_recording', 'enter_sound']
   A: ['account_code', 'announce_count', 'announce_name', 'announce_recording', 'enter_sound', 'moderator_endconf', 'mute', 'start_datetime', 'stop_datetime']
conference_controls: php=6 react-miss=0 api-miss=0
conference_profiles: php=7 react-miss=0 api-miss=0
conferences: php=11 react-miss=0 api-miss=8
   A: ['conference_account_code', 'conference_context', 'conference_description', 'conference_email_address', 'conference_enabled', 'conference_flags', 'conference_order', 'conference_pin_number']
destinations: php=22 react-miss=7 api-miss=18
   R: ['db_destination_number', 'destination_condition_field', 'destination_distinctive_ring', 'destination_type_emergency', 'destination_type_fax', 'destination_type_text', 'destination_type_voice']
   A: ['db_destination_number', 'destination_accountcode', 'destination_area_code', 'destination_caller_id_name', 'destination_caller_id_number', 'destination_cid_name_prefix', 'destination_condition_field', 'destination_context', 'destination_description', 'destination_distinctive_ring', 'destination_email', 'destination_prefix', 'destination_record', 'destination_trunk_prefix', 'destination_type_emergency', 'destination_type_fax', 'destination_type_text', 'destination_type_voice']
devices: php=19 react-miss=5 api-miss=4
   R: ['action', 'device_uuid_alternate', 'name', 'subtype', 'type']
   A: ['action', 'name', 'subtype', 'type']
dialplans: php=10 react-miss=1 api-miss=1
   R: ['dialplan_details_order']
   A: ['dialplan_details_order']
email_queue: php=10 react-miss=1 api-miss=3
   R: ['email_response']
   A: ['email_date', 'email_response', 'email_retry_count']
event_guard: php=7 react-miss=0 api-miss=7
   A: ['extension', 'filter', 'hostname', 'ip_address', 'log_date', 'log_status', 'user_agent']
extension_settings: php=5 react-miss=0 api-miss=2
   A: ['extension_setting_type', 'extension_setting_value']
extensions: php=49 react-miss=8 api-miss=6
   R: ['id', 'order', 'voicemail_file', 'voicemail_local_after_email', 'voicemail_mail_to', 'voicemail_transcription_enabled', 'voicemail_transcription_prompt', 'voicemail_transcription_prompt_enabled']
   A: ['voicemail_file', 'voicemail_local_after_email', 'voicemail_mail_to', 'voicemail_transcription_enabled', 'voicemail_transcription_prompt', 'voicemail_transcription_prompt_enabled']
fax: php=12 react-miss=1 api-miss=3
   R: ['fax_file']
   A: ['fax_file', 'fax_send_channels', 'fax_toll_allow']
fifo: php=10 react-miss=0 api-miss=7
   A: ['fifo_agent_queue', 'fifo_agent_status', 'fifo_description', 'fifo_exit_key', 'fifo_order', 'fifo_strategy', 'fifo_timeout_seconds']
gateways: php=32 react-miss=0 api-miss=0
ivr_menus: php=19 react-miss=4 api-miss=0
   R: ['ivr_menu_cid_prefix', 'ivr_menu_confirm_attempts', 'ivr_menu_language', 'ivr_menu_pin_number']
modules: php=6 react-miss=0 api-miss=2
   A: ['module_default_enabled', 'module_label']
music_on_hold: php=10 react-miss=10 api-miss=9
   R: ['music_on_hold_channels', 'music_on_hold_chime_freq', 'music_on_hold_chime_list', 'music_on_hold_chime_max', 'music_on_hold_interval', 'music_on_hold_name', 'music_on_hold_path', 'music_on_hold_rate', 'music_on_hold_shuffle', 'music_on_hold_timer_name']
   A: ['music_on_hold_channels', 'music_on_hold_chime_freq', 'music_on_hold_chime_list', 'music_on_hold_chime_max', 'music_on_hold_interval', 'music_on_hold_path', 'music_on_hold_rate', 'music_on_hold_shuffle', 'music_on_hold_timer_name']
number_translations: php=3 react-miss=0 api-miss=1
   A: ['number_translation_description']
phrases: php=8 react-miss=1 api-miss=5
   R: ['phrase_details_order']
   A: ['phrase_description', 'phrase_detail_data', 'phrase_detail_order', 'phrase_details_order', 'phrase_language']
pin_numbers: php=4 react-miss=0 api-miss=2
   A: ['accountcode', 'description']
recordings: php=10 react-miss=6 api-miss=6
   R: ['recording_description', 'recording_filename', 'recording_filename_original', 'recording_language', 'recording_model', 'recording_voice_source']
   A: ['recording_description', 'recording_filename', 'recording_filename_original', 'recording_language', 'recording_model', 'recording_voice_source']
ring_groups: php=22 react-miss=0 api-miss=0
sip_profiles: php=4 react-miss=4 api-miss=1
   R: ['sip_profile_description', 'sip_profile_enabled', 'sip_profile_hostname', 'sip_profile_name']
   A: ['sip_profile_hostname']
sofia_global_settings: php=4 react-miss=0 api-miss=2
   A: ['global_setting_description', 'global_setting_value']
streams: php=4 react-miss=0 api-miss=2
   A: ['stream_description', 'stream_location']
time_conditions: php=6 react-miss=2 api-miss=3
   R: ['dialplan_number', 'dialplan_order']
   A: ['dialplan_context', 'dialplan_description', 'dialplan_order']
vars: php=7 react-miss=0 api-miss=0
voicemail_greetings: php=9 react-miss=3 api-miss=5
   R: ['greeting_language', 'greeting_model', 'greeting_voice_source']
   A: ['greeting_language', 'greeting_message', 'greeting_model', 'greeting_voice', 'greeting_voice_source']
voicemails: php=21 react-miss=6 api-miss=8
   R: ['alternate_voicemail_enabled', 'recorded_name', 'voicemail_destination', 'voicemail_id_previous', 'voicemail_recording_options', 'voicemail_transcription_prompt_enabled']
   A: ['alternate_voicemail_enabled', 'recorded_name', 'voicemail_destination', 'voicemail_id_previous', 'voicemail_recording_instructions', 'voicemail_recording_options', 'voicemail_transcription_prompt', 'voicemail_transcription_prompt_enabled']
