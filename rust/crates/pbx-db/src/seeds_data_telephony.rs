// GENERATED from app/seeds/dialplans.py + app/seeds/sip_profiles.py — do not hand-edit.
// Regenerate with: python3 rust/crates/pbx-db/gen_telephony_seeds.py
//
// Included by `seeds.rs` via `include!` — seed structs are in scope there.

/// 4 dialplans from app/seeds/dialplans.py (each with 2 details).
pub const DIALPLAN_SEED: &[DialplanSeed] = &[
    DialplanSeed {
        name: "local_extension",
        order: 890,
        description: Some("Bridge extension-to-extension"),
        details: &[
            DialplanDetailSeed {
                tag: "condition",
                r#type: "destination_number",
                data: "^(\\\\d{3,4})$",
                order: 0,
            },
            DialplanDetailSeed {
                tag: "action",
                r#type: "bridge",
                data: "user/$1@${domain_name}",
                order: 1,
            },
        ],
    },
    DialplanSeed {
        name: "voicemail_98",
        order: 320,
        description: Some("Voicemail *98"),
        details: &[
            DialplanDetailSeed {
                tag: "condition",
                r#type: "destination_number",
                data: "^\\\\*98$",
                order: 0,
            },
            DialplanDetailSeed {
                tag: "action",
                r#type: "transfer",
                data: "*98 XML ${domain_name}",
                order: 1,
            },
        ],
    },
    DialplanSeed {
        name: "conference_3500",
        order: 345,
        description: Some("Conference 3500"),
        details: &[
            DialplanDetailSeed {
                tag: "condition",
                r#type: "destination_number",
                data: "^3500$",
                order: 0,
            },
            DialplanDetailSeed {
                tag: "action",
                r#type: "conference",
                data: "3500@${domain_name}",
                order: 1,
            },
        ],
    },
    DialplanSeed {
        name: "toll_allow",
        order: 100,
        description: Some("Allow local dialing (seed; tighten per tenant)"),
        details: &[
            DialplanDetailSeed {
                tag: "condition",
                r#type: "destination_number",
                data: "^(\\\\d{7,15})$",
                order: 0,
            },
            DialplanDetailSeed {
                tag: "action",
                r#type: "bridge",
                data: "user/$1@${domain_name}",
                order: 1,
            },
        ],
    },
];

