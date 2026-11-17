//! FS XML handler — mirrors `app/freeswitch/xml_router.py`
//! (directory/dialplan/configuration) with a 60s in-memory cache
//! (mirrors `app/freeswitch/cache.py`) and IP-fallback note.
//!
//! Cache sits in front of the SeaORM/postgres render; `invalidate_cache`
//! is the invalidation hook vars PATCH/DELETE already call.

use axum::{http::StatusCode, response::IntoResponse, Extension as AxExtension, Router};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

fn cache() -> &'static tokio::sync::RwLock<pbx_fs::cache::XmlCache> {
    static C: OnceLock<tokio::sync::RwLock<pbx_fs::cache::XmlCache>> = OnceLock::new();
    C.get_or_init(|| tokio::sync::RwLock::new(pbx_fs::cache::XmlCache::new()))
}

/// Invalidate the FS XML cache (called by vars PATCH/DELETE).
pub fn invalidate_cache() {
    // Best-effort: only used in tests/single-threaded paths.
    if let Ok(mut guard) = cache().try_write() {
        *guard = pbx_fs::cache::XmlCache::new();
    }
}

/// Drop one FS XML cache key (Task 5 CRUD triggers: per-key invalidate,
/// not full wipe).
pub fn invalidate_key(key: &str) {
    if let Ok(mut guard) = cache().try_write() {
        guard.remove(key);
    }
}

/// Drop every FS XML cache key with `prefix` (e.g. `configuration:ivr.conf`
/// covers per-menu `configuration:ivr.conf:{uuid}` keys).
pub fn invalidate_prefix(prefix: &str) {
    if let Ok(mut guard) = cache().try_write() {
        guard.remove_prefix(prefix);
    }
}

/// Best-effort FreeSWITCH propagation with LOUD failures: every command runs
/// over real ESL (`try_api`, never the mock fallback); an unreachable switch
/// is logged to stderr so failures are never silently masked. DB writes
/// already succeeded — callers do not fail the HTTP request on ESL errors.
pub fn propagate_esl(commands: &[&str]) {
    let esl = pbx_fs::esl::RealEs::from_env();
    for cmd in commands {
        if esl.try_api(cmd).is_none() {
            eprintln!("[pbx-api] ESL propagate failed (FS down?) cmd={cmd}");
        }
    }
}

/// Switch hostname for hostname-scoped rows (`v_vars.var_hostname`,
/// `v_sip_profiles.hostname`, `v_gateways.hostname`; Lua `hostname` parity).
/// Containers set `HOSTNAME`; empty means "no hostname rows match".
fn local_hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_default()
}

/// Full-domain directory document: every enabled extension with its
/// stored password. mod_sofia resolves REGISTER users from this.
/// Enriched per lua `directory.lua` + PHP `extension.php`: joins
/// `voicemails` (on alias-or-extension) and enabled
/// `extension_settings` rows per extension.
async fn domain_document(domain: &str) -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let domain_id = match pbx_db::entities::domains::Entity::find()
        .filter(pbx_db::entities::domains::Column::DomainName.eq(domain.to_owned()))
        .one(db)
        .await
        .unwrap_or(None)
    {
        Some(d) => d.domain_id,
        None => return pbx_fs::xml::build_directory_xml_multi(domain, &[]),
    };
    let rows = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
        .filter(pbx_db::entities::extensions::Column::Enabled.eq(true))
        .all(db)
        .await
        .unwrap_or_default();
    if rows.is_empty() {
        return pbx_fs::xml::build_directory_xml_multi(domain, &[]);
    }
    let ext_ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.extension_uuid).collect();
    let vms = pbx_db::entities::voicemails::Entity::find()
        .filter(pbx_db::entities::voicemails::Column::DomainId.eq(domain_id))
        .all(db)
        .await
        .unwrap_or_default();
    let settings = pbx_db::entities::extension_settings::Entity::find()
        .filter(
            pbx_db::entities::extension_settings::Column::ExtensionUuid.is_in(ext_ids),
        )
        .filter(
            pbx_db::entities::extension_settings::Column::ExtensionSettingEnabled.eq(true),
        )
        .all(db)
        .await
        .unwrap_or_default();
    let users: Vec<pbx_fs::xml::DirectoryUser> = rows
        .iter()
        .map(|r| {
            // Lua parity: voicemail_id matches the alias when set, else the
            // extension.
            let vm_id = r
                .number_alias
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(r.extension.as_str());
            let vm = vms.iter().find(|v| v.voicemail_id == vm_id);
            let owned: Vec<pbx_db::entities::extension_settings::Model> = settings
                .iter()
                .filter(|s| s.extension_uuid == r.extension_uuid)
                .cloned()
                .collect();
            directory_user_from_row(r, domain, domain_id, vm, &owned)
        })
        .collect();
    pbx_fs::xml::build_directory_xml_users(domain, &users)
}

