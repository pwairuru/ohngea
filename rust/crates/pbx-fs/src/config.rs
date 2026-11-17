//! Configuration builders: gateway / vars / callcenter / IVR /
//! conference / local_stream XML.
//!
//! Ground truth is PHP `fusionpbx/resources/switch.php` (file writers:
//! `save_gateway_xml`, `save_var_xml`, `save_call_center_xml`) plus the Lua
//! `xml_handler` scripts (`ivr.conf.lua`, `conference.conf.lua`,
//! `local_stream.conf.lua`, `sofia.conf.lua`) for the served-document shape.
//! Pure functions (no DB): callers map SeaORM rows onto the input structs.

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn non_empty(v: &Option<String>) -> Option<&str> {
    v.as_deref().filter(|s| !s.is_empty())
}

fn bool_text(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

// ---- gateway (switch.php::save_gateway_xml, v_<uuid>.xml) ----

/// One `v_gateways` row's fields used by the XML builder.
#[derive(Clone, Debug, Default)]
pub struct GatewayInput {
    pub gateway_uuid: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub proxy: Option<String>,
    pub register: bool,
    /// Profile name (`external` default; PHP falls back when empty).
    pub profile: String,
    pub context: Option<String>,
    pub enabled: bool,
    pub distinct_to: Option<bool>,
    pub auth_username: Option<String>,
    pub realm: Option<String>,
    pub from_user: Option<String>,
    pub from_domain: Option<String>,
    pub register_proxy: Option<String>,
    pub outbound_proxy: Option<String>,
    pub expire_seconds: Option<i32>,
    pub register_transport: Option<String>,
    pub contact_params: Option<String>,
    pub retry_seconds: Option<i32>,
    pub extension: Option<String>,
    pub ping: Option<String>,
    pub ping_min: Option<i32>,
    pub ping_max: Option<i32>,
    pub contact_in_ping: Option<bool>,
    pub caller_id_in_from: Option<bool>,
    pub supress_cng: Option<bool>,
    pub sip_cid_type: Option<String>,
    pub extension_in_contact: Option<String>,
}

fn gateway_param(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!("      <param name=\"{name}\" value=\"{}\"/>\n", esc(value)));
}

/// Bare `<gateway>` element (shared by the file writer and the sofia
/// embedder). PHP `!empty` semantics: `None`/empty strings and zero
/// `expire/retry_seconds` are omitted; `Option<bool>` renders
/// `true`/`false` when set (`!empty("false")` is true in PHP).
/// `ping-min`/`ping-max`/`contact-in-ping` are a documented superset of the
/// PHP file list (Lua `sofia.conf.lua` serves them inline).
pub fn gateway_element(g: &GatewayInput) -> String {
    let mut out = format!("    <gateway name=\"{}\">\n", esc(&g.gateway_uuid.to_lowercase()));
    if let Some(v) = non_empty(&g.username) {
        gateway_param(&mut out, "username", v);
    }
    if let Some(b) = g.distinct_to {
        gateway_param(&mut out, "distinct-to", bool_text(b));
    }
    if let Some(v) = non_empty(&g.auth_username) {
        gateway_param(&mut out, "auth-username", v);
    }
    if let Some(v) = non_empty(&g.password) {
        gateway_param(&mut out, "password", v);
    }
    if let Some(v) = non_empty(&g.realm) {
        gateway_param(&mut out, "realm", v);
    }
    if let Some(v) = non_empty(&g.from_user) {
        gateway_param(&mut out, "from-user", v);
    }
    if let Some(v) = non_empty(&g.from_domain) {
        gateway_param(&mut out, "from-domain", v);
    }
    if let Some(v) = non_empty(&g.proxy) {
        gateway_param(&mut out, "proxy", v);
    }
    if let Some(v) = non_empty(&g.register_proxy) {
        gateway_param(&mut out, "register-proxy", v);
    }
    if let Some(v) = non_empty(&g.outbound_proxy) {
        gateway_param(&mut out, "outbound-proxy", v);
    }
    if let Some(n) = g.expire_seconds.filter(|n| *n != 0) {
        gateway_param(&mut out, "expire-seconds", &n.to_string());
    }
    gateway_param(&mut out, "register", bool_text(g.register));
    if let Some(t) = non_empty(&g.register_transport) {
        // PHP switch: udp/tcp/tls pass through, default writes the raw value.
        gateway_param(&mut out, "register-transport", t);
    }
    if let Some(v) = non_empty(&g.contact_params) {
        gateway_param(&mut out, "contact-params", v);
    }
    if let Some(n) = g.retry_seconds.filter(|n| *n != 0) {
        gateway_param(&mut out, "retry-seconds", &n.to_string());
    }
    if let Some(v) = non_empty(&g.extension) {
        gateway_param(&mut out, "extension", v);
    }
    if let Some(v) = non_empty(&g.ping) {
        gateway_param(&mut out, "ping", v);
    }
    if let Some(n) = g.ping_min {
        gateway_param(&mut out, "ping-min", &n.to_string());
    }
    if let Some(n) = g.ping_max {
        gateway_param(&mut out, "ping-max", &n.to_string());
    }
    if let Some(b) = g.contact_in_ping {
        gateway_param(&mut out, "contact-in-ping", bool_text(b));
    }
    if let Some(v) = non_empty(&g.context) {
        gateway_param(&mut out, "context", v);
    }
    if let Some(b) = g.caller_id_in_from {
        gateway_param(&mut out, "caller-id-in-from", bool_text(b));
    }
    if let Some(b) = g.supress_cng {
        gateway_param(&mut out, "supress-cng", bool_text(b));
    }
    if let Some(v) = non_empty(&g.sip_cid_type) {
        gateway_param(&mut out, "sip_cid_type", v);
    }
    if let Some(v) = non_empty(&g.extension_in_contact) {
        gateway_param(&mut out, "extension-in-contact", v);
    }
    out.push_str("    </gateway>\n");
    out
}

