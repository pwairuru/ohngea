# Field Parity Audit — Legacy Ohngea vs Modern

Date: 2026-08-22
Scope: Switch-relevant apps that affect FreeSWITCH XML / behaviour.

Legend: ✅ parity, ⚠️ partial, ❌ missing (legacy field not in new model)

## Extensions — ✅ FIXED in 021_extension_full_parity

| Legacy field | New model | Notes |
|---|---|---|
| extension | ✅ | 2-7 digits validator |
| number_alias | ✅ |  |
| password | ✅ **NEW** | was hardcoded 1234; now generated 20 chars if blank |
| accountcode | ✅ NEW |  |
| effective_caller_id_name/number | ✅ |  |
| outbound_caller_id_name/number | ✅ NEW |  |
| emergency_caller_id_name/number | ✅ NEW |  |
| directory_first/last_name, directory_visible/exten_visible | ✅ NEW |  |
| max_registrations, limit_max/destination | ✅ NEW | defaults 5 / !USER_BUSY |
| missed_call_app/data | ✅ NEW | hidden advanced |
| user_context | ✅ NEW | default domain_name |
| toll_allow, call_timeout, call_group, call_screen_enabled, user_record, hold_music | ✅ NEW |  |
| auth_acl, cidr, sip_force_contact/expires, nibble/mwi/sip_bypass_media, absolute_codec_string, force_ping, dial_string | ✅ NEW | all in hidden SIP Advanced |
| extension_language/dialect/voice, extension_type | ✅ NEW |  |
| do_not_disturb, forward_* (4 pairs), follow_me_uuid/enabled | ✅ NEW |  |
| enabled, description | ✅ |  |
| FS XML | ✅ | xml_builder.php:335-494 parity, cidr+number-alias attrs, vm-*, dial-string, variables |
| Cache | ✅ | service/router now invalidate `xml:directory:<domain>` + wildcard `xml:directory:*@<domain>` + dialplan, plus ESL reloadxml |
| UI | ✅ | Primary 4 fields visible; 50+ hidden behind Show Advanced / details; Verify FS XML button fetches `GET /api/internal/freeswitch/xml?section=directory&key_value=ext@domain` with Basic |

All 60 legacy columns now present. Old `unique_id`/`dial_user`/`dial_domain`/`insert_user` dropped as derived.

---

## Gateways — ✅ FIXED in 022_gateways_full

## Devices/Dialplans/SIP/IVR — ✅ FIXED in 023

| Entity | Legacy fields | New | Status |
|---|---|---|---|
| devices | device_address/label/vendor/model/firmware/template/location/serial/description/username/password + device_lines (server_address, user_id, auth_id, password, sip_port...) | ✅ expanded models + device_lines table | hidden advanced via API, provision `/provision/{mac}.cfg` uses DB |
| dialplans | dialplan_continue/order, hostname, app_uuid + details | ✅ added | FS dialplan XML `app/dialplans` already supports details; continue/order now honoured |
| sip_profiles | sip_secure, rtp_ip, hold_music, ACLs, record | ✅ expanded | FS sofia config via `GET /xml?section=configuration&key_value=sofia.conf` |
| ivr_menus | greet_long/short, invalid/exit, confirm, timeout, tts | ✅ expanded | FS ivr config via DB |

| Legacy | Modern (now 36 fields) | Status |
|---|---|---|
| gateway, username, password, proxy, register, enabled | ✅ |  |
| distinct_to, auth_username, realm, from_user/domain, register_proxy, outbound_proxy, expire_seconds, register_transport, contact_params, retry_seconds, extension, ping/min/max, contact_in_ping, caller_id_in_from, supress_cng, sip_cid_type, codec_prefs, channels, extension_in_contact, context, profile, hostname, description | ❌ | 30 fields missing; affects sofia gateway XML generation (not yet implemented via xml_curl configuration section) |

Fixed: all 30 legacy fields now in model + gateway sofia XML via xml_curl configuration, UI hidden Advanced 30 fields, Verify Sofia XML button.

## Dialplans — ⚠️ partial