/// Map one extension row (+ voicemail + settings joins) to the enriched
/// directory struct (lua `directory.lua` variable mapping).
fn directory_user_from_row(
    r: &pbx_db::entities::extensions::Model,
    domain_name: &str,
    domain_id: uuid::Uuid,
    vm: Option<&pbx_db::entities::voicemails::Model>,
    settings: &[pbx_db::entities::extension_settings::Model],
) -> pbx_fs::xml::DirectoryUser {
    let mut u = pbx_fs::xml::DirectoryUser::minimal(
        &r.extension,
        &r.password.clone().unwrap_or_default(),
    );
    u.number_alias.clone_from(&r.number_alias);
    u.cidr.clone_from(&r.cidr);
    u.dial_string.clone_from(&r.dial_string);
    u.call_timeout = r.call_timeout;
    u.user_context.clone_from(&r.user_context);
    u.domain_uuid = Some(domain_id.to_string());
    u.extension_uuid = Some(r.extension_uuid.to_string());
    u.accountcode.clone_from(&r.accountcode);
    u.toll_allow.clone_from(&r.toll_allow);
    u.call_group.clone_from(&r.call_group);
    u.call_screen_enabled = r.call_screen_enabled;
    if !r.effective_caller_id_name.is_empty() {
        u.effective_caller_id_name = Some(r.effective_caller_id_name.clone());
    }
    u.effective_caller_id_number.clone_from(&r.effective_caller_id_number);
    u.outbound_caller_id_name.clone_from(&r.outbound_caller_id_name);
    u.outbound_caller_id_number.clone_from(&r.outbound_caller_id_number);
    u.emergency_caller_id_name.clone_from(&r.emergency_caller_id_name);
    u.emergency_caller_id_number.clone_from(&r.emergency_caller_id_number);
    u.directory_first_name.clone_from(&r.directory_first_name);
    u.directory_last_name.clone_from(&r.directory_last_name);
    u.directory_visible = r.directory_visible;
    u.directory_exten_visible = r.directory_exten_visible;
    u.limit_max.clone_from(&r.limit_max);
    u.limit_destination.clone_from(&r.limit_destination);
    u.missed_call_app.clone_from(&r.missed_call_app);
    u.missed_call_data.clone_from(&r.missed_call_data);
    u.user_record.clone_from(&r.user_record);
    u.hold_music.clone_from(&r.hold_music);
    u.auth_acl.clone_from(&r.auth_acl);
    u.sip_force_contact.clone_from(&r.sip_force_contact);
    u.sip_force_expires = r.sip_force_expires.map(|n| n.to_string());
    u.nibble_account.clone_from(&r.nibble_account);
    u.mwi_account.clone_from(&r.mwi_account);
    u.sip_bypass_media.clone_from(&r.sip_bypass_media);
    u.absolute_codec_string.clone_from(&r.absolute_codec_string);
    u.force_ping = r.force_ping;
    u.max_registrations.clone_from(&r.max_registrations);
    u.forward_all_enabled = r.forward_all_enabled;
    u.forward_all_destination.clone_from(&r.forward_all_destination);
    u.forward_busy_enabled = r.forward_busy_enabled;
    u.forward_busy_destination.clone_from(&r.forward_busy_destination);
    u.forward_no_answer_enabled = r.forward_no_answer_enabled;
    u.forward_no_answer_destination.clone_from(&r.forward_no_answer_destination);
    u.forward_user_not_registered_enabled = r.forward_user_not_registered_enabled;
    u.forward_user_not_registered_destination
        .clone_from(&r.forward_user_not_registered_destination);
    u.follow_me_enabled = r.follow_me_enabled;
    u.do_not_disturb = r.do_not_disturb;
    u.extension_language.clone_from(&r.extension_language);
    u.extension_dialect.clone_from(&r.extension_dialect);
    u.extension_voice.clone_from(&r.extension_voice);
    if let Some(v) = vm {
        u.vm_password = Some(v.voicemail_password.clone());
        u.vm_enabled = if v.enabled {
            "true".to_owned()
        } else {
            "false".to_owned()
        };
        u.vm_mail_to.clone_from(&v.voicemail_mail_to);
        u.vm_attach_file = Some(match v.voicemail_attach_file {
            Some(true) => "true".to_owned(),
            _ => "false".to_owned(),
        });
        u.vm_keep_local_after_email = Some(match v.voicemail_local_after_email {
            Some(false) => "false".to_owned(),
            _ => "true".to_owned(),
        });
    }
    u.settings = settings
        .iter()
        .filter_map(|s| {
            s.extension_setting_type.as_deref().map(|t| pbx_fs::xml::ExtSetting {
                setting_type: t.to_owned(),
                name: s.extension_setting_name.clone(),
                value: s.extension_setting_value.clone(),
            })
        })
        .collect();
    let _ = domain_name;
    u
}

/// Per-user directory document, enriched like the domain document.
/// Strict tenant scoping first (`domain_id` + extension-or-alias); legacy
/// compat shim second (pre-domain rows carry the requested domain in
/// `dial_string`). Unknown user/domain → None (bare-shell fallback).
async fn single_user_document(domain: &str, user: &str) -> Option<String> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    if let Some(dom) = pbx_db::entities::domains::Entity::find()
        .filter(pbx_db::entities::domains::Column::DomainName.eq(domain.to_owned()))
        .one(db)
        .await
        .unwrap_or(None)
    {
        let row = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(dom.domain_id))
            .filter(
                sea_orm::Condition::any()
                    .add(pbx_db::entities::extensions::Column::Extension.eq(user.to_owned()))
                    .add(pbx_db::entities::extensions::Column::NumberAlias.eq(user.to_owned())),
            )
            .filter(pbx_db::entities::extensions::Column::Enabled.eq(true))
            .one(db)
            .await
            .unwrap_or(None);
        if let Some(r) = row {
            return Some(enriched_doc_for_row(domain, dom.domain_id, &r, db).await);
        }
    }
    // Legacy shim: rows created before the domain existed carry the
    // requested domain label in `dial_string` (extensions compat shim).
    let row = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::Extension.eq(user.to_owned()))
        .filter(pbx_db::entities::extensions::Column::DialString.eq(domain.to_owned()))
        .filter(pbx_db::entities::extensions::Column::Enabled.eq(true))
        .one(db)
        .await
        .unwrap_or(None);
    match row {
        Some(r) => {
            let did = r.domain_id;
            Some(enriched_doc_for_row(domain, did, &r, db).await)
        }
        None => None,
    }
}

async fn enriched_doc_for_row(
    domain: &str,
    domain_id: uuid::Uuid,
    r: &pbx_db::entities::extensions::Model,
    db: &sea_orm::DatabaseConnection,
) -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let vm_id = r
        .number_alias
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(r.extension.as_str())
        .to_owned();
    let vm = pbx_db::entities::voicemails::Entity::find()
        .filter(pbx_db::entities::voicemails::Column::DomainId.eq(domain_id))
        .filter(pbx_db::entities::voicemails::Column::VoicemailId.eq(vm_id))
        .one(db)
        .await
        .unwrap_or(None);
    let settings = pbx_db::entities::extension_settings::Entity::find()
        .filter(
            pbx_db::entities::extension_settings::Column::ExtensionUuid.eq(r.extension_uuid),
        )
        .filter(
            pbx_db::entities::extension_settings::Column::ExtensionSettingEnabled.eq(true),
        )
        .all(db)
        .await
        .unwrap_or_default();
    let u = directory_user_from_row(r, domain, domain_id, vm.as_ref(), &settings);
    pbx_fs::xml::build_directory_xml_users(domain, std::slice::from_ref(&u))
}

/// Full-domain dialplan document: every enabled dialplan with its ordered
/// details, grouped by context (PHP `dialplan_xml.php` parity). Falls back
/// to the static skeleton when the DB has no rows.
async fn dialplan_document_for_domain(claims_domain: &str, fallback_ctx: &str) -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let db = crate::routing_db::conn().await;
    let domain_id = match claims_domain.parse::<uuid::Uuid>() {
        Ok(id) => id,
        Err(_) => match pbx_db::entities::domains::Entity::find()
            .filter(pbx_db::entities::domains::Column::DomainName.eq(claims_domain.to_owned()))
            .one(db)
            .await
            .unwrap_or(None)
        {
            Some(d) => d.domain_id,
            None => crate::routing_db::domain(&pbx_core::auth::Claims {
                sub: String::new(),
                domain_id: claims_domain.to_owned(),
                username: String::new(),
                permissions: vec![],
                exp: 0,
                iat: 0,
            }),
        },
    };
    // Tenant isolation: serve only this tenant's dialplans. Unknown tenants
    // (derived UUID with no rows) fall through to the static skeleton in
    // `dialplan_document` — never another tenant's rows.
    dialplan_document(&domain_id, fallback_ctx).await
}