/// Exact `v_<uuid>.xml` file content: `<include>` wrapper around the gateway
/// element (PHP `save_gateway_xml`).
#[must_use]
pub fn build_gateway_xml(g: &GatewayInput) -> String {
    format!("<include>\n{}</include>", gateway_element(g))
}

// ---- vars (switch.php::save_var_xml) ----

/// One `v_vars` row's fields used by the XML builder.
#[derive(Clone, Debug, Default)]
pub struct VarInput {
    pub category: String,
    pub name: String,
    pub value: String,
    pub command: Option<String>,
    pub hostname: Option<String>,
    pub enabled: bool,
    pub order: Option<i32>,
}

/// `vars.xml` body: one `<X-PRE-PROCESS>` line per enabled, hostname-scoped,
/// non-`Provision` var ordered by `(category, order)` with `<!-- category -->`
/// headers. Empty command defaults to `set`; `Exec-Set` maps to `exec-set`.
#[must_use]
pub fn build_vars_xml(vars: &[VarInput], hostname: &str) -> String {
    let mut rows: Vec<&VarInput> = vars
        .iter()
        .filter(|v| {
            v.enabled
                && v.category != "Provision"
                && v.hostname.as_deref().is_none_or(|h| h.is_empty() || h == hostname)
        })
        .collect();
    rows.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.order.unwrap_or(i32::MAX).cmp(&b.order.unwrap_or(i32::MAX)))
            .then(a.name.cmp(&b.name))
    });
    let mut out = String::new();
    let mut prev = String::new();
    for v in rows {
        if v.category != prev {
            out.push_str(&format!("\n<!-- {} -->\n", v.category));
        }
        let cmd = if v.category == "Exec-Set" {
            "exec-set".to_owned()
        } else {
            let c = v.command.clone().unwrap_or_default();
            if c.is_empty() {
                "set".to_owned()
            } else {
                c
            }
        };
        out.push_str(&format!(
            "<X-PRE-PROCESS cmd=\"{}\" data=\"{}={}\" />\n",
            esc(&cmd),
            esc(&v.name),
            esc(&v.value)
        ));
        prev = v.category.clone();
    }
    out.push('\n');
    out
}

// ---- callcenter (switch.php::save_call_center_xml) ----