/// 4 profiles + 4 domains + 335 settings from app/seeds/sip_profiles.py.
pub const SIP_PROFILE_SEED: &[SipProfileSeed] = &[
    SipProfileSeed {
        name: "external-ipv6",
        enabled: true,
        port: 5080,
        description: Some("The External IPV6 profile binds to the IP version 6 address and is similar to the External profile."),
        domains: &[
            SipDomainSeed {
                name: "all",
                alias: "false",
                parse: "true",
            },
        ],
        settings: &[
            SipSettingSeed {
                name: "debug",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "user-agent-string",
                value: Some("FreeSWITCH"),
                enabled: true,
            },
            SipSettingSeed {
                name: "shutdown-on-fail",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-trace",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-capture",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rfc2833-pt",
                value: Some("101"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-rfc-5626",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-port",
                value: Some("5080"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dialplan",
                value: Some("XML"),
                enabled: true,
            },
            SipSettingSeed {
                name: "context",
                value: Some("public"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-duration",
                value: Some("2000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-prefs",
                value: Some("$${global_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "outbound-codec-prefs",
                value: Some("$${outbound_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "hold-music",
                value: Some("$${hold_music}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timer-name",
                value: Some("soft"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-100rel",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-srv503",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "local-network-acl",
                value: Some("localnet.auto"),
                enabled: true,
            },
            SipSettingSeed {
                name: "manage-presence",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dbname",
                value: Some("share_presence"),
                enabled: true,
            },
            SipSettingSeed {
                name: "presence-hosts",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-db-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-negotiation",
                value: Some("generous"),
                enabled: true,
            },
            SipSettingSeed {
                name: "nonce-ttl",
                value: Some("60"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-calls",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-all-packets",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-inbound-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-register-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-type",
                value: Some("rfc2833"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-subscriptions",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-late-negotiation",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-ip",
                value: Some("$${local_ip_v6}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-ip",
                value: Some("$${local_ip_v6}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-rtp-ip",
                value: Some("$${external_rtp_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-sip-ip",
                value: Some("$${external_sip_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timeout-sec",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-hold-timeout-sec",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_timeout",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_hold_timeout",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-3pcc",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls",
                value: Some("$${external_ssl_enable}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-only",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-bind-params",
                value: Some("transport=tls"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-sip-port",
                value: Some("5081"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-cert-dir",
                value: Some("$${external_ssl_dir}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-passphrase",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-date",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-policy",
                value: Some("none"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-depth",
                value: Some("2"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-in-subjects",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-version",
                value: Some("$${sip_tls_version}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "zrtp-passthru",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-all-packets",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "session-timeout",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-timer",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "minimum-session-expires",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "odbc-dsn",
                value: Some("$${dsn}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "track-calls",
                value: Some("false"),
                enabled: true,
            },
        ],
    },
    SipProfileSeed {
        name: "external",
        enabled: true,
        port: 5080,
        description: Some("The External profile external provides anonymous calling in the public context. By default the External profile binds to port 5080."),
        domains: &[
            SipDomainSeed {
                name: "all",
                alias: "false",
                parse: "false",
            },
        ],
        settings: &[
            SipSettingSeed {
                name: "debug",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "user-agent-string",
                value: Some("FreeSWITCH"),
                enabled: true,
            },
            SipSettingSeed {
                name: "shutdown-on-fail",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-trace",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-capture",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rfc2833-pt",
                value: Some("101"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-rfc-5626",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-port",
                value: Some("5080"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dialplan",
                value: Some("XML"),
                enabled: true,
            },
            SipSettingSeed {
                name: "context",
                value: Some("public"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-duration",
                value: Some("2000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-prefs",
                value: Some("$${global_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "outbound-codec-prefs",
                value: Some("$${outbound_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "hold-music",
                value: Some("$${hold_music}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timer-name",
                value: Some("soft"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-100rel",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-srv503",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "manage-presence",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "local-network-acl",
                value: Some("localnet.auto"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-nat-acl",
                value: Some("nat.auto"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-inbound-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-register-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dbname",
                value: Some("share_presence"),
                enabled: true,
            },
            SipSettingSeed {
                name: "presence-hosts",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-db-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "aggressive-nat-detection",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-negotiation",
                value: Some("generous"),
                enabled: true,
            },
            SipSettingSeed {
                name: "nonce-ttl",
                value: Some("60"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-calls",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-subscriptions",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-late-negotiation",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-ip",
                value: Some("$${local_ip_v4}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-ip",
                value: Some("$${local_ip_v4}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-rtp-ip",
                value: Some("$${external_rtp_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-sip-ip",
                value: Some("$${external_sip_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timeout-sec",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-hold-timeout-sec",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_timeout",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_hold_timeout",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-3pcc",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls",
                value: Some("$${external_ssl_enable}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-only",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-bind-params",
                value: Some("transport=tls"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-sip-port",
                value: Some("5081"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-cert-dir",
                value: Some("$${external_ssl_dir}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-passphrase",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-date",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-policy",
                value: Some("none"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-depth",
                value: Some("2"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-in-subjects",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-version",
                value: Some("$${sip_tls_version}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-ciphers",
                value: Some("$${sip_tls_ciphers}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "suppress-cng",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "session-timeout",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-timer",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "minimum-session-expires",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "odbc-dsn",
                value: Some("$${dsn}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-type",
                value: Some("rfc2833"),
                enabled: true,
            },
            SipSettingSeed {
                name: "zrtp-passthru",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "track-calls",
                value: Some("false"),
                enabled: true,
            },
        ],
    },
    SipProfileSeed {
        name: "internal-ipv6",
        enabled: true,
        port: 5060,
        description: Some("The Internal IPV6 profile binds to the IP version 6 address and is similar to the Internal profile."),
        domains: &[
            SipDomainSeed {
                name: "all",
                alias: "false",
                parse: "false",
            },
        ],
        settings: &[
            SipSettingSeed {
                name: "user-agent-string",
                value: Some("FreeSWITCH"),
                enabled: true,
            },
            SipSettingSeed {
                name: "debug",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-trace",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "context",
                value: Some("public"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rfc2833-pt",
                value: Some("101"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-rfc-5626",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-port",
                value: Some("5060"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dialplan",
                value: Some("XML"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-duration",
                value: Some("2000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-prefs",
                value: Some("$${global_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "outbound-codec-prefs",
                value: Some("$${global_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "use-rtp-timer",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timer-name",
                value: Some("soft"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-ip",
                value: Some("$${local_ip_v6}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-ip",
                value: Some("$${local_ip_v6}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "hold-music",
                value: Some("$${hold_music}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-100rel",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-srv503",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-inbound-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-register-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-type",
                value: Some("rfc2833"),
                enabled: true,
            },
            SipSettingSeed {
                name: "record-template",
                value: Some("$${recordings_dir}/${caller_id_number}.${strftime(%Y-%m-%d-%H-%M-%S)}.${record_ext}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "manage-presence",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "bitpacking",
                value: Some("aal2"),
                enabled: true,
            },
            SipSettingSeed {
                name: "max-proceeding",
                value: Some("1000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "max-recv-requests-per-second",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "session-timeout",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-timer",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "minimum-session-expires",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "multiple-registrations",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-negotiation",
                value: Some("generous"),
                enabled: true,
            },
            SipSettingSeed {
                name: "bind-params",
                value: Some("transport=udp"),
                enabled: true,
            },
            SipSettingSeed {
                name: "unregister-on-options-fail",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls",
                value: Some("$${internal_ssl_enable}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-only",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-bind-params",
                value: Some("transport=tls"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-sip-port",
                value: Some("5061"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-cert-dir",
                value: Some("$${internal_ssl_dir}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-passphrase",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-date",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-policy",
                value: Some("none"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-depth",
                value: Some("2"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-in-subjects",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-version",
                value: Some("$${sip_tls_version}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-ciphers",
                value: Some("$${sip_tls_ciphers}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-orq-connect-timeout",
                value: Some("3000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-rewrite-timestamps",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "pass-rfc2833",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "odbc-dsn",
                value: Some("$${dsn}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-bypass-media",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-proxy-media",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-late-negotiation",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "accept-blind-reg",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "accept-blind-auth",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "suppress-cng",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "nonce-ttl",
                value: Some("60"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-transcoding",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "NDLB-broken-auth-hash",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "NDLB-received-in-nat-reg-contact",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-calls",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-subscriptions",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-all-packets",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-rtp-ip",
                value: Some("$${external_rtp_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-sip-ip",
                value: Some("$${external_sip_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timeout-sec",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-hold-timeout-sec",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_timeout",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_hold_timeout",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "vad",
                value: Some("out"),
                enabled: true,
            },
            SipSettingSeed {
                name: "alias",
                value: Some("sip:10.0.1.251:5555"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-db-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-transfer",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-register",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-3pcc",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "stun-enabled",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "stun-auto-disable",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-srv",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-naptr",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "track-calls",
                value: Some("false"),
                enabled: true,
            },
        ],
    },
    SipProfileSeed {
        name: "internal",
        enabled: true,
        port: 5060,
        description: Some("The Internal profile by default requires registration which is used by the endpoints. By default the Internal profile binds to port 5060."),
        domains: &[
            SipDomainSeed {
                name: "all",
                alias: "false",
                parse: "false",
            },
        ],
        settings: &[
            SipSettingSeed {
                name: "rtp-digit-delay",
                value: Some("40"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media-option",
                value: Some("resume-media-on-hold"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media-option",
                value: Some("bypass-media-after-att-xfer"),
                enabled: true,
            },
            SipSettingSeed {
                name: "user-agent-string",
                value: Some("FreeSWITCH"),
                enabled: true,
            },
            SipSettingSeed {
                name: "debug",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "shutdown-on-fail",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-trace",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-capture",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "presence-proto-lookup",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "liberal-dtmf",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "watchdog-enabled",
                value: Some("no"),
                enabled: true,
            },
            SipSettingSeed {
                name: "watchdog-step-timeout",
                value: Some("30000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "watchdog-event-timeout",
                value: Some("30000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "log-auth-failures",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "forward-unsolicited-mwi-notify",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "context",
                value: Some("public"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rfc2833-pt",
                value: Some("101"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-rfc-5626",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-port",
                value: Some("5060"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dialplan",
                value: Some("XML"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-duration",
                value: Some("2000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-prefs",
                value: Some("$${global_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "outbound-codec-prefs",
                value: Some("$${global_codec_prefs}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timer-name",
                value: Some("soft"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-ip",
                value: Some("$${local_ip_v4}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-ip",
                value: Some("$${local_ip_v4}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "hold-music",
                value: Some("$${hold_music}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-nat-acl",
                value: Some("nat.auto"),
                enabled: true,
            },
            SipSettingSeed {
                name: "cid-in-1xx",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "extended-info-parsing",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "aggressive-nat-detection",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-100rel",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-srv503",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-compact-headers",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-timer",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "minimum-session-expires",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-inbound-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "local-network-acl",
                value: Some("localnet.auto"),
                enabled: true,
            },
            SipSettingSeed {
                name: "apply-register-acl",
                value: Some("providers"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dtmf-type",
                value: Some("rfc2833"),
                enabled: true,
            },
            SipSettingSeed {
                name: "send-message-query-on-register",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "send-presence-on-register",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "caller-id-type",
                value: Some("rpid"),
                enabled: true,
            },
            SipSettingSeed {
                name: "caller-id-type",
                value: Some("pid"),
                enabled: true,
            },
            SipSettingSeed {
                name: "caller-id-type",
                value: Some("none"),
                enabled: true,
            },
            SipSettingSeed {
                name: "record-path",
                value: Some("$${recordings_dir}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "record-template",
                value: Some("${domain_name}/archive/${strftime(%Y)}/${strftime(%b)}/${strftime(%d)}/${uuid}.${record_ext}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "manage-presence",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "presence-probe-on-register",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "manage-shared-appearance",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "dbname",
                value: Some("share_presence"),
                enabled: true,
            },
            SipSettingSeed {
                name: "presence-hosts",
                value: Some("$${domain},$${local_ip_v4}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "presence-privacy",
                value: Some("$${presence_privacy}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "bitpacking",
                value: Some("aal2"),
                enabled: true,
            },
            SipSettingSeed {
                name: "max-proceeding",
                value: Some("1000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "max-recv-requests-per-second",
                value: Some("0"),
                enabled: true,
            },
            SipSettingSeed {
                name: "session-timeout",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "multiple-registrations",
                value: Some("contact"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-codec-negotiation",
                value: Some("generous"),
                enabled: true,
            },
            SipSettingSeed {
                name: "bind-params",
                value: Some("transport=udp"),
                enabled: true,
            },
            SipSettingSeed {
                name: "unregister-on-options-fail",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls",
                value: Some("$${internal_ssl_enable}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-only",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-bind-params",
                value: Some("transport=tls"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-sip-port",
                value: Some("5061"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-cert-dir",
                value: Some("$${internal_ssl_dir}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-passphrase",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-date",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-policy",
                value: Some("none"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-depth",
                value: Some("2"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-verify-in-subjects",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-version",
                value: Some("$${sip_tls_version}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-ciphers",
                value: Some("$${sip_tls_ciphers}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "tls-orq-connect-timeout",
                value: Some("3000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-autoflush-during-bridge",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-rewrite-timestamps",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "pass-rfc2833",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "odbc-dsn",
                value: Some("$${dsn}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-bypass-media",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-proxy-media",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-late-negotiation",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "accept-blind-reg",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "accept-blind-auth",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "suppress-cng",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "nonce-ttl",
                value: Some("60"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-transcoding",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "manual-redirect",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "NDLB-broken-auth-hash",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "NDLB-received-in-nat-reg-contact",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-calls",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-subscriptions",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-reg-force-matching-username",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auth-all-packets",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-rtp-ip",
                value: Some("$${external_rtp_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ext-sip-ip",
                value: Some("$${external_sip_ip}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-timeout-sec",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-hold-timeout-sec",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_timeout",
                value: Some("300"),
                enabled: true,
            },
            SipSettingSeed {
                name: "media_hold_timeout",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "vad",
                value: Some("out"),
                enabled: true,
            },
            SipSettingSeed {
                name: "alias",
                value: Some("sip:10.0.1.251:5555"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-subscription-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-register-db-domain",
                value: Some("$${domain}"),
                enabled: true,
            },
            SipSettingSeed {
                name: "ws-binding",
                value: Some(":5066"),
                enabled: true,
            },
            SipSettingSeed {
                name: "wss-binding",
                value: Some(":7443"),
                enabled: true,
            },
            SipSettingSeed {
                name: "delete-subs-on-register",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-reg-in-new-thread",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtcp-audio-interval-msec",
                value: Some("5000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtcp-video-interval-msec",
                value: Some("5000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "force-subscription-expires",
                value: Some("60"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-subscription-max-deviation",
                value: Some("120"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-transfer",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-register",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "enable-3pcc",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "NDLB-force-rport",
                value: Some("safe"),
                enabled: true,
            },
            SipSettingSeed {
                name: "challenge-realm",
                value: Some("auto_to"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-rtp-auto-adjust",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "inbound-use-callid-as-uuid",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "outbound-use-uuid-as-callid",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "rtp-autofix-timing",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "pass-callee-id",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auto-rtp-bugs",
                value: Some(""),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-srv",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "disable-naptr",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "timer-T1",
                value: Some("500"),
                enabled: true,
            },
            SipSettingSeed {
                name: "timer-T1X64",
                value: Some("32000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "timer-T2",
                value: Some("4000"),
                enabled: true,
            },
            SipSettingSeed {
                name: "auto-jitterbuffer-msec",
                value: Some("60"),
                enabled: true,
            },
            SipSettingSeed {
                name: "renegotiate-codec-on-hold",
                value: Some("true"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-expires-max-deviation",
                value: Some("600"),
                enabled: true,
            },
            SipSettingSeed {
                name: "sip-force-expires",
                value: Some("1800"),
                enabled: true,
            },
            SipSettingSeed {
                name: "track-calls",
                value: Some("false"),
                enabled: true,
            },
            SipSettingSeed {
                name: "registration-thread-frequency",
                value: Some("30"),
                enabled: true,
            },
            SipSettingSeed {
                name: "nat-options-ping",
                value: Some("true"),
                enabled: true,
            },
        ],
    },
];