async fn dialplan_document(domain_id: &uuid::Uuid, fallback_ctx: &str) -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let domain_id = *domain_id;
    let rows = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
        .filter(pbx_db::entities::dialplans::Column::Enabled.eq(true))
        .all(db)
        .await
        .unwrap_or_default();
    if rows.is_empty() {
        return pbx_fs::xml::build_dialplan_xml(fallback_ctx);
    }
    let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.dialplan_uuid).collect();
    let dets = pbx_db::entities::dialplan_details::Entity::find()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.is_in(ids))
        .all(db)
        .await
        .unwrap_or_default();
    // Domain name for the public-context prologue (PHP `dialplan.php:944-949`).
    let mut domain_names = std::collections::HashMap::new();
    if let Some(d) = pbx_db::entities::domains::Entity::find_by_id(domain_id)
        .one(db)
        .await
        .unwrap_or(None)
    {
        domain_names.insert(domain_id, d.domain_name);
    }
    render_dialplan_document(&rows, &dets, &domain_names, fallback_ctx)
}

/// PHP `dialplan.php:717-1003` parity: sort dialplans by
/// (context, order, name, uuid) and details by (group, tag rank,
/// detail order), then render the extension/condition/action state
/// machine. Pure function so it is unit-testable without a DB.
fn render_dialplan_document(
    rows: &[pbx_db::entities::dialplans::Model],
    dets: &[pbx_db::entities::dialplan_details::Model],
    domain_names: &std::collections::HashMap<uuid::Uuid, String>,
    fallback_ctx: &str,
) -> String {
    use std::collections::HashMap;
    if rows.is_empty() {
        return pbx_fs::xml::build_dialplan_xml(fallback_ctx);
    }
    // PHP: `dialplan_order, dialplan_name, dialplan_uuid` (NULL order last,
    // Postgres `ASC` semantics); contexts grouped in first-seen order.
    let mut rows: Vec<&pbx_db::entities::dialplans::Model> = rows.iter().collect();
    rows.sort_by(|a, b| {
        a.context
            .cmp(&b.context)
            .then(a.dialplan_order.unwrap_or(i32::MAX).cmp(&b.dialplan_order.unwrap_or(i32::MAX)))
            .then(a.name.cmp(&b.name))
            .then(a.dialplan_uuid.cmp(&b.dialplan_uuid))
    });
    // PHP `and (detail_enabled = true or detail_enabled is null)`.
    let mut by_dp: HashMap<uuid::Uuid, Vec<&pbx_db::entities::dialplan_details::Model>> =
        HashMap::new();
    for d in dets {
        if d.detail_enabled == Some(false) {
            continue;
        }
        by_dp.entry(d.dialplan_uuid).or_default().push(d);
    }
    // PHP tag rank: condition < regex < action < anti-action; then group,
    // then detail order.
    for ds in by_dp.values_mut() {
        ds.sort_by(|a, b| {
            a.dialplan_detail_group
                .unwrap_or(i32::MAX)
                .cmp(&b.dialplan_detail_group.unwrap_or(i32::MAX))
                .then(tag_rank(&a.tag).cmp(&tag_rank(&b.tag)))
                .then(a.order.cmp(&b.order))
        });
    }
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document type=\"freeswitch/xml\">\n<section name=\"dialplan\">\n");
    let mut order: Vec<String> = Vec::new();
    let mut by_ctx: HashMap<String, Vec<&pbx_db::entities::dialplans::Model>> = HashMap::new();
    for r in &rows {
        by_ctx.entry(r.context.clone()).or_insert_with(|| {
            order.push(r.context.clone());
            Vec::new()
        }).push(r);
    }
    for ctx in &order {
        out.push_str(&format!("  <context name=\"{}\">\n", xml_escape(ctx)));
        for r in &by_ctx[ctx] {
            let empty: Vec<&pbx_db::entities::dialplan_details::Model> = Vec::new();
            let ds = by_dp.get(&r.dialplan_uuid).unwrap_or(&empty);
            out.push_str(&render_extension(r, ds, domain_names));
        }
        out.push_str("  </context>\n");
    }
    out.push_str("</section>\n</document>\n");
    out
}

fn tag_rank(tag: &str) -> u8 {
    match tag {
        "condition" => 0,
        "regex" => 1,
        "action" => 2,
        "anti-action" => 3,
        _ => 100,
    }
}

fn is_time_type(t: &str) -> bool {
    matches!(
        t,
        "hour"
            | "minute"
            | "minute-of-day"
            | "mday"
            | "mweek"
            | "mon"
            | "time-of-day"
            | "yday"
            | "year"
            | "wday"
            | "week"
            | "date-time"
    )
}

fn is_public_ctx(ctx: &str) -> bool {
    ctx == "public" || ctx.starts_with("public@") || ctx.ends_with(".public")
}