/// One `v_call_center_queues` row's fields used by the XML builder.
#[derive(Clone, Debug, Default)]
pub struct CallcenterQueueInput {
    pub name: String,
    pub extension: Option<String>,
    pub strategy: String,
    pub moh_sound: Option<String>,
    pub record_template: Option<String>,
    pub time_base_score: Option<String>,
    pub max_wait_time: Option<i32>,
    pub max_wait_time_with_no_agent: Option<i32>,
    pub max_wait_time_with_no_agent_time_reached: Option<i32>,
    pub tier_rules_apply: Option<bool>,
    pub tier_rule_wait_second: Option<i32>,
    pub tier_rule_wait_multiply_level: Option<bool>,
    pub tier_rule_no_agent_no_wait: Option<bool>,
    pub discard_abandoned_after: Option<i32>,
    pub abandoned_resume_allowed: Option<bool>,
    pub announce_sound: Option<String>,
    pub announce_frequency: Option<i32>,
}

/// One `v_call_center_agents` row's fields used by the XML builder.
/// `call_timeout` is required (PHP concatenates it raw; the mapping layer
/// defaults missing values to 30s).
#[derive(Clone, Debug, Default)]
pub struct CallcenterAgentInput {
    pub uuid: String,
    pub name: String,
    pub agent_type: Option<String>,
    pub contact: String,
    pub status: String,
    pub call_timeout: i32,
    pub no_answer_delay_time: Option<i32>,
    pub max_no_answer: Option<i32>,
    pub wrap_up_time: Option<i32>,
    pub reject_delay_time: Option<i32>,
    pub busy_delay_time: Option<i32>,
}

/// One `v_call_center_tiers` row's fields used by the XML builder.
#[derive(Clone, Debug)]
pub struct CallcenterTierInput {
    pub agent_name: String,
    pub queue_name: String,
    pub tier_level: i32,
    pub tier_position: i32,
}

impl Default for CallcenterTierInput {
    fn default() -> Self {
        Self {
            agent_name: String::new(),
            queue_name: String::new(),
            tier_level: 1,
            tier_position: 1,
        }
    }
}

/// PHP `moh-sound` mapping: empty → `local_stream://default`;
/// `local_stream://` passthrough; `${...ring}` → `tone_stream://…;loops=-1`.
fn moh_sound(raw: Option<&str>) -> String {
    match raw.filter(|s| !s.is_empty()) {
        None => "local_stream://default".to_owned(),
        Some(m) if m.starts_with("local_stream://") => m.to_owned(),
        Some(m) if m.starts_with("${") && m.ends_with("ring}") => {
            format!("tone_stream://{m};loops=-1")
        }
        Some(m) => m.to_owned(),
    }
}

/// PHP contact decoration: plain contacts gain `{call_timeout=N}`;
/// `sofia/gateway` contacts additionally gain the `group_confirm`
/// prefix; existing `{...}` blocks are extended before the final `}`.
fn decorate_contact(contact: &str, call_timeout: i32) -> String {
    let confirm = format!(
        "group_confirm_file=custom/press_1_to_accept_this_call.wav,group_confirm_key=1,group_confirm_read_timeout=2000,leg_timeout={call_timeout}"
    );
    let is_gw = contact.to_lowercase().contains("sofia/gateway");
    if !contact.contains('}') {
        if is_gw {
            format!("{{{confirm},call_timeout={call_timeout}}}{contact}")
        } else {
            format!("{{call_timeout={call_timeout}}}{contact}")
        }
    } else if !is_gw {
        if contact.contains("call_timeout") {
            contact.to_owned()
        } else {
            let pos = contact.rfind('}').unwrap_or(contact.len());
            format!("{},call_timeout={}{}", &contact[..pos], call_timeout, &contact[pos..])
        }
    } else if contact.contains("call_timeout") {
        let pos = contact.rfind('}').unwrap_or(contact.len());
        format!("{},{}{}", &contact[..pos], confirm, &contact[pos..])
    } else {
        let pos = contact.rfind('}').unwrap_or(contact.len());
        format!(
            "{},{},call_timeout={}{}",
            &contact[..pos],
            confirm,
            call_timeout,
            &contact[pos..]
        )
    }
}

fn cc_param(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!("            <param name=\"{name}\" value=\"{}\"/>\n", esc(value)));
}

fn opt_str(v: &Option<String>) -> &str {
    v.as_deref().unwrap_or("")
}

fn opt_int(v: Option<i32>) -> String {
    v.map(|n| n.to_string()).unwrap_or_default()
}

