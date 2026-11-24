# PHP → Rust port parity matrix

Generated evidence: every fusionpbx `app/*` + `core/*` module mapped to Rust routers, routes, permission gates, and tables. 111 Rust route prefixes total.

| PHP module | PHP perms | PHP tables (→ Rust entity [RLS]) | Rust routes | Rust files | Rust gates (sample) | note |
|---|---|---|---|---|---|---|
| aastra | 0 | — | — | — |  | vendor metadata only (app_config, no tables/routes) |
| access_controls | 8 | v_access_control_nodes→access_control_nodes.rs[STRICT], v_access_controls→access_controls.rs[STRICT] | access_controls | remaining_c.rs | access_control_add,access_control_delete,access_control_edit,access_control_view,emergency_logs_add |  |
| active_calls | 106 | — | calls | calls.rs |  |  |
| active_conferences | 10 | — | active_conferences | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| algo | 0 | — | — | — |  | vendor metadata only |
| avaya | 0 | — | — | — |  | vendor metadata only |
| azure | 0 | — | — | — |  | vendor metadata only |
| basic_operator_panel | 8 | — | operator_panel | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| bridges | 7 | v_bridges→bridges.rs[STRICT] | bridges | remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| call_block | 11 | v_call_block→call_block.rs[NULLABLE] | call_block | lib.rs,remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| call_broadcast | 15 | v_call_broadcasts→call_broadcasts.rs[STRICT] | call_broadcast | remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| call_center_active | 2 | — | call_centers | call_centers.rs,lib.rs | call_center_active_view,call_center_add,call_center_view |  |
| call_centers | 25 | v_call_center_agents→call_center_agents.rs[STRICT], v_call_center_queues→MISSING[open], v_call_center_tiers→call_center_tiers.rs[STRICT] | call_centers | call_centers.rs,lib.rs | call_center_active_view,call_center_add,call_center_view |  |
| call_flows | 7 | v_call_flows→call_flows.rs[NULLABLE] | call_flows | admin.rs,lib.rs | call_flow_add,call_flow_view,call_recording_add,call_recording_view,extension_add |  |
| call_forward | 4 | — | call_forward | lib.rs,remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| call_recordings | 8 | — | call_recordings | admin.rs | call_flow_add,call_flow_view,call_recording_add,call_recording_view,extension_add |  |
| calls_active | 11 | — | calls,active_calls | calls.rs,remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| cisco | 0 | — | — | — |  | vendor metadata only |
| click_to_call | 2 | — | click_to_call | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| conference_centers | 35 | v_conference_centers→conference_centers.rs[STRICT], v_conference_room_users→conference_room_users.rs[STRICT], v_conference_rooms→conference_rooms.rs[STRICT], v_conference_session_details→conference_session_details.rs[STRICT], v_conference_sessions→conference_sessions.rs[STRICT] | conference_centers | conference_centers.rs | conference_center_add,conference_center_delete,conference_center_edit,conference_center_view,conference_room_add |  |
| conference_controls | 8 | v_conference_control_details→conference_control_details.rs[open], v_conference_controls→conference_controls.rs[open] | conference_controls | conference_controls.rs | conference_control_add,conference_control_view,conference_profile_add,conference_profile_view |  |
| conference_profiles | 8 | v_conference_profile_params→conference_profile_params.rs[open], v_conference_profiles→conference_profiles.rs[open] | conference_profiles | conference_controls.rs | conference_control_add,conference_control_view,conference_profile_add,conference_profile_view |  |
| conferences | 13 | v_conference_users→conference_users.rs[STRICT], v_conferences→conferences.rs[NULLABLE] | conferences | conferences.rs,lib.rs | conference_add,conference_view |  |
| conferences_active | 10 | — | conferences | conferences.rs,lib.rs | conference_add,conference_view |  |
| database_transactions | 4 | v_database_transactions→database_transactions.rs[STRICT] | database_transactions | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| destinations | 32 | v_destinations→destinations.rs[NULLABLE] | destinations | lib.rs,remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| devices | 88 | v_device_keys→device_keys.rs[STRICT], v_device_lines→device_lines.rs[STRICT], v_device_profile_keys→device_profile_keys.rs[STRICT], v_device_profile_settings→device_profile_settings.rs[STRICT], v_device_profiles→device_profiles.rs[STRICT], v_device_settings→device_settings.rs[STRICT], v_device_vendor_function_groups→device_vendor_function_groups.rs[open], v_device_vendor_functions→device_vendor_functions.rs[open], v_device_vendors→device_vendors.rs[open], v_devices→devices.rs[NULLABLE] | devices | devices.rs,lib.rs | device_add,device_delete,device_edit,device_profile_view,device_view |  |
| dialplan_inbound | 6 | — | dialplans | dialplans.rs,lib.rs,remaining.rs | active_calls_hangup,conference_active_view,dialplan_add,dialplan_delete,dialplan_edit |  |
| dialplan_outbound | 8 | — | dialplans | dialplans.rs,lib.rs,remaining.rs | active_calls_hangup,conference_active_view,dialplan_add,dialplan_delete,dialplan_edit |  |
| dialplans | 14 | v_dialplan_details→dialplan_details.rs[NULLABLE], v_dialplans→dialplans.rs[NULLABLE] | dialplans | dialplans.rs,lib.rs,remaining.rs | active_calls_hangup,conference_active_view,dialplan_add,dialplan_delete,dialplan_edit |  |
| domain_limits | 0 | — | domain_limits | admin.rs | call_flow_add,call_flow_view,call_recording_add,call_recording_view,extension_add |  |
| email_queue | 10 | v_email_queue→email_queue.rs[STRICT], v_email_queue_attachments→email_queue_attachments.rs[STRICT] | email_queue | auth.rs,email_queue.rs,lib.rs | email_queue_add,email_queue_delete,email_queue_edit,email_queue_view,email_template_add |  |
| emergency | 2 | v_emergency_logs→emergency_logs.rs[STRICT] | emergency_logs | remaining_c.rs | access_control_add,access_control_delete,access_control_edit,access_control_view,emergency_logs_add |  |
| event_guard | 6 | v_event_guard_logs→event_guard_logs.rs[open] | event_guard | remaining_c.rs | access_control_add,access_control_delete,access_control_edit,access_control_view,emergency_logs_add |  |
| extension_settings | 5 | v_extension_settings→extension_settings.rs[open] | extension_settings | admin.rs | call_flow_add,call_flow_view,call_recording_add,call_recording_view,extension_add |  |
| extensions | 50 | v_extension_users→extension_users.rs[STRICT], v_extensions→extensions.rs[STRICT] | extensions | extensions.rs,lib.rs | extension_add,extension_view |  |
| fanvil | 0 | — | — | — |  | vendor metadata only |
| fax | 46 | v_fax→faxes.rs[NULLABLE], v_fax_files→fax_files.rs[STRICT], v_fax_logs→fax_logs.rs[STRICT], v_fax_users→fax_users.rs[STRICT] | fax | fax.rs,lib.rs | fax_add,fax_queue_add,fax_queue_view,fax_view |  |
| fax_queue | 6 | v_fax_queue→fax_queue.rs[STRICT] | fax_queue | fax.rs | fax_add,fax_queue_add,fax_queue_view,fax_view |  |
| feature_codes | 3 | — | feature_codes | remaining_b.rs | feature_code_add,feature_code_view,music_on_hold_add,music_on_hold_delete,music_on_hold_edit |  |
| fifo | 11 | v_fifo→fifo.rs[STRICT], v_fifo_members→fifo_members.rs[STRICT] | fifo | remaining_c.rs | access_control_add,access_control_delete,access_control_edit,access_control_view,emergency_logs_add |  |
| fifo_list | 4 | — | fifo_list | remaining_c.rs | access_control_add,access_control_delete,access_control_edit,access_control_view,emergency_logs_add |  |
| flyingvoice | 0 | — | — | — |  | vendor metadata only |
| follow_me | 12 | v_follow_me→follow_me.rs[STRICT], v_follow_me_destinations→follow_me_destinations.rs[STRICT] | follow_me | lib.rs,remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| gateways | 8 | v_gateways→gateways.rs[NULLABLE] | gateways | gateways.rs,lib.rs | gateway_add,gateway_delete,gateway_edit,gateway_view |  |
| grandstream | 0 | — | — | — |  | vendor metadata only |
| htek | 0 | — | — | — |  | vendor metadata only |
| ivr_menus | 15 | v_ivr_menu_options→ivr_menu_options.rs[STRICT], v_ivr_menus→ivr_menus.rs[NULLABLE] | ivr_menus | ivr_menus.rs,lib.rs | ivr_menu_add,ivr_menu_delete,ivr_menu_edit,ivr_menu_view |  |
| log_viewer | 3 | — | logs,log_viewer | lib.rs,logs.rs,remaining.rs | active_calls_hangup,conference_active_view,tone_destinations,user_log_add,user_log_delete |  |
| modules | 4 | v_modules→modules.rs[open] | modules | missing_tables.rs |  |  |
| music_on_hold | 10 | v_music_on_hold→music_on_hold.rs[NULLABLE] | music_on_hold | lib.rs,remaining_b.rs | feature_code_add,feature_code_view,music_on_hold_add,music_on_hold_delete,music_on_hold_edit |  |
| number_translations | 9 | v_number_translation_details→number_translation_details.rs[STRICT], v_number_translations→number_translations.rs[STRICT] | number_translations | remaining_b.rs | feature_code_add,feature_code_view,music_on_hold_add,music_on_hold_delete,music_on_hold_edit |  |
| operator_panel | 16 | — | operator_panel | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| phrases | 9 | v_phrase_details→phrase_details.rs[STRICT], v_phrases→phrases.rs[NULLABLE] | phrases | remaining_b.rs | feature_code_add,feature_code_view,music_on_hold_add,music_on_hold_delete,music_on_hold_edit |  |
| pin_numbers | 4 | v_pin_numbers→pin_numbers.rs[STRICT] | pin_numbers | remaining_b.rs | feature_code_add,feature_code_view,music_on_hold_add,music_on_hold_delete,music_on_hold_edit |  |
| poly | 0 | — | — | — |  | vendor metadata only |
| polycom | 0 | — | — | — |  | vendor metadata only |
| provision | 0 | — | provision | — |  | GET /provision/{mac}.cfg vendor-matrix render |
| recordings | 12 | v_recordings→recordings.rs[STRICT] | recordings | lib.rs,recordings.rs | call_recording_add,call_recording_delete,call_recording_edit,call_recording_view,recording_add |  |
| registrations | 3 | — | registrations | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| ring_groups | 25 | v_ring_group_destinations→MISSING[open], v_ring_group_users→ring_group_users.rs[STRICT], v_ring_groups→ring_groups.rs[STRICT] | ring_groups | lib.rs,ring_groups.rs | ring_group_add,ring_group_delete,ring_group_edit,ring_group_view |  |
| sip_profiles | 12 | v_sip_profile_domains→sip_profile_domains.rs[STRICT], v_sip_profile_settings→sip_profile_settings.rs[STRICT], v_sip_profiles→sip_profiles.rs[STRICT] | sip_profiles | fs_status.rs,lib.rs,remaining.rs,sip.rs | active_calls_hangup,conference_active_view,sip_profile_add,sip_profile_delete,sip_profile_domain_add |  |
| sip_status | 5 | — | sip_status | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| snom | 0 | — | — | — |  | vendor metadata only |
| sofia_global_settings | 5 | v_sofia_global_settings→sofia_global_settings.rs[open] | sofia_global_settings | admin.rs | call_flow_add,call_flow_view,call_recording_add,call_recording_view,extension_add |  |
| streams | 6 | v_streams→streams.rs[NULLABLE] | streams | remaining_b.rs | feature_code_add,feature_code_view,music_on_hold_add,music_on_hold_delete,music_on_hold_edit |  |
| swissvoice | 0 | — | — | — |  | vendor metadata only |
| switch | 5 | — | switch | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| system | 13 | — | system | lib.rs,remaining.rs,system.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| time_conditions | 8 | — | time_conditions | lib.rs,remaining_a.rs | bridge_add,bridge_view,call_block_add,call_block_view,call_broadcast_add |  |
| tones | 1 | — | tones | remaining.rs | active_calls_hangup,conference_active_view,tone_destinations |  |
| vars | 4 | v_vars→vars.rs[STRICT] | vars | lib.rs,vars_fs.rs | var_add,var_delete,var_edit,var_view |  |
| voicemail_greetings | 7 | v_voicemail_greetings→voicemail_greetings.rs[STRICT] | voicemail_greetings | voicemails.rs | voicemail_add,voicemail_view |  |
| voicemails | 28 | v_voicemail_destinations→voicemail_destinations.rs[STRICT], v_voicemail_messages→voicemail_messages.rs[STRICT], v_voicemail_options→voicemail_options.rs[STRICT], v_voicemails→voicemails.rs[NULLABLE] | voicemails | lib.rs,voicemails.rs | voicemail_add,voicemail_view |  |
| xml_cdr | 80 | v_xml_cdr→MISSING[open], v_xml_cdr_extensions→xml_cdr_extensions.rs[STRICT], v_xml_cdr_flow→xml_cdr_flow.rs[STRICT], v_xml_cdr_json→xml_cdr_json.rs[STRICT], v_xml_cdr_logs→xml_cdr_logs.rs[STRICT], v_xml_cdr_transcripts→xml_cdr_transcripts.rs[STRICT] | cdr,xml_cdr | cdr.rs,lib.rs,remaining.rs | active_calls_hangup,cdr_add,cdr_view,conference_active_view,tone_destinations |  |
| yealink | 0 | — | — | — |  | vendor metadata only |
| core/authentication | 0 | — | auth | auth.rs,lib.rs | user_edit |  |
| core/contacts | 52 | v_contact_addresses→contact_addresses.rs[STRICT], v_contact_attachments→contact_attachments.rs[STRICT], v_contact_emails→contact_emails.rs[STRICT], v_contact_groups→contact_groups.rs[STRICT], v_contact_notes→contact_notes.rs[STRICT], v_contact_phones→contact_phones.rs[STRICT], v_contact_relations→contact_relations.rs[STRICT], v_contact_settings→contact_settings.rs[STRICT], v_contact_times→contact_times.rs[STRICT], v_contact_urls→contact_urls.rs[STRICT], v_contact_users→contact_users.rs[STRICT], v_contacts→contacts.rs[STRICT] | contacts | contacts.rs,lib.rs | contact_add,contact_delete,contact_edit,contact_view |  |
| core/dashboard | 15 | v_dashboard_widget_groups→MISSING[open], v_dashboard_widgets→dashboard_widgets.rs[open], v_dashboards→dashboards.rs[open] | dashboards | dashboard.rs |  |  |
| core/databases | 4 | v_databases→databases.rs[open] | databases | menus.rs |  |  |
| core/default_settings | 5 | v_countries→countries.rs[open], v_default_settings→default_settings.rs[open], v_languages→languages.rs[open] | default_settings | lib.rs,settings.rs | default_setting_add,default_setting_delete,default_setting_edit,default_setting_view,domain_setting_add |  |
| core/domain_settings | 6 | v_domain_settings→domain_settings.rs[NULLABLE] | domain_settings | lib.rs,settings.rs | default_setting_add,default_setting_delete,default_setting_edit,default_setting_view,domain_setting_add |  |
| core/domains | 7 | v_domains→domains.rs[open] | domains | domains.rs,lib.rs,plugins.rs | domain_add,domain_delete,domain_edit,domain_view |  |
| core/email_templates | 5 | v_email_templates→email_templates.rs[open] | email_templates | email_queue.rs | email_queue_add,email_queue_delete,email_queue_edit,email_queue_view,email_template_add |  |
| core/events | 0 | — | — | — |  | internal plugin bus (events class, no HTTP surface) |
| core/groups | 16 | v_group_permissions→group_permissions.rs[open], v_groups→groups.rs[NULLABLE] | groups | groups.rs,lib.rs | group_add,group_delete,group_edit,group_permission_edit,group_permission_view |  |
| core/install | 1 | — | domains | domains.rs,lib.rs,plugins.rs | domain_add,domain_delete,domain_edit,domain_view | first-install via POST /api/v1/domains/bootstrap |
| core/menu | 17 | v_menu_item_groups→menu_item_groups.rs[open], v_menu_items→menu_items.rs[open], v_menu_languages→menu_languages.rs[open], v_menus→menus.rs[open] | — | — |  |  |
| core/notifications | 0 | v_notifications→notifications.rs[open] | notifications | menus.rs |  |  |
| core/permissions | 5 | v_permissions→permissions.rs[open] | permissions | lib.rs,permissions.rs | group_permission_view |  |
| core/services | 5 | v_services→services.rs[open] | services | lib.rs,services.rs | service_add,service_delete,service_edit,service_view |  |
| core/software | 0 | v_software→software.rs[open] | software | menus.rs |  |  |
| core/upgrade | 4 | — | — | — |  | schema upgrades via Migrator (fresh) + alembic (prod) |
| core/user_logs | 3 | v_user_logs→user_logs.rs[open] | user_logs | logs.rs | user_log_add,user_log_delete,user_log_view |  |
| core/user_settings | 5 | — | users | auth.rs,lib.rs,users.rs | user_add,user_delete,user_edit,user_setting_view,user_view | GET/PUT /api/v1/users/{id}/settings |
| core/users | 17 | v_user_settings→user_settings.rs[open], v_users→users.rs[NULLABLE] | users | auth.rs,lib.rs,users.rs | user_add,user_delete,user_edit,user_setting_view,user_view |  |
| core/websockets | 0 | — | — | — |  | transport replaced by REST + ESL; surface = operator_panel |

## Rename-table notes (same data, different names)

- `call_centers`: v_call_center_queues
- `cdr`: v_xml_cdr
- `ring_group_members`: v_ring_group_destinations (VIEW)
- `domain_apps`: Rust-only (plugin system)
- `domain_plugins`: Rust-only (plugin system)
- `cdr_archive`: Rust-only (CDR archiving)
- `remaining_records`: Rust-only (migration spillover)