/// One `<extension>` block: PHP `dialplan.php:826-999` state machine.
/// Conditions wrap their actions; time details merge into a single
/// `<condition hour=... minute=...>`; a new detail group closes the open
/// condition but stays in the same extension.
fn render_extension(
    r: &pbx_db::entities::dialplans::Model,
    ds: &[&pbx_db::entities::dialplan_details::Model],
    domain_names: &std::collections::HashMap<uuid::Uuid, String>,
) -> String {
    let cont = if r.dialplan_continue { "true" } else { "false" };
    let mut ext = format!(
        "    <extension name=\"{}\" continue=\"{}\" uuid=\"{}\">\n",
        xml_escape(&r.name),
        cont,
        r.dialplan_uuid
    );
    // Pending (not yet flushed) condition state.
    let mut pending = String::new();
    let mut pending_attrs = String::new();
    let mut pending_break = String::new();
    let mut cond_open = false;
    let mut first_action = true;
    let mut prev_group: Option<i32> = None;

    // Flush pending condition state into `ext`. `open_children` mirrors
    // PHP's action-arrival path (`...>` + newline, children follow);
    // otherwise self-close (`.../>`) or emit `</condition>`.
    macro_rules! flush_pending {
        ($open_children:expr) => {{
            if !pending_attrs.is_empty() {
                if $open_children {
                    ext.push_str(&format!(
                        "      <condition {}{}>\n",
                        pending_attrs, pending_break
                    ));
                } else {
                    ext.push_str(&format!(
                        "      <condition {}{}/>\n",
                        pending_attrs, pending_break
                    ));
                }
                pending_attrs.clear();
                if !$open_children {
                    cond_open = false;
                }
            } else if pending.ends_with('>') {
                ext.push_str(&pending);
                ext.push('\n');
                pending.clear();
                if !$open_children {
                    cond_open = false;
                }
            } else if !pending.is_empty() {
                if $open_children {
                    ext.push_str(&pending);
                    ext.push_str(">\n");
                } else {
                    ext.push_str(&pending);
                    ext.push_str("/>\n");
                }
                pending.clear();
                if !$open_children {
                    cond_open = false;
                }
            } else if cond_open && !$open_children {
                ext.push_str("      </condition>\n");
                cond_open = false;
            }
        }};
    }

    for d in ds {
        // PHP `$dialplan_detail_data = str_replace("$$", "$", ...)`.
        let data = xml_escape(&d.data.replace("$$", "$"));
        let typ = xml_escape(&d.detail_type);
        let inline_attr = match d.dialplan_detail_inline.as_deref() {
            Some(s) if !s.is_empty() => format!(" inline=\"{}\"", xml_escape(s)),
            _ => String::new(),
        };
        // Group change (or first row): close the previous group's
        // condition; the extension stays open. Prologue re-arms per group.
        if prev_group != d.dialplan_detail_group {
            flush_pending!(false);
            first_action = true;
        }
        if d.tag == "condition" || d.tag == "regex" {
            if cond_open {
                if pending.ends_with('>') {
                    ext.push_str(&pending);
                    ext.push('\n');
                    pending.clear();
                    cond_open = false;
                } else if !pending.is_empty() {
                    ext.push_str(&pending);
                    ext.push_str("/>\n");
                    pending.clear();
                    cond_open = false;
                } else if !pending_attrs.is_empty() && !is_time_type(&d.detail_type) {
                    // Previous rows were time details; the new one is not:
                    // self-close the merged time condition (PHP keeps
                    // accumulating when the new row is also a time type).
                    ext.push_str(&format!(
                        "      <condition {}{}/>\n",
                        pending_attrs, pending_break
                    ));
                    pending_attrs.clear();
                    cond_open = false;
                }
            }
            // PHP passes `break` through only when stored (never forced).
            pending_break = match d.dialplan_detail_break.as_deref() {
                Some(s) if !s.is_empty() => format!(" break=\"{}\"", xml_escape(s)),
                _ => String::new(),
            };
            if is_time_type(&d.detail_type) {
                pending_attrs.push_str(&format!("{}=\"{}\" ", d.detail_type, data));
                pending.clear();
            } else if d.tag == "condition" && d.detail_type == "regex" {
                pending = format!("      <condition regex=\"{}\">", data);
            } else if d.tag == "regex" {
                pending = format!(
                    "      <regex field=\"{}\" expression=\"{}\"{}/>",
                    typ, data, pending_break
                );
            } else {
                pending = format!(
                    "      <condition field=\"{}\" expression=\"{}\"{}",
                    typ, data, pending_break
                );
            }
            cond_open = true;
        }
        if d.tag == "action" || d.tag == "anti-action" {
            if cond_open {
                flush_pending!(true);
            }
            // PHP `dialplan.php:938-953`: public contexts stamp the call
            // direction + tenant domain before the first action figure.
            if is_public_ctx(&r.context) && d.tag == "action" && first_action {
                ext.push_str("        <action application=\"export\" data=\"call_direction=inbound\" inline=\"true\"/>\n");
                ext.push_str(&format!(
                    "        <action application=\"set\" data=\"domain_uuid={}\" inline=\"true\"/>\n",
                    r.domain_id
                ));
                if let Some(name) = domain_names.get(&r.domain_id) {
                    ext.push_str(&format!(
                        "        <action application=\"set\" data=\"domain_name={}\" inline=\"true\"/>\n",
                        xml_escape(name)
                    ));
                }
                first_action = false;
            }
            ext.push_str(&format!(
                "        <{} application=\"{}\" data=\"{}\"{}/>\n",
                d.tag, typ, data, inline_attr
            ));
        }
        prev_group = d.dialplan_detail_group;
    }
    if cond_open {
        flush_pending!(false);
    }
    ext.push_str("    </extension>\n");
    ext
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Task 5: DB-backed `configuration:{section}` documents (PHP
/// `switch.php` + Lua `xml_handler` parity). Unknown sections keep the
/// IP-fallback shell from `build_configuration_xml`.
async fn render_configuration(key_value: &str, menu: &str) -> String {
    if key_value.contains("sofia.conf") {
        return serve_sofia_conf().await;
    }
    if key_value == "vars.xml" || key_value.ends_with("/vars.xml") {
        return serve_vars_xml().await;
    }
    if key_value.contains("callcenter.conf") {
        return serve_callcenter_conf().await;
    }
    if key_value.contains("ivr.conf") {
        return serve_ivr_conf(menu).await;
    }
    if key_value.contains("conference.conf") {
        return serve_conference_conf().await;
    }
    if key_value.contains("local_stream.conf") {
        return serve_local_stream_conf().await;
    }
    pbx_fs::xml::build_configuration_xml(key_value, key_value)
}

/// All domains as `{domain_id: domain_name}` (queue/agent `@domain`
/// suffixes; orphan rows fall back to `tenant-{uuid}`).
async fn domain_name_map(
    db: &sea_orm::DatabaseConnection,
) -> std::collections::HashMap<uuid::Uuid, String> {
    use sea_orm::EntityTrait;
    pbx_db::entities::domains::Entity::find()
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|d| (d.domain_id, d.domain_name))
        .collect()
}

fn domain_label(map: &std::collections::HashMap<uuid::Uuid, String>, id: &uuid::Uuid) -> String {
    map.get(id).cloned().unwrap_or_else(|| format!("tenant-{id}"))
}

/// Hostname scoping (Lua `... or hostname is null or hostname = ''`
/// parity): rows with no hostname always match.
fn hostname_cond<C: sea_orm::ColumnTrait>(col: C, host: &str) -> sea_orm::Condition {
    sea_orm::Condition::any()
        .add(col.is_null())
        .add(col.eq(""))
        .add(col.eq(host.to_owned()))
}

fn gateway_to_input(r: &pbx_db::entities::gateways::Model) -> pbx_fs::config::GatewayInput {
    pbx_fs::config::GatewayInput {
        gateway_uuid: r.gateway_uuid.to_string(),
        username: r.username.clone(),
        password: r.password.clone(),
        proxy: r.proxy.clone(),
        register: r.gateway_register,
        profile: r.profile.clone().unwrap_or_default(),
        context: r.context.clone(),
        enabled: r.enabled,
        distinct_to: r.distinct_to,
        auth_username: r.auth_username.clone(),
        realm: r.realm.clone(),
        from_user: r.from_user.clone(),
        from_domain: r.from_domain.clone(),
        register_proxy: r.register_proxy.clone(),
        outbound_proxy: r.outbound_proxy.clone(),
        expire_seconds: r.expire_seconds,
        register_transport: r.register_transport.clone(),
        contact_params: r.contact_params.clone(),
        retry_seconds: r.retry_seconds,
        extension: r.extension.clone(),
        ping: r.ping.clone(),
        ping_min: r.ping_min,
        ping_max: r.ping_max,
        contact_in_ping: r.contact_in_ping,
        caller_id_in_from: r.caller_id_in_from,
        supress_cng: r.supress_cng,
        sip_cid_type: r.sip_cid_type.clone(),
        extension_in_contact: r.extension_in_contact.clone(),
    }
}

/// `sofia.conf` from rows (never the fallback when rows exist): enabled,
/// hostname-scoped profiles + their enabled settings, domains and enabled,
/// profile/hostname-scoped gateways + enabled global settings.
async fn serve_sofia_conf() -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let db = crate::routing_db::conn().await;
    let host = local_hostname();
    let profiles = pbx_db::entities::sip_profiles::Entity::find()
        .filter(pbx_db::entities::sip_profiles::Column::Enabled.eq(true))
        .filter(hostname_cond(
            pbx_db::entities::sip_profiles::Column::Hostname,
            &host,
        ))
        .order_by_asc(pbx_db::entities::sip_profiles::Column::Name)
        .all(db)
        .await
        .unwrap_or_default();
    if profiles.is_empty() {
        return pbx_fs::sofia::build_sofia_xml(&[]);
    }
    let mut inputs = Vec::with_capacity(profiles.len());
    for p in &profiles {
        let settings = pbx_db::entities::sip_profile_settings::Entity::find()
            .filter(
                pbx_db::entities::sip_profile_settings::Column::SipProfileUuid.eq(p.sip_profile_uuid),
            )
            .filter(pbx_db::entities::sip_profile_settings::Column::Enabled.eq(true))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|s| (s.setting_name, s.setting_value.unwrap_or_default()))
            .collect();
        let domains = pbx_db::entities::sip_profile_domains::Entity::find()
            .filter(
                pbx_db::entities::sip_profile_domains::Column::SipProfileUuid
                    .eq(p.sip_profile_uuid),
            )
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|d| {
                (
                    d.domain_name,
                    d.domain_alias.filter(|s| !s.is_empty()).unwrap_or_else(|| "false".to_owned()),
                    d.domain_parse.filter(|s| !s.is_empty()).unwrap_or_else(|| "false".to_owned()),
                )
            })
            .collect();
        let gateways = pbx_db::entities::gateways::Entity::find()
            .filter(pbx_db::entities::gateways::Column::Profile.eq(p.name.clone()))
            .filter(pbx_db::entities::gateways::Column::Enabled.eq(true))
            .filter(hostname_cond(
                pbx_db::entities::gateways::Column::Hostname,
                &host,
            ))
            .all(db)
            .await
            .unwrap_or_default()
            .iter()
            .map(gateway_to_input)
            .collect();
        inputs.push(pbx_fs::sofia::SofiaProfileInput {
            name: p.name.clone(),
            sip_port: p.sip_port,
            sip_ip: p.sip_ip.clone(),
            ext_sip_ip: p.ext_sip_ip.clone(),
            ext_rtp_ip: p.ext_rtp_ip.clone(),
            rtp_ip: p.rtp_ip.clone(),
            realm: p.realm.clone(),
            context: p.context.clone(),
            challenge_realm: p.challenge_realm.clone(),
            apply_inbound_acl: p.apply_inbound_acl.clone(),
            local_network_acl: p.local_network_acl.clone(),
            hold_music: p.hold_music.clone(),
            record_path: p.record_path.clone(),
            settings,
            domains,
            gateways,
        });
    }
    let globals = pbx_db::entities::sofia_global_settings::Entity::find()
        .filter(pbx_db::entities::sofia_global_settings::Column::GlobalSettingEnabled.eq(true))
        .order_by_asc(
            pbx_db::entities::sofia_global_settings::Column::GlobalSettingName,
        )
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|g| (g.global_setting_name, g.global_setting_value))
        .collect::<Vec<_>>();
    pbx_fs::sofia::build_sofia_xml_with_globals(&inputs, &globals)
}