fn opt_bool(v: Option<bool>) -> &'static str {
    match v {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    }
}

/// `callcenter.conf` document: queues (`queue@domain`, spaces → `_`),
/// agents (`agent@domain` with decorated contact), tiers
/// (`agent@domain`/`queue@domain`).
#[must_use]
pub fn build_callcenter_xml(
    queues: &[(CallcenterQueueInput, String)],
    agents: &[(CallcenterAgentInput, String)],
    tiers: &[(CallcenterTierInput, String)],
) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document type=\"freeswitch/xml\">\n    <section name=\"configuration\">\n            <configuration name=\"callcenter.conf\" description=\"Call Center\">\n                    <queues>\n",
    );
    for (q, domain) in queues {
        let name = q.name.replace(' ', "_");
        out.push_str(&format!("                        <queue name=\"{name}@{}\">\n", esc(domain)));
        cc_param(&mut out, "strategy", &q.strategy);
        cc_param(&mut out, "moh-sound", &moh_sound(q.moh_sound.as_deref()));
        if let Some(rt) = non_empty(&q.record_template) {
            cc_param(&mut out, "record-template", rt);
        }
        cc_param(&mut out, "time-base-score", opt_str(&q.time_base_score));
        cc_param(&mut out, "max-wait-time", &opt_int(q.max_wait_time));
        cc_param(&mut out, "max-wait-time-with-no-agent", &opt_int(q.max_wait_time_with_no_agent));
        cc_param(
            &mut out,
            "max-wait-time-with-no-agent-time-reached",
            &opt_int(q.max_wait_time_with_no_agent_time_reached),
        );
        cc_param(&mut out, "tier-rules-apply", opt_bool(q.tier_rules_apply));
        cc_param(&mut out, "tier-rule-wait-second", &opt_int(q.tier_rule_wait_second));
        cc_param(
            &mut out,
            "tier-rule-wait-multiply-level",
            opt_bool(q.tier_rule_wait_multiply_level),
        );
        cc_param(
            &mut out,
            "tier-rule-no-agent-no-wait",
            opt_bool(q.tier_rule_no_agent_no_wait),
        );
        cc_param(&mut out, "discard-abandoned-after", &opt_int(q.discard_abandoned_after));
        cc_param(&mut out, "abandoned-resume-allowed", opt_bool(q.abandoned_resume_allowed));
        cc_param(&mut out, "announce-sound", opt_str(&q.announce_sound));
        cc_param(&mut out, "announce-frequency", &opt_int(q.announce_frequency));
        out.push_str("                        </queue>\n");
    }
    out.push_str("                    </queues>\n                    <agents>\n");
    for (a, domain) in agents {
        out.push_str(&format!(
            "                        <agent name=\"{}@{}\" type=\"{}\" contact=\"{}\" status=\"{}\" no-answer-delay-time=\"{}\" max-no-answer=\"{}\" wrap-up-time=\"{}\" reject-delay-time=\"{}\" busy-delay-time=\"{}\"/>\n",
            esc(&a.name),
            esc(domain),
            esc(opt_str(&a.agent_type)),
            esc(&decorate_contact(&a.contact, a.call_timeout)),
            esc(&a.status),
            opt_int(a.no_answer_delay_time),
            opt_int(a.max_no_answer),
            opt_int(a.wrap_up_time),
            opt_int(a.reject_delay_time),
            opt_int(a.busy_delay_time),
        ));
    }
    out.push_str("                    </agents>\n                    <tiers>\n");
    for (t, domain) in tiers {
        out.push_str(&format!(
            "                        <tier agent=\"{}@{}\" queue=\"{}@{}\" level=\"{}\" position=\"{}\"/>\n",
            esc(&t.agent_name),
            esc(domain),
            esc(&t.queue_name.replace(' ', "_")),
            esc(domain),
            t.tier_level,
            t.tier_position,
        ));
    }
    out.push_str(
        "                    </tiers>\n            </configuration>\n    </section>\n</document>\n",
    );
    out
}

// ---- IVR (xml_handler/configuration/ivr.conf.lua) ----

