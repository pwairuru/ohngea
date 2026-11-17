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
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let db = crate::routing_db::conn().await;
    let domain_id = *domain_id;
    let rows = pbx_db::entities::dialplans::Entity::find()
        .filter(pbx_db::entities::dialplans::Column::DomainId.eq(domain_id))
        .filter(pbx_db::entities::dialplans::Column::Enabled.eq(true))
        .order_by_asc(pbx_db::entities::dialplans::Column::DialplanOrder)
        .all(db)
        .await
        .unwrap_or_default();
    if rows.is_empty() {
        return pbx_fs::xml::build_dialplan_xml(fallback_ctx);
    }
    let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.dialplan_uuid).collect();
    let dets = pbx_db::entities::dialplan_details::Entity::find()
        .filter(pbx_db::entities::dialplan_details::Column::DialplanUuid.is_in(ids))
        .order_by_asc(pbx_db::entities::dialplan_details::Column::DialplanDetailGroup)
        .order_by_asc(pbx_db::entities::dialplan_details::Column::Order)
        .all(db)
        .await
        .unwrap_or_default();
    let mut by_dp: std::collections::HashMap<uuid::Uuid, Vec<(String, String, String, Option<i32>)>> =
        std::collections::HashMap::new();
    for d in &dets {
        by_dp.entry(d.dialplan_uuid).or_default().push((
            d.tag.clone(),
            d.detail_type.clone(),
            d.data.clone(),
            d.dialplan_detail_group,
        ));
    }
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document type=\"freeswitch/xml\">\n<section name=\"dialplan\">\n");
    // One <context> per name (FS uses the first same-named block only).
    let mut order: Vec<String> = Vec::new();
    let mut by_ctx: std::collections::HashMap<String, Vec<&pbx_db::entities::dialplans::Model>> =
        std::collections::HashMap::new();
    for r in &rows {
        by_ctx.entry(r.context.clone()).or_insert_with(|| {
            order.push(r.context.clone());
            Vec::new()
        }).push(r);
    }
    for ctx in &order {
        out.push_str(&format!("  <context name=\"{}\">\n", xml_escape(ctx)));
        for r in &by_ctx[ctx] {
        out.push_str(&format!("    <extension name=\"{}\">\n", xml_escape(&r.name)));
        if let Some(ds) = by_dp.get(&r.dialplan_uuid) {
            // FS semantics: actions nest inside an open <condition> block;
            // a new condition closes the previous one (fall-through order).
            let mut open_cond = false;
            for (tag, typ, data, _group) in ds {
                if tag == "condition" {
                    if open_cond {
                        out.push_str("      </condition>\n");
                    }
                    out.push_str(&format!("      <condition field=\"{}\" expression=\"{}\" break=\"on-true\">\n", xml_escape(typ), xml_escape(data)));
                    open_cond = true;
                } else if tag == "action" || tag == "anti-action" {
                    if !open_cond {
                        out.push_str("      <condition>\n");
                        open_cond = true;
                    }
                    out.push_str(&format!("        <{tag} application=\"{}\" data=\"{}\" />\n", xml_escape(typ), xml_escape(data)));
                } else {
                    if !open_cond {
                        out.push_str("      <condition>\n");
                        open_cond = true;
                    }
                    out.push_str(&format!("        <{tag} type=\"{}\" data=\"{}\" />\n", xml_escape(typ), xml_escape(data)));
                }
            }
            if open_cond {
                out.push_str("      </condition>\n");
            }
        }
        out.push_str("    </extension>\n");
        }
        out.push_str("  </context>\n");
    }
    out.push_str("</section>\n</document>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
fn cached_or_render(key: &str, render: impl FnOnce() -> String) -> String {
    // Sync render path (dialplan/configuration): use a blocking read, then
    // a single write holding one `Arc` clone for the store — no double
    // `String` duplication per miss.
    if let Ok(guard) = cache().try_read() {
        if let Some(hit) = guard.get(key) {
            return hit.to_string();
        }
    }
    let doc: Arc<str> = Arc::from(render());
    if let Ok(mut guard) = cache().try_write() {
        guard.insert(key.to_owned(), Arc::clone(&doc));
    }
    doc.to_string()
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
            let key = format!("configuration:{key_value}");
            let doc = cached_or_render(&key, || {
                pbx_fs::xml::build_configuration_xml(&key_value, &key_value)
            });
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