/// `vars.xml` body from enabled rows (hostname scoping + `Provision`
/// exclusion + `Exec-Set` mapping live in the builder).
async fn serve_vars_xml() -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let rows = pbx_db::entities::vars::Entity::find()
        .filter(pbx_db::entities::vars::Column::VarEnabled.eq(true))
        .all(db)
        .await
        .unwrap_or_default();
    let inputs: Vec<pbx_fs::config::VarInput> = rows
        .iter()
        .map(|v| pbx_fs::config::VarInput {
            category: v.var_category.clone(),
            name: v.var_name.clone(),
            value: v.var_value.clone(),
            command: v.var_command.clone(),
            hostname: v.var_hostname.clone(),
            enabled: v.var_enabled,
            order: v.var_order,
        })
        .collect();
    pbx_fs::config::build_vars_xml(&inputs, &local_hostname())
}

/// `callcenter.conf` from queues/agents/tiers + domain names (PHP
/// `save_call_center_xml` semantics: no enabled filter, matching the file
/// writer and the Lua join which serve every row).
async fn serve_callcenter_conf() -> String {
    use sea_orm::EntityTrait;
    let db = crate::routing_db::conn().await;
    let names = domain_name_map(db).await;
    let queues = pbx_db::entities::call_centers::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let agents = pbx_db::entities::call_center_agents::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let tiers = pbx_db::entities::call_center_tiers::Entity::find()
        .all(db)
        .await
        .unwrap_or_default();
    let by_queue: std::collections::HashMap<String, &pbx_db::entities::call_centers::Model> =
        queues.iter().map(|q| (q.call_center_queue_uuid.clone(), q)).collect();
    let qinputs: Vec<(pbx_fs::config::CallcenterQueueInput, String)> = queues
        .iter()
        .map(|q| {
            (
                pbx_fs::config::CallcenterQueueInput {
                    name: q.queue_name.clone(),
                    extension: q.queue_extension.clone(),
                    strategy: q.queue_strategy.clone(),
                    moh_sound: q.moh_sound.clone(),
                    record_template: q.record_template.clone(),
                    time_base_score: q.time_base_score.clone(),
                    max_wait_time: q.max_wait_time,
                    max_wait_time_with_no_agent: q.max_wait_time_with_no_agent,
                    max_wait_time_with_no_agent_time_reached: q
                        .max_wait_time_with_no_agent_time_reached,
                    tier_rules_apply: q.tier_rules_apply,
                    tier_rule_wait_second: q.tier_rule_wait_second,
                    tier_rule_wait_multiply_level: q.tier_rule_wait_multiply_level,
                    tier_rule_no_agent_no_wait: q.tier_rule_no_agent_no_wait,
                    discard_abandoned_after: q.discard_abandoned_after,
                    abandoned_resume_allowed: q.abandoned_resume_allowed,
                    announce_sound: q.announce_sound.clone(),
                    announce_frequency: q.announce_frequency,
                },
                domain_label(&names, &q.domain_id),
            )
        })
        .collect();
    let ainputs: Vec<(pbx_fs::config::CallcenterAgentInput, String)> = agents
        .iter()
        .map(|a| {
            (
                pbx_fs::config::CallcenterAgentInput {
                    uuid: a.call_center_agent_uuid.clone(),
                    name: a.agent_name.clone(),
                    agent_type: a.agent_type.clone(),
                    contact: a.agent_contact.clone(),
                    status: a.agent_status.clone(),
                    call_timeout: a.agent_call_timeout.unwrap_or(30),
                    no_answer_delay_time: a.agent_no_answer_delay_time,
                    max_no_answer: a.agent_max_no_answer,
                    wrap_up_time: a.agent_wrap_up_time,
                    reject_delay_time: a.agent_reject_delay_time,
                    busy_delay_time: a.agent_busy_delay_time,
                    ..Default::default()
                },
                domain_label(&names, &a.domain_id),
            )
        })
        .collect();
    let tinputs: Vec<(pbx_fs::config::CallcenterTierInput, String)> = tiers
        .iter()
        .map(|t| {
            // Queue side: resolve via the linked queue row first (Lua join
            // parity), fall back to the tier's own queue_name column.
            let qname = t
                .call_center_queue_uuid
                .as_deref()
                .and_then(|id| by_queue.get(id))
                .map(|q| q.queue_name.clone())
                .or_else(|| t.queue_name.clone())
                .unwrap_or_default();
            (
                pbx_fs::config::CallcenterTierInput {
                    agent_name: t.agent_name.clone(),
                    queue_name: qname,
                    tier_level: t.tier_level,
                    tier_position: t.tier_position,
                },
                domain_label(&names, &t.domain_id),
            )
        })
        .collect();
    pbx_fs::config::build_callcenter_xml(&qinputs, &ainputs, &tinputs)
}