/// One `v_ivr_menus` row's fields used by the XML builder.
#[derive(Clone, Debug, Default)]
pub struct IvrMenuInput {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub greet_long: Option<String>,
    pub greet_short: Option<String>,
    pub invalid_sound: Option<String>,
    pub exit_sound: Option<String>,
    pub pin_number: Option<String>,
    pub confirm_macro: Option<String>,
    pub confirm_key: Option<String>,
    pub tts_engine: Option<String>,
    pub tts_voice: Option<String>,
    pub confirm_attempts: Option<i32>,
    pub timeout: Option<i32>,
    pub inter_digit_timeout: Option<i32>,
    pub max_failures: Option<i32>,
    pub max_timeouts: Option<i32>,
    pub digit_len: Option<i32>,
    pub exit_app: Option<String>,
    pub exit_data: Option<String>,
    pub direct_dial: bool,
    /// Transfer context for the direct-dial entry (falls back to domain).
    pub context: Option<String>,
    pub domain: String,
}

/// One enabled `v_ivr_menu_options` row.
#[derive(Clone, Debug, Default)]
pub struct IvrOptionInput {
    pub action: String,
    pub digits: String,
    pub param: Option<String>,
    pub description: Option<String>,
}

/// `ivr.conf` document: one `<menu name={uuid}>` per menu plus its
/// `<entry>` rows. Direct dial appends the PHP-style
/// `menu-exec-app … transfer $1 XML {context}` entry.
#[must_use]
pub fn build_ivr_xml(menus: &[(&IvrMenuInput, &[IvrOptionInput])]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document type=\"freeswitch/xml\">\n    <section name=\"configuration\">\n        <configuration name=\"ivr.conf\" description=\"IVR Menus\">\n            <menus>\n",
    );
    for (m, opts) in menus {
        let attr = |v: &Option<String>| esc(opt_str(v));
        let attr_i = |v: Option<i32>| opt_int(v);
        out.push_str(&format!(
            "                <menu name=\"{}\" description=\"{}\" greet-long=\"{}\" greet-short=\"{}\" invalid-sound=\"{}\" exit-sound=\"{}\" pin=\"{}\" confirm-macro=\"{}\" confirm-key=\"{}\" tts-engine=\"{}\" tts-voice=\"{}\" confirm-attempts=\"{}\" timeout=\"{}\" inter-digit-timeout=\"{}\" max-failures=\"{}\" max-timeouts=\"{}\" digit-len=\"{}\" ivr_menu_exit_app=\"{}\" ivr_menu_exit_data=\"{}\" >\n",
            esc(&m.uuid),
            esc(&m.description.clone().unwrap_or_else(|| m.name.clone())),
            attr(&m.greet_long),
            attr(&m.greet_short),
            attr(&m.invalid_sound),
            attr(&m.exit_sound),
            attr(&m.pin_number),
            attr(&m.confirm_macro),
            attr(&m.confirm_key),
            attr(&m.tts_engine),
            attr(&m.tts_voice),
            attr_i(m.confirm_attempts),
            attr_i(m.timeout),
            attr_i(m.inter_digit_timeout),
            attr_i(m.max_failures),
            attr_i(m.max_timeouts),
            attr_i(m.digit_len),
            attr(&m.exit_app),
            attr(&m.exit_data),
        ));
        for o in *opts {
            out.push_str(&format!(
                "                    <entry action=\"{}\" digits=\"{}\" param=\"{}\" description=\"{}\"/>\n",
                esc(&o.action),
                esc(&o.digits),
                esc(opt_str(&o.param)),
                esc(opt_str(&o.description)),
            ));
        }
        if m.direct_dial {
            let ctx = non_empty(&m.context).unwrap_or(&m.domain);
            out.push_str(&format!(
                "                    <entry action=\"menu-exec-app\" digits=\"/(^\\d{{3,6}}$)/\" param=\"transfer $1 XML {}\"/>\n",
                esc(ctx),
            ));
        }
        out.push_str("                </menu>\n");
    }
    out.push_str("            </menus>\n        </configuration>\n    </section>\n</document>\n");
    out
}

// ---- conference (xml_handler/configuration/conference.conf.lua) ----

/// One enabled `v_conference_profiles` row plus its enabled params.
#[derive(Clone, Debug, Default)]
pub struct ConferenceProfileInput {
    pub name: String,
    pub params: Vec<(String, String)>,
}

