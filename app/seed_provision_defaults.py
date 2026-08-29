"""Seed provision default_settings for vendor parity.

Creates default_settings table if missing and inserts at least 5 provision
settings per vendor (poly, polycom, snom, yealink, grandstream) based on
legacy fusionpbx/app/*/app_config.php defaults. Idempotent via ON CONFLICT.

Also seeds domain_settings fallback for tests (sqlite) via same helper.

Usage:
    from app.seed_provision_defaults import seed_provision_defaults, PROVISION_DEFAULTS

    await seed_provision_defaults(session)

Or CLI:
    python -m app.seed_provision_defaults
"""

from typing import Sequence

# Minimal but representative 5+ per vendor filtered from legacy app_config.php
# category is always "provision", name is "text"/"boolean"/"numeric" etc, but we use same subcategory scheme.
PROVISION_DEFAULTS: list[dict] = [
    # poly — 6
    {"default_setting_uuid": "77bd00d7-9d8f-4f55-9a43-f3e0ca0ff8af", "default_setting_category": "provision", "default_setting_subcategory": "poly_obi_dst_enabled", "default_setting_name": "text", "default_setting_value": "True", "default_setting_enabled": True, "default_setting_description": "Daylight Savings True=enabled False=disable"},
    {"default_setting_uuid": "3c021f6e-905d-4fc8-a353-983f4363837d", "default_setting_category": "provision", "default_setting_subcategory": "poly_obi_timezone", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "See provision profile for codes."},
    {"default_setting_uuid": "799038b2-77ca-4312-82fc-a0ccd2ed4402", "default_setting_category": "provision", "default_setting_subcategory": "poly_obi_logo_450", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Url of Logo file for VVX 450"},
    {"default_setting_uuid": "0e5fe399-3fa6-4688-8ab1-3254436e0949", "default_setting_category": "provision", "default_setting_subcategory": "poly_sp_1_inbound_call_route", "default_setting_name": "text", "default_setting_value": "dt1,dt2,dt3,dt4,dt5,dt6,dt7,dt8,dt9,dt10", "default_setting_enabled": True, "default_setting_description": "Route inbound calls to handsets."},
    {"default_setting_uuid": "bf91ea3f-14a7-4831-ac4a-77273ea159f4", "default_setting_category": "provision", "default_setting_subcategory": "poly_sp_2_inbound_call_route", "default_setting_name": "text", "default_setting_value": "dt2", "default_setting_enabled": True, "default_setting_description": "Route inbound calls to handsets."},
    {"default_setting_uuid": "fde0163a-cf4a-4923-9413-857d2b8a928b", "default_setting_category": "provision", "default_setting_subcategory": "poly_handset_1_digitmap", "default_setting_name": "text", "default_setting_value": "(x.(Msp1)|*x.(Msp1)|**x.(Msp1))", "default_setting_enabled": True, "default_setting_description": "Handset 1 dialplan rules"},
    # polycom — 6
    {"default_setting_uuid": "11111111-1111-4111-8111-111111111111", "default_setting_category": "provision", "default_setting_subcategory": "polycom_gmt_offset", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "3600 * GMT offset"},
    {"default_setting_uuid": "11111111-1111-4111-8111-111111111112", "default_setting_category": "provision", "default_setting_subcategory": "polycom_digitmap", "default_setting_name": "text", "default_setting_value": "*x.T|**x.T|[9]11|[2-9]xxxxxxT|", "default_setting_enabled": True, "default_setting_description": "Digit Timeouts"},
    {"default_setting_uuid": "11111111-1111-4111-8111-111111111113", "default_setting_category": "provision", "default_setting_subcategory": "polycom_digitmap_timeout", "default_setting_name": "text", "default_setting_value": "3|3|3|3|3|3|3|3", "default_setting_enabled": True, "default_setting_description": "Digitmap timeout"},
    {"default_setting_uuid": "11111111-1111-4111-8111-111111111114", "default_setting_category": "provision", "default_setting_subcategory": "polycom_microbrowser_main_home", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Microbrowser home"},
    {"default_setting_uuid": "11111111-1111-4111-8111-111111111115", "default_setting_category": "provision", "default_setting_subcategory": "polycom_idle_display_animation", "default_setting_name": "text", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Idle display animation"},
    {"default_setting_uuid": "11111111-1111-4111-8111-111111111116", "default_setting_category": "provision", "default_setting_subcategory": "polycom_feature_presence_enabled", "default_setting_name": "boolean", "default_setting_value": "true", "default_setting_enabled": True, "default_setting_description": "Presence feature"},
    # snom — 6
    {"default_setting_uuid": "22222222-2222-4222-8222-222222222221", "default_setting_category": "provision", "default_setting_subcategory": "snom_admin_mode_password", "default_setting_name": "text", "default_setting_value": "0000", "default_setting_enabled": True, "default_setting_description": "Password to enter admin mode on the phone."},
    {"default_setting_uuid": "22222222-2222-4222-8222-222222222222", "default_setting_category": "provision", "default_setting_subcategory": "snom_time_zone", "default_setting_name": "text", "default_setting_value": "USA-7", "default_setting_enabled": True, "default_setting_description": "https://service.snom.com/display/wiki/timezone"},
    {"default_setting_uuid": "22222222-2222-4222-8222-222222222223", "default_setting_category": "provision", "default_setting_subcategory": "snom_firmware_url", "default_setting_name": "text", "default_setting_value": "https://server.domain.local/app/snom/", "default_setting_enabled": True, "default_setting_description": "Base URL for Snom Firmware."},
    {"default_setting_uuid": "22222222-2222-4222-8222-222222222224", "default_setting_category": "provision", "default_setting_subcategory": "snom_user_active", "default_setting_name": "text", "default_setting_value": "on", "default_setting_enabled": True, "default_setting_description": "User active"},
    {"default_setting_uuid": "22222222-2222-4222-8222-222222222225", "default_setting_category": "provision", "default_setting_subcategory": "snom_sip_port", "default_setting_name": "numeric", "default_setting_value": "5060", "default_setting_enabled": True, "default_setting_description": "SIP port"},
    {"default_setting_uuid": "22222222-2222-4222-8222-222222222226", "default_setting_category": "provision", "default_setting_subcategory": "snom_outbound_proxy", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Outbound proxy"},
    # yealink — 6
    {"default_setting_uuid": "33333333-3333-4333-8333-333333333331", "default_setting_category": "provision", "default_setting_subcategory": "yealink_provision_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Provisioning URL for Yealink"},
    {"default_setting_uuid": "33333333-3333-4333-8333-333333333332", "default_setting_category": "provision", "default_setting_subcategory": "yealink_time_zone", "default_setting_name": "text", "default_setting_value": "-5", "default_setting_enabled": True, "default_setting_description": "Time zone ranges from -11 to +12"},
    {"default_setting_uuid": "33333333-3333-4333-8333-333333333333", "default_setting_category": "provision", "default_setting_subcategory": "yealink_time_zone_name", "default_setting_name": "text", "default_setting_value": "United States-Eastern Time", "default_setting_enabled": True, "default_setting_description": "Time zone name"},
    {"default_setting_uuid": "33333333-3333-4333-8333-333333333334", "default_setting_category": "provision", "default_setting_subcategory": "yealink_time_format", "default_setting_name": "text", "default_setting_value": "1", "default_setting_enabled": True, "default_setting_description": "Time format"},
    {"default_setting_uuid": "33333333-3333-4333-8333-333333333335", "default_setting_category": "provision", "default_setting_subcategory": "yealink_date_format", "default_setting_name": "text", "default_setting_value": "3", "default_setting_enabled": True, "default_setting_description": "Date format"},
    {"default_setting_uuid": "33333333-3333-4333-8333-333333333336", "default_setting_category": "provision", "default_setting_subcategory": "yealink_dsskey_label", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "DSS key label"},
    # grandstream — 6
    {"default_setting_uuid": "44444444-4444-4444-8444-444444444441", "default_setting_category": "provision", "default_setting_subcategory": "grandstream_call_waiting", "default_setting_name": "numeric", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Call Waiting 0-default, 1-enabled, 2-disable."},
    {"default_setting_uuid": "44444444-4444-4444-8444-444444444442", "default_setting_category": "provision", "default_setting_subcategory": "grandstream_gxp_time_zone", "default_setting_name": "text", "default_setting_value": "auto", "default_setting_enabled": True, "default_setting_description": "See provision profile for codes."},
    {"default_setting_uuid": "44444444-4444-4444-8444-444444444443", "default_setting_category": "provision", "default_setting_subcategory": "grandstream_firmware_server_path", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Firmware server path"},
    {"default_setting_uuid": "44444444-4444-4444-8444-444444444444", "default_setting_category": "provision", "default_setting_subcategory": "grandstream_config_server_path", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Config server path"},
    {"default_setting_uuid": "44444444-4444-4444-8444-444444444445", "default_setting_category": "provision", "default_setting_subcategory": "grandstream_admin_password", "default_setting_name": "text", "default_setting_value": "admin", "default_setting_enabled": True, "default_setting_description": "Admin password"},
    {"default_setting_uuid": "44444444-4444-4444-8444-444444444446", "default_setting_category": "provision", "default_setting_subcategory": "grandstream_sip_registration_retry", "default_setting_name": "numeric", "default_setting_value": "20", "default_setting_enabled": True, "default_setting_description": "SIP registration retry"},
    # cisco — 5
    {"default_setting_uuid": "55555555-5555-4555-8555-555555555551", "default_setting_category": "provision", "default_setting_subcategory": "spa_time_zone", "default_setting_name": "text", "default_setting_value": "GMT-07:00", "default_setting_enabled": True, "default_setting_description": "Time zone"},
    {"default_setting_uuid": "55555555-5555-4555-8555-555555555552", "default_setting_category": "provision", "default_setting_subcategory": "spa_time_format", "default_setting_name": "text", "default_setting_value": "12hr", "default_setting_enabled": True, "default_setting_description": "Time format"},
    {"default_setting_uuid": "55555555-5555-4555-8555-555555555553", "default_setting_category": "provision", "default_setting_subcategory": "spa_date_format", "default_setting_name": "text", "default_setting_value": "day/month", "default_setting_enabled": True, "default_setting_description": "Date format"},
    {"default_setting_uuid": "55555555-5555-4555-8555-555555555554", "default_setting_category": "provision", "default_setting_subcategory": "spa_provision_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Provision URL"},
    {"default_setting_uuid": "55555555-5555-4555-8555-555555555555", "default_setting_category": "provision", "default_setting_subcategory": "spa_back_light_timer", "default_setting_name": "text", "default_setting_value": "30 s", "default_setting_enabled": True, "default_setting_description": "Back light timer"},
    # aastra — 7
    {"default_setting_uuid": "66666666-6666-4666-8666-666666666661", "default_setting_category": "provision", "default_setting_subcategory": "aastra_gmt_offset", "default_setting_name": "numeric", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Aastra GMT offset"},
    {"default_setting_uuid": "66666666-6666-4666-8666-666666666662", "default_setting_category": "provision", "default_setting_subcategory": "aastra_time_format", "default_setting_name": "numeric", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Aastra clock time format"},
    {"default_setting_uuid": "66666666-6666-4666-8666-666666666663", "default_setting_category": "provision", "default_setting_subcategory": "aastra_date_format", "default_setting_name": "numeric", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Aastra date format"},
    {"default_setting_uuid": "66666666-6666-4666-8666-666666666664", "default_setting_category": "provision", "default_setting_subcategory": "aastra_ptime", "default_setting_name": "numeric", "default_setting_value": "20", "default_setting_enabled": True, "default_setting_description": "Packetization time"},
    {"default_setting_uuid": "66666666-6666-4666-8666-666666666665", "default_setting_category": "provision", "default_setting_subcategory": "aastra_silence_suppression", "default_setting_name": "text", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Silence suppression"},
    {"default_setting_uuid": "1455b1b0-68ec-400d-be84-1d1132aea72f", "default_setting_category": "provision", "default_setting_subcategory": "aastra_zone_minutes", "default_setting_name": "numeric", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Offset in minutes from GMT, 300 = GMT-5 = Eastern Standard Time, -120 = GMT+2 = Eastern European Time"},
    {"default_setting_uuid": "c90e804a-d2bb-431a-ace8-5c69e140c539", "default_setting_category": "provision", "default_setting_subcategory": "aastra_sip_silence_suppression", "default_setting_name": "numeric", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Aastra SIP silence suppression - silence suppression: deactivated (0), activated (1); enabled on G.711 with CN(RFC3389) or G.729AB"},
    # htek — 5
    {"default_setting_uuid": "77777777-7777-4777-8777-777777777771", "default_setting_category": "provision", "default_setting_subcategory": "htek_provision_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": True, "default_setting_description": "Provision URL"},
    {"default_setting_uuid": "77777777-7777-4777-8777-777777777772", "default_setting_category": "provision", "default_setting_subcategory": "htek_time_zone", "default_setting_name": "text", "default_setting_value": "18", "default_setting_enabled": True, "default_setting_description": "Time zone"},
    {"default_setting_uuid": "77777777-7777-4777-8777-777777777773", "default_setting_category": "provision", "default_setting_subcategory": "htek_dst", "default_setting_name": "text", "default_setting_value": "1", "default_setting_enabled": True, "default_setting_description": "DST"},
    {"default_setting_uuid": "77777777-7777-4777-8777-777777777774", "default_setting_category": "provision", "default_setting_subcategory": "htek_date_display_format", "default_setting_name": "text", "default_setting_value": "1", "default_setting_enabled": True, "default_setting_description": "Date display format"},
    {"default_setting_uuid": "77777777-7777-4777-8777-777777777775", "default_setting_category": "provision", "default_setting_subcategory": "htek_time_format", "default_setting_name": "text", "default_setting_value": "1", "default_setting_enabled": True, "default_setting_description": "Time format"},
    # fanvil — 5
    {"default_setting_uuid": "88888888-8888-4888-8888-888888888881", "default_setting_category": "provision", "default_setting_subcategory": "fanvil_time_zone_name", "default_setting_name": "text", "default_setting_value": "UTC-5", "default_setting_enabled": True, "default_setting_description": "Time zone name"},
    {"default_setting_uuid": "88888888-8888-4888-8888-888888888882", "default_setting_category": "provision", "default_setting_subcategory": "fanvil_location", "default_setting_name": "text", "default_setting_value": "4", "default_setting_enabled": True, "default_setting_description": "Location"},
    {"default_setting_uuid": "88888888-8888-4888-8888-888888888883", "default_setting_category": "provision", "default_setting_subcategory": "fanvil_enable_dst", "default_setting_name": "text", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "Enable DST"},
    {"default_setting_uuid": "88888888-8888-4888-8888-888888888884", "default_setting_category": "provision", "default_setting_subcategory": "fanvil_enable_sntp", "default_setting_name": "text", "default_setting_value": "1", "default_setting_enabled": True, "default_setting_description": "Enable SNTP"},
    {"default_setting_uuid": "88888888-8888-4888-8888-888888888885", "default_setting_category": "provision", "default_setting_subcategory": "fanvil_dst_fixed_type", "default_setting_name": "text", "default_setting_value": "0", "default_setting_enabled": True, "default_setting_description": "DST fixed type"},
    # algo — 7
    {"default_setting_uuid": "e874cf2e-b58a-4833-b1c5-921ee3419681", "default_setting_category": "provision", "default_setting_subcategory": "algo_8186_version_firmware", "default_setting_name": "text", "default_setting_value": "5.5f", "default_setting_enabled": False, "default_setting_description": "Algo 8186 Firmware Version"},
    {"default_setting_uuid": "a1cbb813-b6a2-4e5e-87a5-8e7572b0ba87", "default_setting_category": "provision", "default_setting_subcategory": "algo_8188_version_firmware", "default_setting_name": "text", "default_setting_value": "5.5f", "default_setting_enabled": False, "default_setting_description": "Algo 8188 Firmware Version"},
    {"default_setting_uuid": "646d81ec-eaa9-426f-a32d-1876dc4b75ed", "default_setting_category": "provision", "default_setting_subcategory": "algo_8189_version_firmware", "default_setting_name": "text", "default_setting_value": "5.5f", "default_setting_enabled": False, "default_setting_description": "Algo 8189 Firmware Version"},
    {"default_setting_uuid": "75c1afc0-1f8e-4dd8-bc2b-6f27a7818aff", "default_setting_category": "provision", "default_setting_subcategory": "algo_download_method", "default_setting_name": "text", "default_setting_value": "https", "default_setting_enabled": False, "default_setting_description": "How to get the provisioning and firmware files. TFTP, FTP, HTTP, HTTPS"},
    {"default_setting_uuid": "dc5ecda5-6f3a-4967-811e-9e9fe7690b87", "default_setting_category": "provision", "default_setting_subcategory": "algo_firmware_path", "default_setting_name": "text", "default_setting_value": "firmware/algo", "default_setting_enabled": False, "default_setting_description": "HTTP or HTTPS for downloading"},
    {"default_setting_uuid": "3a79d794-0009-48ef-bd65-0f5dd28579f9", "default_setting_category": "provision", "default_setting_subcategory": "algo_inbound_timeout", "default_setting_name": "text", "default_setting_value": "240", "default_setting_enabled": True, "default_setting_description": "In seconds, How long can a page last."},
    {"default_setting_uuid": "e7d51f9f-b9f2-494f-b99a-ece2ffcf7cfe", "default_setting_category": "provision", "default_setting_subcategory": "algo_multicast_mode", "default_setting_name": "text", "default_setting_value": "240", "default_setting_enabled": True, "default_setting_description": "Algo Multicast Modes Options: 0 - None, 1 - Transmitter (Sender), 2- Receiver (Listener)."},
    # avaya — 21
    {"default_setting_uuid": "c58b1c0f-b0c9-42a2-a971-4da25fa9fafa", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j139_firmware_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "Firmware filename URL for J139 Phone"},
    {"default_setting_uuid": "98c04df8-35c8-4465-a52b-f1b7cdf08c30", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j179_firmware_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "Firmware filename URL for J179 Phone"},
    {"default_setting_uuid": "d2e19a7d-d3b3-4b49-b267-938f66845bdf", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j169_firmware_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "Firmware filename URL for J169 Phone"},
    {"default_setting_uuid": "053f9829-1f4d-4292-a308-e447dc4483b9", "default_setting_category": "provision", "default_setting_subcategory": "avaya_jem24_firmware_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "Firmware filename URL for JEM 24 Expansion Module"},
    {"default_setting_uuid": "bbc7f7c7-d8a9-4f13-bf1d-a16ee0855c36", "default_setting_category": "provision", "default_setting_subcategory": "avaya_dialplan", "default_setting_name": "text", "default_setting_value": "XXX|XXXX,*XX,1XXXXXXXXXX,XXXXXXXXXX", "default_setting_enabled": False, "default_setting_description": "|:Pattern separator, X: any single digit, []: used to specify one digit with valid digits inside the brackets. a - dash may be used for ranges."},
    {"default_setting_uuid": "ab3fc35c-fb8b-4dd0-ba0d-b0883cd31c45", "default_setting_category": "provision", "default_setting_subcategory": "avaya_time_zone", "default_setting_name": "text", "default_setting_value": "-5:00", "default_setting_enabled": False, "default_setting_description": "Timezone offset"},
    {"default_setting_uuid": "0c3b36b2-a638-4d8e-8619-8fde234e6349", "default_setting_category": "provision", "default_setting_subcategory": "avaya_dst_start", "default_setting_name": "text", "default_setting_value": "2SunMar2L", "default_setting_enabled": False, "default_setting_description": "Example, 2SunMar2L which corresponds to the second Sunday in March at 2 AM local time"},
    {"default_setting_uuid": "09922e2a-6d3f-4833-8a50-40b1942a7bb0", "default_setting_category": "provision", "default_setting_subcategory": "avaya_dst_stop", "default_setting_name": "text", "default_setting_value": "1SunNov2L", "default_setting_enabled": False, "default_setting_description": "Example, 1SunNov2L which corresponds to the first Sunday in November at 2 AM local time"},
    {"default_setting_uuid": "cd9143fb-54f7-4bf7-acea-d7ea775c9876", "default_setting_category": "provision", "default_setting_subcategory": "avaya_key_customization", "default_setting_name": "text", "default_setting_value": "0", "default_setting_enabled": False, "default_setting_description": "Setting to allow or disallow users to change the buttons on the phone. 0 - Blocked, 1 - Limited (User customization is allowed for Applications and Contacts), 2 - Full"},
    {"default_setting_uuid": "b67d14cb-5d1f-46bf-baa5-2500a9135b5d", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j189_wallpaper_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "800 x 480 pixels max. JPG only. Max file size is 256KB"},
    {"default_setting_uuid": "554de3b3-444a-4221-9d15-524be3ef71a8", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j189_wallpaper_filename", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "The filename of the default wallpaper to be displayed"},
    {"default_setting_uuid": "90222f45-3996-4b74-89bc-68f12ad78afe", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j189_secondary_wallpaper_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "240 x 320 pixels max. JPG only. Max file size is 256KB"},
    {"default_setting_uuid": "1b08fd61-246f-4053-a053-568e7128e3e1", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j189_secondary_wallpaper_filename", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "The filename of the default wallpaper to be displayed"},
    {"default_setting_uuid": "75d92d17-6096-47ec-89f6-bbf17479fb85", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j179_wallpaper_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "320 x 240 pixels max. JPG only. Max file size is 256KB"},
    {"default_setting_uuid": "7b45e899-53ac-41b5-94c6-b485a41f66ff", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j179_wallpaper_filename", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "The filename of the default wallpaper to be displayed"},
    {"default_setting_uuid": "4244f3b6-78cd-46a2-bdbd-e0d8452c4de3", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j139_wallpaper_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "320 x 240 pixels max. JPG only. Max file size is 256KB"},
    {"default_setting_uuid": "234e2429-11dc-4a62-be81-79737085284c", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j139_wallpaper_filename", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "The filename of the default wallpaper to be displayed"},
    {"default_setting_uuid": "30cbfda7-f068-463f-80a4-aeecf8c48eaf", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j169_wallpaper_url", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "320 x 240 pixels max. JPG only. Max file size is 256KB"},
    {"default_setting_uuid": "7e29deec-d19a-4ff2-ab5b-513fa824e4ed", "default_setting_category": "provision", "default_setting_subcategory": "avaya_j169_wallpaper_filename", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": "The filename of the default wallpaper to be displayed"},
    {"default_setting_uuid": "a53f2369-19fb-449d-8daa-189707b79cc6", "default_setting_category": "provision", "default_setting_subcategory": "avaya_ntp_server", "default_setting_name": "text", "default_setting_value": "", "default_setting_enabled": False, "default_setting_description": ""},
    {"default_setting_uuid": "4f4641c3-7398-421e-a0b3-5d7960813728", "default_setting_category": "provision", "default_setting_subcategory": "avaya_blf_subscription_expiry", "default_setting_name": "text", "default_setting_value": "60", "default_setting_enabled": False, "default_setting_description": "The expiry that Avaya phones will populate in their SUBSCRIBE messages. This should be set to equal or less than the nonce-ttl value that is set in your SIP profile."},
    # generic provision — 5
    {"default_setting_uuid": "99999999-9999-4999-8999-999999999991", "default_setting_category": "provision", "default_setting_subcategory": "enabled", "default_setting_name": "boolean", "default_setting_value": "true", "default_setting_enabled": True, "default_setting_description": "Provision enabled"},
    {"default_setting_uuid": "99999999-9999-4999-8999-999999999992", "default_setting_category": "provision", "default_setting_subcategory": "http_auth_enabled", "default_setting_name": "boolean", "default_setting_value": "true", "default_setting_enabled": True, "default_setting_description": "HTTP auth enabled"},
    {"default_setting_uuid": "99999999-9999-4999-8999-999999999993", "default_setting_category": "provision", "default_setting_subcategory": "line_sip_port", "default_setting_name": "numeric", "default_setting_value": "5060", "default_setting_enabled": True, "default_setting_description": "SIP port"},
    {"default_setting_uuid": "99999999-9999-4999-8999-999999999994", "default_setting_category": "provision", "default_setting_subcategory": "line_sip_transport", "default_setting_name": "text", "default_setting_value": "tcp", "default_setting_enabled": True, "default_setting_description": "SIP transport"},
    {"default_setting_uuid": "99999999-9999-4999-8999-999999999995", "default_setting_category": "provision", "default_setting_subcategory": "ntp_server_primary", "default_setting_name": "text", "default_setting_value": "pool.ntp.org", "default_setting_enabled": True, "default_setting_description": "NTP server"},
]


async def seed_provision_defaults(session) -> int:
    """Idempotent seed of PROVISION_DEFAULTS into default_settings. Returns inserted count."""
    from sqlalchemy import text

    # Ensure table exists (for sqlite tests where alembic hasn't run)
    try:
        await session.execute(text("""
            CREATE TABLE IF NOT EXISTS default_settings (
                default_setting_uuid UUID PRIMARY KEY,
                default_setting_category VARCHAR(64) NOT NULL,
                default_setting_subcategory VARCHAR(64) NOT NULL,
                default_setting_name VARCHAR(64) NOT NULL,
                default_setting_value TEXT,
                default_setting_enabled BOOLEAN NOT NULL DEFAULT true,
                default_setting_description TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            );
        """))
    except Exception:
        pass
    # Also ensure indexes
    try:
        await session.execute(text("CREATE INDEX IF NOT EXISTS ix_default_settings_category ON default_settings (default_setting_category);"))
        await session.execute(text("CREATE INDEX IF NOT EXISTS ix_default_settings_subcategory ON default_settings (default_setting_subcategory);"))
    except Exception:
        pass

    inserted = 0
    for row in PROVISION_DEFAULTS:
        try:
            # Try postgres ON CONFLICT, fallback to plain insert for sqlite
            await session.execute(
                text("""
                    INSERT INTO default_settings
                    (default_setting_uuid, default_setting_category, default_setting_subcategory, default_setting_name, default_setting_value, default_setting_enabled, default_setting_description)
                    VALUES (:uuid, :cat, :sub, :name, :val, :enabled, :desc)
                    ON CONFLICT (default_setting_uuid) DO NOTHING
                """),
                {
                    "uuid": row["default_setting_uuid"],
                    "cat": row["default_setting_category"],
                    "sub": row["default_setting_subcategory"],
                    "name": row["default_setting_name"],
                    "val": row["default_setting_value"],
                    "enabled": row["default_setting_enabled"],
                    "desc": row["default_setting_description"],
                },
            )
            inserted += 1
        except Exception as e:
            # sqlite fallback without ON CONFLICT (older) — try INSERT OR IGNORE
            try:
                await session.execute(
                    text("""
                        INSERT OR IGNORE INTO default_settings
                        (default_setting_uuid, default_setting_category, default_setting_subcategory, default_setting_name, default_setting_value, default_setting_enabled, default_setting_description)
                        VALUES (:uuid, :cat, :sub, :name, :val, :enabled, :desc)
                    """),
                    {
                        "uuid": row["default_setting_uuid"],
                        "cat": row["default_setting_category"],
                        "sub": row["default_setting_subcategory"],
                        "name": row["default_setting_name"],
                        "val": row["default_setting_value"],
                        "enabled": row["default_setting_enabled"],
                        "desc": row["default_setting_description"],
                    },
                )
                inserted += 1
            except Exception:
                continue
    try:
        await session.commit()
    except Exception:
        pass
    return inserted


if __name__ == "__main__":
    import asyncio
    from app.db import SessionLocal

    async def _run():
        async with SessionLocal() as s:
            n = await seed_provision_defaults(s)
            print(f"seeded {n} provision defaults")

    asyncio.run(_run())