/// `ivr.conf`: one menu by `Menu-Name` uuid when given, else every enabled
/// menu, each with its enabled options in order.
async fn serve_ivr_conf(menu_uuid: &str) -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let db = crate::routing_db::conn().await;
    let mut mq = pbx_db::entities::ivr_menus::Entity::find()
        .filter(pbx_db::entities::ivr_menus::Column::Enabled.eq(true));
    if !menu_uuid.is_empty() {
        mq = mq.filter(pbx_db::entities::ivr_menus::Column::IvrMenuUuid.eq(menu_uuid.to_owned()));
    }
    let menus = mq.all(db).await.unwrap_or_default();
    let names = domain_name_map(db).await;
    let mut out: Vec<(pbx_fs::config::IvrMenuInput, Vec<pbx_fs::config::IvrOptionInput>)> =
        Vec::with_capacity(menus.len());
    for m in &menus {
        let opts = pbx_db::entities::ivr_menu_options::Entity::find()
            .filter(
                pbx_db::entities::ivr_menu_options::Column::IvrMenuUuid.eq(m.ivr_menu_uuid.clone()),
            )
            .filter(pbx_db::entities::ivr_menu_options::Column::IvrMenuOptionEnabled.eq(true))
            .order_by_asc(pbx_db::entities::ivr_menu_options::Column::OptionOrder)
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|o| pbx_fs::config::IvrOptionInput {
                action: o.action,
                digits: o.digits,
                param: o.param,
                description: None,
            })
            .collect::<Vec<_>>();
        out.push((
            pbx_fs::config::IvrMenuInput {
                uuid: m.ivr_menu_uuid.clone(),
                name: m.name.clone(),
                description: m.description.clone(),
                greet_long: m.greet_long.clone(),
                greet_short: m.greet_short.clone(),
                invalid_sound: m.invalid_sound.clone(),
                exit_sound: m.exit_sound.clone(),
                pin_number: m.pin_number.clone(),
                confirm_macro: m.confirm_macro.clone(),
                confirm_key: m.confirm_key.clone(),
                tts_engine: m.tts_engine.clone(),
                tts_voice: m.tts_voice.clone(),
                confirm_attempts: m.confirm_attempts,
                timeout: m.timeout,
                inter_digit_timeout: m.inter_digit_timeout,
                max_failures: m.max_failures,
                max_timeouts: m.max_timeouts,
                digit_len: m.digit_len,
                exit_app: m.exit_app.clone(),
                exit_data: m.exit_data.clone(),
                direct_dial: m.direct_dial.unwrap_or(false),
                context: m.context.clone(),
                domain: domain_label(&names, &m.domain_id),
            },
            opts,
        ));
    }
    let refs: Vec<(&pbx_fs::config::IvrMenuInput, &[pbx_fs::config::IvrOptionInput])> = out
        .iter()
        .map(|(m, o)| (m, o.as_slice()))
        .collect();
    pbx_fs::config::build_ivr_xml(&refs)
}

/// `conference.conf` from enabled profiles/params + controls/details.
async fn serve_conference_conf() -> String {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    let db = crate::routing_db::conn().await;
    let profiles = pbx_db::entities::conference_profiles::Entity::find()
        .filter(pbx_db::entities::conference_profiles::Column::ProfileEnabled.eq(true))
        .all(db)
        .await
        .unwrap_or_default();
    let mut pinputs = Vec::with_capacity(profiles.len());
    for p in &profiles {
        let params = pbx_db::entities::conference_profile_params::Entity::find()
            .filter(
                pbx_db::entities::conference_profile_params::Column::ConferenceProfileUuid
                    .eq(p.conference_profile_uuid.clone()),
            )
            .filter(
                pbx_db::entities::conference_profile_params::Column::ProfileParamEnabled.eq(true),
            )
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| {
                (
                    r.profile_param_name.unwrap_or_default(),
                    r.profile_param_value.unwrap_or_default(),
                )
            })
            .collect();
        pinputs.push(pbx_fs::config::ConferenceProfileInput {
            name: p.profile_name.clone(),
            params,
        });
    }
    let controls = pbx_db::entities::conference_controls::Entity::find()
        .filter(pbx_db::entities::conference_controls::Column::ControlEnabled.eq(true))
        .all(db)
        .await
        .unwrap_or_default();
    let mut cinputs = Vec::with_capacity(controls.len());
    for c in &controls {
        let details = pbx_db::entities::conference_control_details::Entity::find()
            .filter(
                pbx_db::entities::conference_control_details::Column::ConferenceControlUuid
                    .eq(c.conference_control_uuid.clone()),
            )
            .filter(
                pbx_db::entities::conference_control_details::Column::ControlEnabled.eq(true),
            )
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| {
                (
                    r.control_digits.unwrap_or_default(),
                    r.control_action.unwrap_or_default(),
                    r.control_data.unwrap_or_default(),
                )
            })
            .collect();
        cinputs.push(pbx_fs::config::ConferenceControlInput {
            name: c.control_name.clone(),
            details,
        });
    }
    pbx_fs::config::build_conference_xml(&pinputs, &cinputs)
}

/// `local_stream.conf` from every music-on-hold row (Lua serves all rows)
/// with `domain/name[/rate]` directory names.
async fn serve_local_stream_conf() -> String {
    use sea_orm::{EntityTrait, QueryOrder};
    let db = crate::routing_db::conn().await;
    let names = domain_name_map(db).await;
    let rows = pbx_db::entities::music_on_hold::Entity::find()
        .order_by_asc(pbx_db::entities::music_on_hold::Column::MusicName)
        .all(db)
        .await
        .unwrap_or_default();
    let inputs: Vec<pbx_fs::config::LocalStreamInput> = rows
        .iter()
        .map(|r| {
            let mut name = format!("{}/{}", domain_label(&names, &r.domain_id), r.music_name);
            if let Some(rate) = r.music_rate {
                name.push_str(&format!("/{rate}"));
            }
            pbx_fs::config::LocalStreamInput {
                name,
                uuid: r.music_on_hold_uuid.clone(),
                path: r.music_path.clone(),
                rate: r.music_rate.map(|n| n.to_string()),
                shuffle: r.music_shuffle.clone(),
                timer_name: r.music_timer_name.clone(),
                chime_list: r.music_chime_list.clone(),
                chime_freq: r.music_chime_freq.clone(),
                chime_max: r.music_chime_max.map(|n| n.to_string()),
            }
        })
        .collect();
    pbx_fs::config::build_local_stream_xml(&inputs)
}
async fn cached_or_render_async<F, Fut>(key: &str, render: F) -> String
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = String>,
{
    // Fast path: async read lock, `Arc` clone on hit (no string copy).
    {
        let guard = cache().read().await;
        if let Some(hit) = guard.get(key) {
            return hit.to_string();
        }
    }
    let doc: Arc<str> = Arc::from(render().await);
    {
        let mut guard = cache().write().await;
        guard.insert(key.to_owned(), Arc::clone(&doc));
    }
    doc.to_string()
}