Modern: name, context, enabled, domain_id, description + details(tag/type/data/order). Missing: dialplan_continue, dialplan_order (global), hostname, app_uuid, anti-action handling partially done. Details table already flexible enough; main missing is `continue` flag and ordering across dialplans.

## Devices — ⚠️ thin

Modern: mac, vendor, model, label, enabled, domain_id, extension_uuid (1 FK). Legacy: device_address, vendor, template, label, enabled, plus **device_lines** (user_id, auth_id, password, line_number, server_address, display_name, etc.) and **device_keys**, **device_settings**. Provision files generation (`provision.php`) not yet ported; devices table alone insufficient for FS provisioning.

## SIP Profiles — ❌ thin

Modern: name, sip_port, sip_ip, enabled, domain_id. Legacy: dozens (sip_port, sip_ip, rtp_ip, hold_music, etc.). Currently FS uses static `docker/freeswitch/conf/autoload_configs/sofia.conf.xml`, not DB-driven. Needs expansion if DB-driven sofia profile XML desired.

## IVR Menus — ⚠️ thin

Modern: name, extension, enabled, domain_id + options (digits/action/param/order). Legacy adds: greet_long/short, invalid_sound, exit_sound, timeout, etc. Basic IVR works; transcription options missing.

## Ring Groups — ✅ mostly good (recent 003)

Has strategy enum, timeout, members with delay/timeout. Legacy also: ring_group_forward_*, missed_call, etc. Acceptable.

## Voicemails — ⚠️

Modern has voicemail_id/password/enabled/greeting_id/toll_allow/mail_to/file etc but legacy also: transcription, local_after_email, tutorial flag. Our 005 added most; transcription fields pending.

## Verifying FreeSWITCH Actually Uses DB Data

### 1. Curl the XML curl endpoint (what FS calls)
```bash
# create extension
curl -X POST http://localhost:8000/api/v1/extensions -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"extension":"1009","effective_caller_id_name":"Verify","password":"Test123!Test123!Test0","call_timeout":30,"toll_allow":"domestic","forward_all_destination":"2000","forward_all_enabled":true}'

# fetch what FS sees
curl -u freeswitch:ClueCon "http://localhost:8000/api/internal/freeswitch/xml?section=directory&key_value=1009@test.local" | grep -E "password|toll_allow|forward_all|call_timeout"

# domain-wide
curl -u freeswitch:ClueCon "http://localhost:8000/api/internal/freeswitch/xml?section=directory&key_value=test.local"
```
Expected: `<param name="password" value="Test123!..."/>` not 1234; `<variable name="toll_allow" value="domestic"/>` etc.

### 2. Check Valkey cache invalidation
```bash
redis-cli KEYS "xml:*"
# after POST/PATCH/DELETE, keys `xml:directory:test.local` and `xml:directory:1009@test.local` should be gone (TTL 60 otherwise)
valkey-cli --scan --pattern "xml:*"
```
Logs: `docker compose logs api | grep freeswitch`

### 3. Live SIP registration (E2E)
```bash
make e2e-up   # seeds 1001/1002 with passwords from API (now real, not 1234 unless fallback)
# check FS
docker compose -f docker-compose.yml -f docker-compose.e2e.yml exec freeswitch fs_cli -x "sofia status"
docker compose exec freeswitch fs_cli -x "sofia xmlstatus"
# sipp registers with password from DB (update docker/sipp/*.csv if not 1234)
make e2e-call  # originate 1001->1002, verify CDR via GET /api/v1/cdr
```

### 4. Frontend Verify button
Extensions table now has Eye icon → fetches directory XML for that extension and shows snippet in Card below table.

---

## Next Steps (phased)

- **022_gateways_full**: expand gateways model + migration + sofia gateway XML (configuration section) + UI hidden advanced.
- **023_devices_provision**: device_lines/settings/keys tables + provision xml builder.
- **024_sip_profiles_ivr**: expand sip_profiles, ivr_menus with full legacy fields + FS configuration XML.
- All follow same pattern: add nullable columns, update xml_builder, hide in UI details, invalidate cache.