/// One enabled `v_conference_controls` row plus its enabled details
/// `(digits, action, data)`.
#[derive(Clone, Debug, Default)]
pub struct ConferenceControlInput {
    pub name: String,
    pub details: Vec<(String, String, String)>,
}

/// `conference.conf` document: caller-controls groups + profiles.
/// Callers pass only enabled rows (Lua filters `*_enabled = true`).
#[must_use]
pub fn build_conference_xml(
    profiles: &[ConferenceProfileInput],
    controls: &[ConferenceControlInput],
) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document type=\"freeswitch/xml\">\n    <section name=\"configuration\">\n        <configuration name=\"conference.conf\" description=\"Audio Conference\">\n            <caller-controls>\n",
    );
    for c in controls {
        out.push_str(&format!("                <group name=\"{}\">\n", esc(&c.name)));
        for (digits, action, data) in &c.details {
            out.push_str(&format!(
                "                    <control digits=\"{}\" action=\"{}\" data=\"{}\"/>\n",
                esc(digits),
                esc(action),
                esc(data)
            ));
        }
        out.push_str("                </group>\n");
    }
    out.push_str("            </caller-controls>\n            <profiles>\n");
    for p in profiles {
        out.push_str(&format!("                <profile name=\"{}\">\n", esc(&p.name)));
        for (name, value) in &p.params {
            out.push_str(&format!("                    <param name=\"{}\" value=\"{}\"/>\n", esc(name), esc(value)));
        }
        out.push_str("                </profile>\n");
    }
    out.push_str("            </profiles>\n        </configuration>\n    </section>\n</document>\n");
    out
}

// ---- local_stream (xml_handler/configuration/local_stream.conf.lua) ----

/// One `v_music_on_hold` row's fields (`name` is already
/// `domain/name[/rate]`-joined by the caller, per Lua).
#[derive(Clone, Debug, Default)]
pub struct LocalStreamInput {
    pub name: String,
    pub uuid: String,
    pub path: String,
    pub rate: Option<String>,
    pub shuffle: Option<String>,
    pub timer_name: Option<String>,
    pub chime_list: Option<String>,
    pub chime_freq: Option<String>,
    pub chime_max: Option<String>,
}

/// `local_stream.conf` document: one `<directory>` per stream.
/// Empty rate defaults to `48000`, empty timer to `soft`
/// (Lua parity); channels/interval are fixed `1`/`20`.
#[must_use]
pub fn build_local_stream_xml(streams: &[LocalStreamInput]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document type=\"freeswitch/xml\">\n    <section name=\"configuration\">\n        <configuration name=\"local_stream.conf\" description=\"stream files from local dir\">\n",
    );
    for s in streams {
        let rate = non_empty(&s.rate).unwrap_or("48000");
        let timer = non_empty(&s.timer_name).unwrap_or("soft");
        out.push_str(&format!(
            "        <directory name=\"{}\" uuid=\"{}\" path=\"{}\">\n",
            esc(&s.name),
            esc(&s.uuid),
            esc(&s.path)
        ));
        out.push_str(&format!("            <param name=\"rate\" value=\"{rate}\"/>\n"));
        out.push_str(&format!(
            "            <param name=\"shuffle\" value=\"{}\"/>\n",
            esc(opt_str(&s.shuffle))
        ));
        out.push_str("            <param name=\"channels\" value=\"1\"/>\n");
        out.push_str("            <param name=\"interval\" value=\"20\"/>\n");
        out.push_str(&format!("            <param name=\"timer-name\" value=\"{timer}\"/>\n",));
        if let Some(v) = non_empty(&s.chime_list) {
            out.push_str(&format!("            <param name=\"chime-list\" value=\"{}\"/>\n", esc(v)));
        }
        if let Some(v) = non_empty(&s.chime_freq) {
            out.push_str(&format!("            <param name=\"chime-freq\" value=\"{}\"/>\n", esc(v)));
        }
        if let Some(v) = non_empty(&s.chime_max) {
            out.push_str(&format!("            <param name=\"chime-max\" value=\"{}\"/>\n", esc(v)));
        }
        out.push_str("        </directory>\n");
    }
    out.push_str("        </configuration>\n    </section>\n</document>\n");
    out
}