async fn fs_xml_handler(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Per-request tenant scoping for RLS (no-op on sqlite unit tests).
    let db = crate::routing_db::conn().await;
    crate::routing_db::set_tenant(db, &claims).await;
    // mod_xml_curl POSTs form fields in the body; query params win when set.
    // Collect every body field: dialplan hunts arrive with an empty
    // key_value and carry the context in Hunt-Context/Caller-Context.
    let mut section = params.get("section").cloned().unwrap_or_default();
    let mut key_value = params.get("key_value").cloned().unwrap_or_default();
    let mut body_params: HashMap<String, String> = HashMap::new();
    if !body.is_empty() {
        let text = String::from_utf8_lossy(&body);
        for pair in text.split('&') {
            let (k, v) = match pair.split_once('=') {
                Some(p) => p,
                None => continue,
            };
            // Tiny percent-decode (enough for `user%40domain`).
            let dec = |s: &str| -> String {
                let b = s.as_bytes();
                let mut out = Vec::with_capacity(b.len());
                let mut i = 0;
                while i < b.len() {
                    if b[i] == b'%' && i + 2 < b.len() {
                        let h = |c: u8| match c {
                            b'0'..=b'9' => Some(c - b'0'),
                            b'a'..=b'f' => Some(c - b'a' + 10),
                            b'A'..=b'F' => Some(c - b'A' + 10),
                            _ => None,
                        };
                        if let (Some(h), Some(l)) = (h(b[i + 1]), h(b[i + 2])) {
                            out.push(h << 4 | l);
                            i += 3;
                            continue;
                        }
                    }
                    out.push(if b[i] == b'+' { b' ' } else { b[i] });
                    i += 1;
                }
                String::from_utf8_lossy(&out).into_owned()
            };
            if k == "section" && section.is_empty() {
                section = dec(v);
            } else if k == "key_value" && key_value.is_empty() {
                key_value = dec(v);
            } else {
                body_params.entry(k.to_owned()).or_insert_with(|| dec(v));
            }
        }
    }
    match section.as_str() {
        "directory" => {
            // mod_sofia REGISTER lookups request the whole domain document
            // (`key_value=<domain>`); per-user `user@domain` comes from
            // targeted fetches (fs_cli, tests). A domain-only key must list
            // every extension or REGISTER always 403s.
            let (user, domain) = match key_value.split_once('@') {
                Some((u, d)) => (u.to_owned(), d.to_owned()),
                None => (String::new(), key_value.clone()),
            };
            let key = format!("directory:{domain}:{user}");
            let doc = cached_or_render_async(&key, || async {
                if user.is_empty() {
                    return domain_document(&domain).await;
                }
                // Enriched per-user document (voicemail + settings joins).
                match single_user_document(&domain, &user).await {
                    Some(doc) => doc,
                    // IP fallback: unknown user still renders a shell with the domain.
                    None => pbx_fs::xml::build_directory_xml(&domain, &user),
                }
            })
            .await;
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                doc,
            )
                .into_response()
        }
        "dialplan" => {
            // FS sends an empty key_value for dialplan hunts; the context
            // rides in Hunt-Context/Caller-Context (verified on the wire).
            let hunt_ctx = ["Hunt-Context", "Caller-Context", "context"]
                .iter()
                .filter_map(|k| body_params.get(*k).or_else(|| params.get(*k)))
                .map(|s| s.trim().to_owned())
                .find(|s| !s.is_empty())
                .unwrap_or_else(|| key_value.clone());
            // key_value is the requested context; rows scope by tenant domain.
            let claims_domain = claims.domain_id.clone();
            let key = format!("dialplan:{claims_domain}:{hunt_ctx}");
            let doc = cached_or_render_async(&key, || async move {
                dialplan_document_for_domain(&claims_domain, &hunt_ctx).await
            })
            .await;
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                doc,
            )
                .into_response()
        }
        "configuration" => {
            // Menu-Name rides in the body for `ivr.conf` (Lua parity);
            // per-menu cache key so one menu never serves another.
            let menu = ["Menu-Name", "menu-name"]
                .iter()
                .filter_map(|k| body_params.get(*k).or_else(|| params.get(*k)))
                .map(|s| s.trim().to_owned())
                .find(|s| !s.is_empty())
                .unwrap_or_default();
            let key = if key_value.contains("ivr.conf") && !menu.is_empty() {
                format!("configuration:{key_value}:{menu}")
            } else {
                format!("configuration:{key_value}")
            };
            let kv = key_value.clone();
            let doc = cached_or_render_async(&key, || async move {
                render_configuration(&kv, &menu).await
            })
            .await;
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/xml")],
                doc,
            )
                .into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            JsonNotFound(serde_json::json!({"detail": "unknown section"})),
        )
            .into_response(),
    }
}

struct JsonNotFound(serde_json::Value);

impl IntoResponse for JsonNotFound {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::NOT_FOUND, axum::Json(self.0)).into_response()
    }
}

pub fn fs_xml_router() -> Router {
    // mod_xml_curl may GET or POST; auth is Bearer JWT or Basic
    // `FS_XML_USER`:`FS_XML_PASSWORD` (parity with
    // `app/freeswitch/xml_router.py::check_fs_auth`).
    Router::new()
        .route(
            "/api/internal/freeswitch/xml",
            axum::routing::get(fs_xml_handler).post(fs_xml_handler),
        )
        .layer(axum::middleware::from_fn(crate::require_fs_or_tenant))
}

/// Task 4 (PHP `dialplan.php:717-1042` parity) renderer tests.
///
/// Pure-function tests over `render_dialplan_document`: ordering,
/// time-merge, `$$` unescape, public prologue, disabled filter,
/// break/inline/continue/uuid passthrough.
#[cfg(test)]
mod dialplan_render_tests {
    use super::*;
    use pbx_db::entities::{dialplan_details, dialplans};

    fn domain_id() -> uuid::Uuid {
        uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }

    fn dp(uuid_s: &str, name: &str, ctx: &str, order: i32, cont: bool) -> dialplans::Model {
        dialplans::Model {
            dialplan_uuid: uuid_s.parse().unwrap(),
            name: name.to_owned(),
            context: ctx.to_owned(),
            enabled: true,
            domain_id: domain_id(),
            description: None,
            dialplan_continue: cont,
            dialplan_order: Some(order),
            hostname: None,
            app_uuid: None,
            dialplan_number: None,
            dialplan_destination: None,
            dialplan_xml: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn det(
        dp_uuid_s: &str,
        tag: &str,
        typ: &str,
        data: &str,
        group: i32,
        order: i32,
        brk: Option<&str>,
        inline: Option<&str>,
        enabled: Option<bool>,
    ) -> dialplan_details::Model {
        dialplan_details::Model {
            detail_uuid: uuid::Uuid::new_v4(),
            dialplan_uuid: dp_uuid_s.parse().unwrap(),
            tag: tag.to_owned(),
            detail_type: typ.to_owned(),
            data: data.to_owned(),
            order,
            domain_id: Some(domain_id()),
            dialplan_detail_break: brk.map(|s| s.to_owned()),
            dialplan_detail_inline: inline.map(|s| s.to_owned()),
            dialplan_detail_group: Some(group),
            detail_enabled: enabled,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn domains() -> std::collections::HashMap<uuid::Uuid, String> {
        [(domain_id(), "example.com".to_owned())]
            .into_iter()
            .collect()
    }

    #[test]
    fn extension_carries_continue_and_uuid() {
        let dp_uuid = "22222222-2222-2222-2222-222222222222";
        let rows = vec![dp(dp_uuid, "xfer", "default", 100, true)];
        let dets = vec![
            det(dp_uuid, "condition", "destination_number", "^1000$", 0, 10, None, None, Some(true)),
            det(dp_uuid, "action", "transfer", "1000 XML default", 0, 20, None, None, Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        assert!(
            xml.contains(&format!("<extension name=\"xfer\" continue=\"true\" uuid=\"{dp_uuid}\">")),
            "extension attrs: {xml}"
        );
    }

    #[test]
    fn condition_without_break_has_no_break_attr() {
        let dp_uuid = "33333333-3333-3333-3333-333333333333";
        let rows = vec![dp(dp_uuid, "nobrk", "default", 100, false)];
        let dets = vec![
            det(dp_uuid, "condition", "destination_number", "^1001$", 0, 10, None, None, Some(true)),
            det(dp_uuid, "action", "transfer", "1001 XML default", 0, 20, None, None, Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        assert!(!xml.contains("break="), "no forced break: {xml}");
    }

    #[test]
    fn break_and_inline_passthrough() {
        let dp_uuid = "44444444-4444-4444-4444-444444444444";
        let rows = vec![dp(dp_uuid, "brk", "default", 100, false)];
        let dets = vec![
            det(dp_uuid, "condition", "destination_number", "^1002$", 0, 10, Some("on-false"), None, Some(true)),
            det(dp_uuid, "action", "set", "x=1", 0, 20, None, Some("true"), Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        assert!(xml.contains("break=\"on-false\""), "break passthrough: {xml}");
        assert!(xml.contains("inline=\"true\""), "inline passthrough: {xml}");
    }

    #[test]
    fn time_conditions_merge_into_single_condition() {
        let dp_uuid = "55555555-5555-5555-5555-555555555555";
        let rows = vec![dp(dp_uuid, "tc", "default", 100, false)];
        let dets = vec![
            det(dp_uuid, "condition", "hour", "8-17", 0, 10, None, None, Some(true)),
            det(dp_uuid, "condition", "minute", "0-59", 0, 20, None, None, Some(true)),
            det(dp_uuid, "action", "transfer", "1000 XML default", 0, 30, None, None, Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        assert!(
            xml.contains("hour=\"8-17\"") && xml.contains("minute=\"0-59\""),
            "time merge: {xml}"
        );
        assert_eq!(xml.matches("<condition").count(), 1, "single merged condition: {xml}");
    }

    #[test]
    fn dollar_unescape_and_disabled_filter() {
        let dp_uuid = "66666666-6666-6666-6666-666666666666";
        let rows = vec![dp(dp_uuid, "unesc", "default", 100, false)];
        let dets = vec![
            det(dp_uuid, "condition", "destination_number", "^1003$", 0, 10, None, None, Some(true)),
            det(dp_uuid, "action", "set", "x=$${domain_name}", 0, 20, None, None, Some(true)),
            det(dp_uuid, "action", "hangup", "", 0, 30, None, None, Some(false)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        assert!(xml.contains("${domain_name}"), "$$ unescape: {xml}");
        assert!(!xml.contains("$$"), "no doubled dollars: {xml}");
        assert!(!xml.contains("hangup"), "disabled detail filtered: {xml}");
    }

    #[test]
    fn public_context_gets_inbound_prologue() {
        let dp_uuid = "77777777-7777-7777-7777-777777777777";
        let rows = vec![dp(dp_uuid, "did", "public", 100, false)];
        let dets = vec![
            det(dp_uuid, "condition", "destination_number", "^1555$", 0, 10, None, None, Some(true)),
            det(dp_uuid, "action", "transfer", "1000 XML default", 0, 20, None, None, Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "public");
        assert!(xml.contains("call_direction=inbound"), "prologue: {xml}");
        assert!(xml.contains("domain_uuid=11111111-1111-1111-1111-111111111111"), "prologue uuid: {xml}");
        assert!(xml.contains("domain_name=example.com"), "prologue name: {xml}");
    }

    #[test]
    fn ordering_by_order_name_then_tag_rank() {
        let b_uuid = "88888888-8888-8888-8888-888888888888";
        let a_uuid = "99999999-9999-9999-9999-999999999999";
        // input order scrambled: higher-order dialplan first, action before condition
        let rows = vec![
            dp(b_uuid, "zeta", "default", 200, false),
            dp(a_uuid, "alpha", "default", 100, false),
        ];
        let dets = vec![
            det(b_uuid, "action", "transfer", "x", 0, 10, None, None, Some(true)),
            det(b_uuid, "condition", "destination_number", "^9$", 0, 20, None, None, Some(true)),
            det(a_uuid, "action", "transfer", "y", 0, 10, None, None, Some(true)),
            det(a_uuid, "condition", "destination_number", "^8$", 0, 20, None, None, Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        let pa = xml.find("name=\"alpha\"").unwrap();
        let pz = xml.find("name=\"zeta\"").unwrap();
        assert!(pa < pz, "dialplan_order first: {xml}");
        // condition element precedes action element inside zeta's extension
        let zcond = xml.find("field=\"destination_number\"").unwrap();
        let zact = xml.find("application=\"transfer\"").unwrap();
        assert!(zcond < zact, "condition before action: {xml}");
    }

    #[test]
    fn regex_tag_renders_regex_element() {
        let dp_uuid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let rows = vec![dp(dp_uuid, "rx", "default", 100, false)];
        let dets = vec![
            det(dp_uuid, "regex", "sip_from_uri", "^sip:", 0, 10, None, None, Some(true)),
            det(dp_uuid, "action", "transfer", "1000 XML default", 0, 20, None, None, Some(true)),
        ];
        let xml = render_dialplan_document(&rows, &dets, &domains(), "default");
        assert!(xml.contains("<regex"), "regex element: {xml}");
    }
}
