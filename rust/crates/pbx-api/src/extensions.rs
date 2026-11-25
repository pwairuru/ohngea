//! Extensions router — PHP ground truth (`fusionpbx/app/extensions/`:
//! `extensions.php` list, `extension_edit.php` add/update + `range` batch +
//! voicemail upsert + `limit.extensions` cap, `extension_copy.php` copy,
//! `extension_imports.php` CSV import).
//!
//! SeaORM/postgres backed (`pbx_db::entities::extensions`) with per-request
//! `set_tenant` RLS scoping. Legacy `domain` string from the contract harness
//! is carried in `dial_string` (compat shim); prod scoping uses `domain_id`
//! from verified JWT claims.

use axum::{
    http::StatusCode, response::IntoResponse, routing::post, Extension as AxExtension, Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extension {
    pub extension_uuid: String,
    pub extension: String,
    pub password: String,
    pub domain: String,
    pub number_alias: Option<String>,
    pub accountcode: Option<String>,
    pub effective_caller_id_name: String,
    pub effective_caller_id_number: Option<String>,
    pub outbound_caller_id_name: Option<String>,
    pub outbound_caller_id_number: Option<String>,
    pub emergency_caller_id_name: Option<String>,
    pub emergency_caller_id_number: Option<String>,
    pub directory_first_name: Option<String>,
    pub directory_last_name: Option<String>,
    pub directory_visible: bool,
    pub directory_exten_visible: bool,
    pub max_registrations: Option<String>,
    pub limit_max: Option<String>,
    pub limit_destination: Option<String>,
    pub missed_call_app: Option<String>,
    pub missed_call_data: Option<String>,
    pub user_context: Option<String>,
    pub toll_allow: Option<String>,
    pub call_timeout: Option<i32>,
    pub call_group: Option<String>,
    pub call_screen_enabled: bool,
    pub user_record: Option<String>,
    pub hold_music: Option<String>,
    pub auth_acl: Option<String>,
    pub cidr: Option<String>,
    pub sip_force_contact: Option<String>,
    pub sip_force_expires: Option<i32>,
    pub nibble_account: Option<String>,
    pub mwi_account: Option<String>,
    pub sip_bypass_media: Option<String>,
    pub absolute_codec_string: Option<String>,
    pub force_ping: bool,
    pub dial_string: Option<String>,
    pub do_not_disturb: bool,
    pub forward_all_destination: Option<String>,
    pub forward_all_enabled: bool,
    pub forward_busy_destination: Option<String>,
    pub forward_busy_enabled: bool,
    pub forward_no_answer_destination: Option<String>,
    pub forward_no_answer_enabled: bool,
    pub forward_user_not_registered_destination: Option<String>,
    pub forward_user_not_registered_enabled: bool,
    pub follow_me_uuid: Option<String>,
    pub follow_me_enabled: bool,
    pub extension_language: Option<String>,
    pub extension_dialect: Option<String>,
    pub extension_voice: Option<String>,
    pub extension_type: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub unique_id: Option<String>,
    pub dial_user: Option<String>,
    pub dial_domain: Option<String>,
    pub follow_me_destinations: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExtensionCreate {
    extension: String,
    /// Accepted for Python parity (`ExtensionCreate.domain_id`); honored
    /// only with the `extension_domain` perm (PHP parity — cross-tenant
    /// assign), otherwise the tenant domain from JWT wins.
    #[serde(default)]
    domain_id: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    number_alias: Option<String>,
    #[serde(default)]
    accountcode: Option<String>,
    #[serde(default)]
    effective_caller_id_name: Option<String>,
    #[serde(default)]
    effective_caller_id_number: Option<String>,
    #[serde(default)]
    outbound_caller_id_name: Option<String>,
    #[serde(default)]
    outbound_caller_id_number: Option<String>,
    #[serde(default)]
    emergency_caller_id_name: Option<String>,
    #[serde(default)]
    emergency_caller_id_number: Option<String>,
    #[serde(default)]
    directory_first_name: Option<String>,
    #[serde(default)]
    directory_last_name: Option<String>,
    #[serde(default)]
    directory_visible: Option<bool>,
    #[serde(default)]
    directory_exten_visible: Option<bool>,
    #[serde(default)]
    max_registrations: Option<String>,
    #[serde(default)]
    limit_max: Option<String>,
    #[serde(default)]
    limit_destination: Option<String>,
    #[serde(default)]
    missed_call_app: Option<String>,
    #[serde(default)]
    missed_call_data: Option<String>,
    #[serde(default)]
    user_context: Option<String>,
    #[serde(default)]
    toll_allow: Option<String>,
    #[serde(default)]
    call_timeout: Option<i32>,
    #[serde(default)]
    call_group: Option<String>,
    #[serde(default)]
    call_screen_enabled: Option<bool>,
    #[serde(default)]
    user_record: Option<String>,
    #[serde(default)]
    hold_music: Option<String>,
    #[serde(default)]
    auth_acl: Option<String>,
    #[serde(default)]
    cidr: Option<String>,
    #[serde(default)]
    sip_force_contact: Option<String>,
    #[serde(default)]
    sip_force_expires: Option<i32>,
    #[serde(default)]
    nibble_account: Option<String>,
    #[serde(default)]
    mwi_account: Option<String>,
    #[serde(default)]
    sip_bypass_media: Option<String>,
    #[serde(default)]
    absolute_codec_string: Option<String>,
    #[serde(default)]
    force_ping: Option<bool>,
    #[serde(default)]
    dial_string: Option<String>,
    #[serde(default)]
    do_not_disturb: Option<bool>,
    #[serde(default)]
    forward_all_destination: Option<String>,
    #[serde(default)]
    forward_all_enabled: Option<bool>,
    #[serde(default)]
    forward_busy_destination: Option<String>,
    #[serde(default)]
    forward_busy_enabled: Option<bool>,
    #[serde(default)]
    forward_no_answer_destination: Option<String>,
    #[serde(default)]
    forward_no_answer_enabled: Option<bool>,
    #[serde(default)]
    forward_user_not_registered_destination: Option<String>,
    #[serde(default)]
    forward_user_not_registered_enabled: Option<bool>,
    #[serde(default)]
    follow_me_uuid: Option<String>,
    #[serde(default)]
    follow_me_enabled: Option<bool>,
    #[serde(default)]
    extension_language: Option<String>,
    #[serde(default)]
    extension_dialect: Option<String>,
    #[serde(default)]
    extension_voice: Option<String>,
    #[serde(default)]
    extension_type: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    unique_id: Option<String>,
    #[serde(default)]
    dial_user: Option<String>,
    #[serde(default)]
    dial_domain: Option<String>,
    #[serde(default)]
    follow_me_destinations: Option<String>,
    // Linked voicemail (PHP extension_edit upserts v_voicemails id=extension).
    #[serde(default)]
    voicemail_password: Option<String>,
    #[serde(default)]
    voicemail_enabled: Option<bool>,
    #[serde(default)]
    voicemail_mail_to: Option<String>,
    #[serde(default)]
    voicemail_file: Option<String>,
    #[serde(default)]
    voicemail_local_after_email: Option<bool>,
    #[serde(default)]
    voicemail_transcription_enabled: Option<bool>,
    #[serde(default)]
    voicemail_transcription_prompt: Option<String>,
    #[serde(default)]
    voicemail_transcription_prompt_enabled: Option<bool>,
}

/// List query parity with `app/extensions/router.py::list_ext`:
/// `page`/`per_page` (shared shape), `q` substring filter, `sort`/`order`
/// allowlist, `filter[extension]` alias.
#[derive(Debug, Deserialize)]
struct ListQuery {
    #[serde(default)]
    show: Option<String>,
    #[serde(default = "list_default_page")]
    page: u64,
    #[serde(default = "list_default_per_page")]
    per_page: u64,
    #[serde(default)]
    q: Option<String>,
    #[serde(default = "default_sort")]
    sort: String,
    #[serde(default = "default_order")]
    order: String,
    #[serde(default, rename = "filter[extension]")]
    filter_extension: Option<String>,
}

fn default_sort() -> String {
    "extension".to_owned()
}

fn list_default_page() -> u64 {
    1
}

fn list_default_per_page() -> u64 {
    20
}

fn default_order() -> String {
    "asc".to_owned()
}

use pbx_db::entities::extensions::{Column as ExtCol, Model as ExtModel};

fn row_to_extension(r: &ExtModel, fallback_domain: &str) -> Extension {
    Extension {
        extension_uuid: r.extension_uuid.to_string(),
        extension: r.extension.clone(),
        password: r.password.clone().unwrap_or_default(),
        domain: r
            .dial_string
            .clone()
            .unwrap_or_else(|| fallback_domain.to_owned()),
        number_alias: r.number_alias.clone(),
        accountcode: r.accountcode.clone(),
        effective_caller_id_name: r.effective_caller_id_name.clone(),
        effective_caller_id_number: r.effective_caller_id_number.clone(),
        outbound_caller_id_name: r.outbound_caller_id_name.clone(),
        outbound_caller_id_number: r.outbound_caller_id_number.clone(),
        emergency_caller_id_name: r.emergency_caller_id_name.clone(),
        emergency_caller_id_number: r.emergency_caller_id_number.clone(),
        directory_first_name: r.directory_first_name.clone(),
        directory_last_name: r.directory_last_name.clone(),
        directory_visible: r.directory_visible,
        directory_exten_visible: r.directory_exten_visible,
        max_registrations: r.max_registrations.clone(),
        limit_max: r.limit_max.clone(),
        limit_destination: r.limit_destination.clone(),
        missed_call_app: r.missed_call_app.clone(),
        missed_call_data: r.missed_call_data.clone(),
        user_context: r.user_context.clone(),
        toll_allow: r.toll_allow.clone(),
        call_timeout: r.call_timeout,
        call_group: r.call_group.clone(),
        call_screen_enabled: r.call_screen_enabled,
        user_record: r.user_record.clone(),
        hold_music: r.hold_music.clone(),
        auth_acl: r.auth_acl.clone(),
        cidr: r.cidr.clone(),
        sip_force_contact: r.sip_force_contact.clone(),
        sip_force_expires: r.sip_force_expires,
        nibble_account: r.nibble_account.clone(),
        mwi_account: r.mwi_account.clone(),
        sip_bypass_media: r.sip_bypass_media.clone(),
        absolute_codec_string: r.absolute_codec_string.clone(),
        force_ping: r.force_ping,
        dial_string: r.dial_string.clone(),
        do_not_disturb: r.do_not_disturb,
        forward_all_destination: r.forward_all_destination.clone(),
        forward_all_enabled: r.forward_all_enabled,
        forward_busy_destination: r.forward_busy_destination.clone(),
        forward_busy_enabled: r.forward_busy_enabled,
        forward_no_answer_destination: r.forward_no_answer_destination.clone(),
        forward_no_answer_enabled: r.forward_no_answer_enabled,
        forward_user_not_registered_destination: r.forward_user_not_registered_destination.clone(),
        forward_user_not_registered_enabled: r.forward_user_not_registered_enabled,
        follow_me_uuid: r.follow_me_uuid.map(|u| u.to_string()),
        follow_me_enabled: r.follow_me_enabled,
        extension_language: r.extension_language.clone(),
        extension_dialect: r.extension_dialect.clone(),
        extension_voice: r.extension_voice.clone(),
        extension_type: r.extension_type.clone(),
        enabled: r.enabled,
        description: r.description.clone(),
        unique_id: r.unique_id.clone(),
        dial_user: r.dial_user.clone(),
        dial_domain: r.dial_domain.clone(),
        follow_me_destinations: r.follow_me_destinations.clone(),
    }
}

/// Best-effort FreeSWITCH propagation after extension writes (mirrors PHP
/// `extension_edit.php`: directory XML regen + `reloadacl`/`reloadxml` over
/// the event socket). Cache invalidation is local and always runs; the ESL
/// `reloadxml` degrades to a mock reply when no FS is reachable.
fn propagate_to_freeswitch() {
    crate::fs_xml::invalidate_cache();
    let esl = pbx_fs::esl::RealEs::from_env();
    let _ = esl.try_api("reloadxml");
}

fn validate(ext: &str) -> Result<(), String> {
    pbx_db::extensions::validate_extension(ext)
}

/// PHP `extension_edit.php` parity: `str_replace(',', ':', $toll_allow)`.
fn norm_toll_allow(v: Option<String>) -> Option<String> {
    v.map(|s| s.replace(',', ":"))
}

/// PHP parity: split CIDR list on whitespace/commas, keep entries with a
/// parseable IP (default /32 mask, validated range), drop the rest.
fn norm_cidr(v: Option<String>) -> Option<String> {
    let s = v?;
    let mut out = Vec::new();
    for part in s.split(|c| c == ',' || char::is_whitespace(c)) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (ip, mask) = match part.split_once('/') {
            Some((ip, m)) => (ip, m.parse::<u8>().ok()),
            None => (part, None),
        };
        if let Ok(addr) = ip.parse::<std::net::IpAddr>() {
            let max = if addr.is_ipv4() { 32 } else { 128 };
            let m = mask.unwrap_or(32).min(max);
            out.push(format!("{addr}/{m}"));
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join(","))
    }
}

/// PHP parity: `if (!empty($mwi_account) && strpos($mwi_account,'@')===false)`
/// append `@domain_name`.
fn norm_mwi_account(v: Option<String>, domain: &str) -> Option<String> {
    match v {
        Some(s) if !s.trim().is_empty() && !s.contains('@') => {
            Some(format!("{}@{domain}", s.trim()))
        }
        other => other,
    }
}

/// PHP `extension_edit.php` parity: `$limit = $settings->get('limit',
/// 'extensions')` — block adds at the cap, clamp `range` to the remainder.
/// Settings cascade: domain row first, then the global default row.
async fn extension_limit(db: &sea_orm::DatabaseConnection, domain: uuid::Uuid) -> Option<i64> {
    use pbx_db::entities::{default_settings, domain_settings};
    if let Ok(Some(row)) = domain_settings::Entity::find()
        .filter(domain_settings::Column::DomainId.eq(domain))
        .filter(domain_settings::Column::Category.eq("limit"))
        .filter(domain_settings::Column::Subcategory.eq("extensions"))
        .filter(domain_settings::Column::Enabled.eq(true))
        .one(db)
        .await
    {
        if let Ok(n) = row.value.trim().parse::<i64>() {
            return Some(n);
        }
    }
    if let Ok(Some(row)) = default_settings::Entity::find()
        .filter(default_settings::Column::DefaultSettingCategory.eq("limit"))
        .filter(default_settings::Column::DefaultSettingSubcategory.eq("extensions"))
        .filter(default_settings::Column::DefaultSettingEnabled.eq(true))
        .one(db)
        .await
    {
        if let Some(v) = row.default_setting_value {
            if let Ok(n) = v.trim().parse::<i64>() {
                return Some(n);
            }
        }
    }
    None
}

async fn count_extensions(db: &sea_orm::DatabaseConnection, domain: uuid::Uuid) -> u64 {
    use sea_orm::PaginatorTrait;
    pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain))
        .count(db)
        .await
        .unwrap_or(0)
}

/// 403 when the domain already holds `limit` extensions (PHP
/// `message-maximum_extensions` guard).
async fn over_limit(
    db: &sea_orm::DatabaseConnection,
    domain: uuid::Uuid,
) -> Option<axum::Json<serde_json::Value>> {
    if let Some(limit) = extension_limit(db, domain).await {
        if count_extensions(db, domain).await >= limit.max(0) as u64 {
            return Some(axum::Json(serde_json::json!({
                "detail": format!("maximum extensions {limit} reached")
            })));
        }
    }
    None
}

/// Voicemail fields for the linked-mailbox upsert. Create/batch/import
/// build it from `ExtensionCreate`; copy builds it from the source mailbox
/// row (PHP `extension_copy.php` clones mailbox settings under the new
/// number with a fresh password).
struct MailboxSpec {
    password: Option<String>,
    enabled: Option<bool>,
    mail_to: Option<String>,
    file: Option<String>,
    local_after_email: Option<bool>,
    transcription_enabled: Option<bool>,
    transcription_prompt: Option<String>,
    transcription_prompt_enabled: Option<bool>,
}

impl From<&ExtensionCreate> for MailboxSpec {
    fn from(s: &ExtensionCreate) -> Self {
        Self {
            password: s.voicemail_password.clone(),
            enabled: s.voicemail_enabled,
            mail_to: s.voicemail_mail_to.clone(),
            file: s.voicemail_file.clone(),
            local_after_email: s.voicemail_local_after_email,
            transcription_enabled: s.voicemail_transcription_enabled,
            transcription_prompt: s.voicemail_transcription_prompt.clone(),
            transcription_prompt_enabled: s.voicemail_transcription_prompt_enabled,
        }
    }
}

/// PHP `extension_edit.php` parity ("add or update voicemail"): every
/// extension add upserts the linked mailbox (id = extension), defaulting
/// to enabled with a generated password when the caller passes nothing.
/// Generic over the connection so single create, batch, import and copy
/// (open `txn`) share it.
async fn upsert_linked_mailbox<C>(
    db: &C,
    domain_id: uuid::Uuid,
    ext: &str,
    description: Option<String>,
    src: &MailboxSpec,
) where
    C: sea_orm::ConnectionTrait,
{
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait};
    let existing = pbx_db::entities::voicemails::Entity::find()
        .filter(pbx_db::entities::voicemails::Column::DomainId.eq(domain_id))
        .filter(pbx_db::entities::voicemails::Column::VoicemailId.eq(ext.to_owned()))
        .one(db)
        .await
        .unwrap_or(None);
    match existing {
        Some(row) => {
            let mut vam: pbx_db::entities::voicemails::ActiveModel = row.into();
            if let Some(pw) = src.password.clone() {
                if !pw.trim().is_empty() {
                    vam.voicemail_password = Set(pw);
                }
            }
            if let Some(e) = src.enabled {
                vam.enabled = Set(e);
            }
            if let Some(m) = src.mail_to.clone() {
                vam.voicemail_mail_to = Set(Some(m));
            }
            if let Some(f) = src.file.clone() {
                vam.voicemail_file = Set(Some(f));
            }
            if let Some(l) = src.local_after_email {
                vam.voicemail_local_after_email = Set(Some(l));
            }
            if let Some(t) = src.transcription_enabled {
                vam.voicemail_transcription_enabled = Set(Some(t));
            }
            if let Some(tp) = src.transcription_prompt.clone() {
                vam.voicemail_transcription_prompt = Set(Some(tp));
            }
            if let Some(tpe) = src.transcription_prompt_enabled {
                vam.voicemail_transcription_prompt_enabled = Set(Some(tpe));
            }
            vam.updated_at = Set(crate::routing_db::now());
            let _ = vam.update(db).await;
        }
        None => {
            let vpw = src
                .password
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| pbx_db::extensions::ensure_password(None));
            let vam = pbx_db::entities::voicemails::ActiveModel {
                voicemail_uuid: Set(uuid::Uuid::now_v7()),
                domain_id: Set(Some(domain_id)),
                voicemail_id: Set(ext.to_owned()),
                voicemail_password: Set(vpw),
                enabled: Set(src.enabled.unwrap_or(true)),
                description: Set(description),
                voicemail_tutorial: Set(Some(true)),
                greeting_id: Set(None),
                voicemail_alternate_greet_id: Set(None),
                voicemail_recording_instructions: Set(None),
                voicemail_recording_options: Set(None),
                voicemail_mail_to: Set(src.mail_to.clone()),
                voicemail_file: Set(src.file.clone()),
                voicemail_local_after_email: Set(src.local_after_email),
                voicemail_transcription_enabled: Set(src.transcription_enabled.or(Some(false))),
                voicemail_sms_to: Set(None),
                voicemail_transcription_prompt_enabled: Set(src.transcription_prompt_enabled),
                voicemail_transcription_prompt: Set(src.transcription_prompt.clone()),
                voicemail_attach_file: Set(None),
                voicemail_local_after_forward: Set(None),
                alternate_voicemail_destination: Set(None),
                alternate_voicemail_enabled: Set(None),
                voicemail_name_base64: Set(None),
                created_at: Set(crate::routing_db::now()),
                updated_at: Set(crate::routing_db::now()),
            };
            let _ = vam.insert(db).await;
        }
    }
}

#[tracing::instrument(skip(payload), fields(extension = %payload.extension))]
async fn create_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<ExtensionCreate>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let ext = payload.extension.trim().to_owned();
    if let Err(msg) = validate(&ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": msg})),
        )
            .into_response();
    }
    let domain_str = payload
        .domain
        .clone()
        .unwrap_or_else(|| "example.com".to_owned());
    let password = pbx_db::extensions::ensure_password(payload.password.as_deref());
    let caller_name = payload
        .effective_caller_id_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| ext.clone());
    let db = crate::routing_db::conn().await;
    // PHP `extension_domain` parity: holders may assign another tenant
    // (valid UUID only — never global; extensions have no shared rows).
    // Without the perm the posted value is ignored (legacy behavior).
    let tenant_id = crate::routing_db::tenant_domain(db, &claims).await;
    let domain_id = match payload.domain_id.as_deref().map(str::trim) {
        Some(v) if !v.is_empty() && crate::tenant::has_permission(&claims, "extension_domain") => {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => id,
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_id must be a UUID"})),
                    )
                        .into_response();
                }
            }
        }
        _ => tenant_id,
    };
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "tenant txn failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    // PHP `extension_edit.php` parity: refuse the add when the domain is at
    // its `limit.extensions` cap (`message-maximum_extensions`).
    if let Some(limit) = extension_limit(db, domain_id).await {
        use sea_orm::PaginatorTrait;
        let total = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
            .count(db)
            .await
            .unwrap_or(0);
        if total >= limit.max(0) as u64 {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "detail": format!("maximum extensions {limit} reached")
                })),
            )
                .into_response();
        }
    }
    let now = crate::routing_db::now();
    let user_context = payload
        .user_context
        .clone()
        .unwrap_or_else(|| pbx_db::extensions::default_user_context(None, &domain_str));
    let id = uuid::Uuid::now_v7();
    let follow_me_uuid = payload
        .follow_me_uuid
        .as_deref()
        .and_then(|s| s.parse::<uuid::Uuid>().ok());
    let am = pbx_db::entities::extensions::ActiveModel {
        extension_uuid: Set(id),
        domain_id: Set(domain_id),
        extension: Set(ext.clone()),
        number_alias: Set(payload.number_alias.clone()),
        password: Set(Some(password.clone())),
        accountcode: Set(payload.accountcode.clone()),
        effective_caller_id_name: Set(caller_name.clone()),
        effective_caller_id_number: Set(payload.effective_caller_id_number.clone()),
        outbound_caller_id_name: Set(payload.outbound_caller_id_name.clone()),
        outbound_caller_id_number: Set(payload.outbound_caller_id_number.clone()),
        emergency_caller_id_name: Set(payload.emergency_caller_id_name.clone()),
        emergency_caller_id_number: Set(payload.emergency_caller_id_number.clone()),
        directory_first_name: Set(payload.directory_first_name.clone()),
        directory_last_name: Set(payload.directory_last_name.clone()),
        directory_visible: Set(payload.directory_visible.unwrap_or(false)),
        directory_exten_visible: Set(payload.directory_exten_visible.unwrap_or(false)),
        max_registrations: Set(payload.max_registrations.clone()),
        limit_max: Set(payload.limit_max.clone()),
        limit_destination: Set(payload.limit_destination.clone()),
        missed_call_app: Set(payload.missed_call_app.clone()),
        missed_call_data: Set(payload.missed_call_data.clone()),
        user_context: Set(Some(user_context)),
        toll_allow: Set(norm_toll_allow(payload.toll_allow.clone())),
        call_timeout: Set(payload.call_timeout),
        call_group: Set(payload.call_group.clone()),
        call_screen_enabled: Set(payload.call_screen_enabled.unwrap_or(false)),
        user_record: Set(payload.user_record.clone()),
        hold_music: Set(payload.hold_music.clone()),
        auth_acl: Set(payload.auth_acl.clone()),
        cidr: Set(norm_cidr(payload.cidr.clone())),
        sip_force_contact: Set(payload.sip_force_contact.clone()),
        sip_force_expires: Set(payload.sip_force_expires),
        nibble_account: Set(payload.nibble_account.clone()),
        mwi_account: Set(norm_mwi_account(payload.mwi_account.clone(), &domain_str)),
        sip_bypass_media: Set(payload.sip_bypass_media.clone()),
        absolute_codec_string: Set(payload.absolute_codec_string.clone()),
        force_ping: Set(payload.force_ping.unwrap_or(false)),
        dial_string: Set(payload.dial_string.clone()),
        do_not_disturb: Set(payload.do_not_disturb.unwrap_or(false)),
        forward_all_destination: Set(payload.forward_all_destination.clone()),
        forward_all_enabled: Set(payload.forward_all_enabled.unwrap_or(false)),
        forward_busy_destination: Set(payload.forward_busy_destination.clone()),
        forward_busy_enabled: Set(payload.forward_busy_enabled.unwrap_or(false)),
        forward_no_answer_destination: Set(payload.forward_no_answer_destination.clone()),
        forward_no_answer_enabled: Set(payload.forward_no_answer_enabled.unwrap_or(false)),
        forward_user_not_registered_destination: Set(payload
            .forward_user_not_registered_destination
            .clone()),
        forward_user_not_registered_enabled: Set(payload
            .forward_user_not_registered_enabled
            .unwrap_or(false)),
        follow_me_uuid: Set(follow_me_uuid),
        follow_me_enabled: Set(payload.follow_me_enabled.unwrap_or(false)),
        unique_id: Set(payload.unique_id.clone()),
        dial_user: Set(payload.dial_user.clone()),
        dial_domain: Set(payload.dial_domain.clone()),
        follow_me_destinations: Set(payload.follow_me_destinations.clone()),
        extension_language: Set(payload.extension_language.clone()),
        extension_dialect: Set(payload.extension_dialect.clone()),
        extension_voice: Set(payload.extension_voice.clone()),
        extension_type: Set(payload.extension_type.clone()),
        enabled: Set(payload.enabled.unwrap_or(true)),
        description: Set(payload.description.clone()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    if let Err(e) = am.insert(&txn).await {
        tracing::error!(error = %e, extension = %ext, "extension insert failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
        )
            .into_response();
    }
    // Linked mailbox must ride the tenant txn: pool connections carry no
    // RLS GUC outside a txn, so a post-commit insert on `db` dies on the
    // Postgres `WITH CHECK` (and the `let _` inside the helper swallows it).
    upsert_linked_mailbox(
        &txn,
        domain_id,
        &ext,
        payload.description.clone(),
        &MailboxSpec::from(&payload),
    )
    .await;
    let _ = txn.commit().await;
    propagate_to_freeswitch();
    let db = crate::routing_db::conn().await;
    let rec = pbx_db::entities::extensions::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap_or(None)
        .map(|r| row_to_extension(&r, &domain_str))
        .unwrap_or(Extension {
            extension_uuid: id.to_string(),
            extension: ext,
            password,
            domain: domain_str.clone(),
            number_alias: payload.number_alias.clone(),
            accountcode: payload.accountcode.clone(),
            effective_caller_id_name: caller_name,
            effective_caller_id_number: payload.effective_caller_id_number.clone(),
            outbound_caller_id_name: payload.outbound_caller_id_name.clone(),
            outbound_caller_id_number: payload.outbound_caller_id_number.clone(),
            emergency_caller_id_name: payload.emergency_caller_id_name.clone(),
            emergency_caller_id_number: payload.emergency_caller_id_number.clone(),
            directory_first_name: payload.directory_first_name.clone(),
            directory_last_name: payload.directory_last_name.clone(),
            directory_visible: payload.directory_visible.unwrap_or(false),
            directory_exten_visible: payload.directory_exten_visible.unwrap_or(false),
            max_registrations: payload.max_registrations.clone(),
            limit_max: payload.limit_max.clone(),
            limit_destination: payload.limit_destination.clone(),
            missed_call_app: payload.missed_call_app.clone(),
            missed_call_data: payload.missed_call_data.clone(),
            user_context: payload.user_context.clone().or(Some(domain_str.clone())),
            toll_allow: payload.toll_allow.clone(),
            call_timeout: payload.call_timeout,
            call_group: payload.call_group.clone(),
            call_screen_enabled: payload.call_screen_enabled.unwrap_or(false),
            user_record: payload.user_record.clone(),
            hold_music: payload.hold_music.clone(),
            auth_acl: payload.auth_acl.clone(),
            cidr: payload.cidr.clone(),
            sip_force_contact: payload.sip_force_contact.clone(),
            sip_force_expires: payload.sip_force_expires,
            nibble_account: payload.nibble_account.clone(),
            mwi_account: payload.mwi_account.clone(),
            sip_bypass_media: payload.sip_bypass_media.clone(),
            absolute_codec_string: payload.absolute_codec_string.clone(),
            force_ping: payload.force_ping.unwrap_or(false),
            dial_string: payload.dial_string.clone(),
            do_not_disturb: payload.do_not_disturb.unwrap_or(false),
            forward_all_destination: payload.forward_all_destination.clone(),
            forward_all_enabled: payload.forward_all_enabled.unwrap_or(false),
            forward_busy_destination: payload.forward_busy_destination.clone(),
            forward_busy_enabled: payload.forward_busy_enabled.unwrap_or(false),
            forward_no_answer_destination: payload.forward_no_answer_destination.clone(),
            forward_no_answer_enabled: payload.forward_no_answer_enabled.unwrap_or(false),
            forward_user_not_registered_destination: payload
                .forward_user_not_registered_destination
                .clone(),
            forward_user_not_registered_enabled: payload
                .forward_user_not_registered_enabled
                .unwrap_or(false),
            follow_me_uuid: payload.follow_me_uuid.clone(),
            follow_me_enabled: payload.follow_me_enabled.unwrap_or(false),
            extension_language: payload.extension_language.clone(),
            extension_dialect: payload.extension_dialect.clone(),
            extension_voice: payload.extension_voice.clone(),
            extension_type: payload.extension_type.clone(),
            enabled: payload.enabled.unwrap_or(true),
            description: payload.description.clone(),
            unique_id: payload.unique_id.clone(),
            dial_user: payload.dial_user.clone(),
            dial_domain: payload.dial_domain.clone(),
            follow_me_destinations: payload.follow_me_destinations.clone(),
        });
    (StatusCode::CREATED, Json(serde_json::json!(rec))).into_response()
}

async fn list_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Query(pq): axum::extract::Query<ListQuery>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_view") {
        return (s, j).into_response();
    }
    let (per_page, _) = crate::PageQuery {
        show: None,
        page: pq.page,
        per_page: pq.per_page,
    }
    .normalized();
    let page = pq.page.max(1);
    let db = crate::routing_db::conn().await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(_) => {
            return (
                crate::paged_headers("/api/v1/extensions", page, per_page, 0),
                Json(
                    serde_json::json!({"data": [], "total": 0, "page": page, "per_page": per_page}),
                ),
            )
                .into_response();
        }
    };
    let show_all = crate::tenant::wants_all(&claims, pq.show.as_deref(), "extension_all");
    if show_all {
        crate::routing_db::set_tenant_all(&txn).await;
    }
    let domain_id = crate::routing_db::domain(&claims);
    // `q` parity with Python `list_extensions` service: substring match on
    // extension / effective_caller_id_name / number_alias / description.
    // `filter[extension]` acts as `q`. sqlite LIKE is ASCII case-insensitive,
    // matching Python ilike closely enough for the contract harness.
    let q = pq
        .filter_extension
        .or(pq.q)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let mut sel = pbx_db::entities::extensions::Entity::find();
    if !show_all {
        sel = sel.filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id));
    }
    if let Some(ref needle) = q {
        let pat = format!("%{needle}%");
        sel = sel.filter(
            sea_orm::Condition::any()
                .add(ExtCol::Extension.like(pat.clone()))
                .add(ExtCol::EffectiveCallerIdName.like(pat.clone()))
                .add(ExtCol::NumberAlias.like(pat.clone()))
                .add(ExtCol::Description.like(pat)),
        );
    }
    // sort allowlist parity with Python service (unknown keys -> extension).
    let desc = pq.order.eq_ignore_ascii_case("desc");
    let order = if desc {
        sea_orm::Order::Desc
    } else {
        sea_orm::Order::Asc
    };
    sel = match pq.sort.as_str() {
        "effective_caller_id_name" => sel.order_by(ExtCol::EffectiveCallerIdName, order),
        "created_at" => sel.order_by(ExtCol::CreatedAt, order),
        "updated_at" => sel.order_by(ExtCol::UpdatedAt, order),
        "enabled" => sel.order_by(ExtCol::Enabled, order),
        "number_alias" => sel.order_by(ExtCol::NumberAlias, order),
        "call_timeout" => sel.order_by(ExtCol::CallTimeout, order),
        _ => sel.order_by(ExtCol::Extension, order),
    };
    let paginator = sel.paginate(&txn, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page(page.saturating_sub(1))
        .await
        .unwrap_or_default();
    let _ = txn.commit().await;
    let items: Vec<Extension> = rows
        .into_iter()
        .map(|r| row_to_extension(&r, &claims.domain_id))
        .collect();
    (
        crate::paged_headers("/api/v1/extensions", page, per_page, total),
        Json(crate::paged_body(items, total, page, per_page)),
    )
        .into_response()
}

/// Lookup used by the FS XML directory handler (SeaORM backed).
/// Tenant-scoped: resolves `domain` to its `domain_id` via the `domains`
/// table, then matches extension-or-alias within that tenant only.
/// Unknown domains or foreign-tenant users yield None (no cross-serve).
pub async fn find_extension(user: &str, domain: &str) -> Option<Extension> {
    let db = crate::routing_db::conn().await;
    let dom = pbx_db::entities::domains::Entity::find()
        .filter(pbx_db::entities::domains::Column::DomainName.eq(domain.to_owned()))
        .one(db)
        .await
        .unwrap_or(None)?;
    let row = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(dom.domain_id))
        .filter(
            sea_orm::Condition::any()
                .add(ExtCol::Extension.eq(user.to_owned()))
                .add(ExtCol::NumberAlias.eq(user.to_owned())),
        )
        .one(db)
        .await
        .unwrap_or(None)?;
    Some(row_to_extension(&row, domain))
}

/// Field appliers for PATCH (parity with Python `setattr` loop over table
/// columns minus keys). `null` clears an optional; absent leaves untouched.
fn apply_opt_str(slot: &mut sea_orm::ActiveValue<Option<String>>, v: &serde_json::Value) {
    if v.is_null() {
        *slot = Set(None);
    } else if let Some(s) = v.as_str() {
        *slot = Set(Some(s.to_owned()));
    }
}

fn apply_bool(slot: &mut sea_orm::ActiveValue<bool>, v: &serde_json::Value) {
    if let Some(b) = v.as_bool() {
        *slot = Set(b);
    }
}

fn apply_i32(slot: &mut sea_orm::ActiveValue<Option<i32>>, v: &serde_json::Value) {
    if v.is_null() {
        *slot = Set(None);
    } else if let Some(n) = v.as_i64() {
        *slot = Set(Some(n as i32));
    }
}

/// Apply a PATCH payload to an extension row. Returns Err(detail) for blank
/// or overlong `extension` (PHP `empty($extension)` rule → 422).
fn apply_patch(
    am: &mut pbx_db::entities::extensions::ActiveModel,
    payload: &serde_json::Value,
) -> Result<(), String> {
    if let Some(v) = payload.get("extension") {
        if let Some(s) = v.as_str() {
            let ext = s.trim().to_owned();
            validate(&ext)?;
            am.extension = Set(ext);
        }
    }
    if let Some(v) = payload.get("password").and_then(|v| v.as_str()) {
        am.password = Set(Some(v.to_owned()));
    }
    if let Some(v) = payload.get("number_alias") {
        apply_opt_str(&mut am.number_alias, v);
    }
    if let Some(v) = payload.get("accountcode") {
        apply_opt_str(&mut am.accountcode, v);
    }
    if let Some(v) = payload.get("effective_caller_id_name") {
        if let Some(s) = v.as_str() {
            if !s.trim().is_empty() {
                am.effective_caller_id_name = Set(s.to_owned());
            }
        }
    }
    if let Some(v) = payload.get("effective_caller_id_number") {
        apply_opt_str(&mut am.effective_caller_id_number, v);
    }
    for key in [
        "outbound_caller_id_name",
        "outbound_caller_id_number",
        "emergency_caller_id_name",
        "emergency_caller_id_number",
        "directory_first_name",
        "directory_last_name",
        "max_registrations",
        "limit_max",
        "limit_destination",
        "missed_call_app",
        "missed_call_data",
        "user_context",
        "toll_allow",
        "call_group",
        "user_record",
        "hold_music",
        "auth_acl",
        "cidr",
        "sip_force_contact",
        "nibble_account",
        "mwi_account",
        "sip_bypass_media",
        "absolute_codec_string",
        "dial_string",
        "forward_all_destination",
        "forward_busy_destination",
        "forward_no_answer_destination",
        "forward_user_not_registered_destination",
        "extension_language",
        "extension_dialect",
        "extension_voice",
        "extension_type",
        "description",
    ] {
        if let Some(v) = payload.get(key) {
            let slot: &mut sea_orm::ActiveValue<Option<String>> = match key {
                "outbound_caller_id_name" => &mut am.outbound_caller_id_name,
                "outbound_caller_id_number" => &mut am.outbound_caller_id_number,
                "emergency_caller_id_name" => &mut am.emergency_caller_id_name,
                "emergency_caller_id_number" => &mut am.emergency_caller_id_number,
                "directory_first_name" => &mut am.directory_first_name,
                "directory_last_name" => &mut am.directory_last_name,
                "max_registrations" => &mut am.max_registrations,
                "limit_max" => &mut am.limit_max,
                "limit_destination" => &mut am.limit_destination,
                "missed_call_app" => &mut am.missed_call_app,
                "missed_call_data" => &mut am.missed_call_data,
                "user_context" => &mut am.user_context,
                "toll_allow" => &mut am.toll_allow,
                "call_group" => &mut am.call_group,
                "user_record" => &mut am.user_record,
                "hold_music" => &mut am.hold_music,
                "auth_acl" => &mut am.auth_acl,
                "cidr" => &mut am.cidr,
                "sip_force_contact" => &mut am.sip_force_contact,
                "nibble_account" => &mut am.nibble_account,
                "mwi_account" => &mut am.mwi_account,
                "sip_bypass_media" => &mut am.sip_bypass_media,
                "absolute_codec_string" => &mut am.absolute_codec_string,
                "dial_string" => &mut am.dial_string,
                "forward_all_destination" => &mut am.forward_all_destination,
                "forward_busy_destination" => &mut am.forward_busy_destination,
                "forward_no_answer_destination" => &mut am.forward_no_answer_destination,
                "forward_user_not_registered_destination" => {
                    &mut am.forward_user_not_registered_destination
                }
                "extension_language" => &mut am.extension_language,
                "extension_dialect" => &mut am.extension_dialect,
                "extension_voice" => &mut am.extension_voice,
                "extension_type" => &mut am.extension_type,
                _ => &mut am.description,
            };
            apply_opt_str(slot, v);
        }
    }
    for (key, slot) in [
        ("directory_visible", &mut am.directory_visible),
        ("directory_exten_visible", &mut am.directory_exten_visible),
        ("call_screen_enabled", &mut am.call_screen_enabled),
        ("force_ping", &mut am.force_ping),
        ("do_not_disturb", &mut am.do_not_disturb),
        ("forward_all_enabled", &mut am.forward_all_enabled),
        ("forward_busy_enabled", &mut am.forward_busy_enabled),
        (
            "forward_no_answer_enabled",
            &mut am.forward_no_answer_enabled,
        ),
        (
            "forward_user_not_registered_enabled",
            &mut am.forward_user_not_registered_enabled,
        ),
        ("follow_me_enabled", &mut am.follow_me_enabled),
        ("enabled", &mut am.enabled),
    ] {
        if let Some(v) = payload.get(key) {
            apply_bool(slot, v);
        }
    }
    if let Some(v) = payload.get("call_timeout") {
        apply_i32(&mut am.call_timeout, v);
    }
    if let Some(v) = payload.get("sip_force_expires") {
        apply_i32(&mut am.sip_force_expires, v);
    }
    if let Some(v) = payload.get("follow_me_uuid") {
        if v.is_null() {
            am.follow_me_uuid = Set(None);
        } else if let Some(s) = v.as_str() {
            if let Ok(u) = s.parse::<uuid::Uuid>() {
                am.follow_me_uuid = Set(Some(u));
            }
        }
    }
    am.updated_at = Set(crate::routing_db::now());
    Ok(())
}

async fn find_extension_row(
    db: &sea_orm::DatabaseConnection,
    domain_id: uuid::Uuid,
    id: uuid::Uuid,
) -> Option<pbx_db::entities::extensions::Model> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    pbx_db::entities::extensions::Entity::find_by_id(id)
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
        .one(db)
        .await
        .unwrap_or(None)
}

async fn get_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_view") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    match find_extension_row(db, domain_id, id).await {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::json!(row_to_extension(&r, &claims.domain_id))),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response(),
    }
}

#[tracing::instrument(skip(payload), fields(id = %id))]
async fn patch_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let payload = crate::alias::normalize(payload);
    // Python parity (`extensions/router.py::patch_ext`): `extension_add` gates edits.
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(row) = find_extension_row(db, domain_id, id).await else {
        tracing::warn!(id = %id, "extension not found");
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response();
    };
    let mut am: pbx_db::entities::extensions::ActiveModel = row.into();
    // PHP `extension_domain` parity: holders may re-home to another tenant
    // (valid UUID only — extensions are never global).
    if let Some(v) = payload
        .get("domain_uuid")
        .or_else(|| payload.get("domain_id"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if crate::tenant::has_permission(&claims, "extension_domain") {
            match v.parse::<uuid::Uuid>() {
                Ok(id) => am.domain_id = Set(id),
                Err(_) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": "domain_uuid must be a UUID"})),
                    )
                        .into_response();
                }
            }
        }
    }
    if let Err(detail) = apply_patch(&mut am, &payload) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": detail})),
        )
            .into_response();
    }
    use sea_orm::ActiveModelTrait;
    match am.update(db).await {
        Ok(r) => {
            propagate_to_freeswitch();
            (
                StatusCode::OK,
                Json(serde_json::json!(row_to_extension(&r, &claims.domain_id))),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, id = %id, "extension update failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("update failed: {e}")})),
            )
                .into_response()
        }
    }
}

#[tracing::instrument(fields(id = %id))]
async fn delete_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let Some(row) = find_extension_row(db, domain_id, id).await else {
        tracing::warn!(id = %id, "extension not found");
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response();
    };
    let am: pbx_db::entities::extensions::ActiveModel = row.into();
    use sea_orm::ActiveModelTrait;
    match am.delete(db).await {
        Ok(_) => {
            propagate_to_freeswitch();
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, id = %id, "extension delete failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("delete failed: {e}")})),
            )
                .into_response()
        }
    }
}

/// POST /api/v1/extensions:batch — bulk create (parity with
/// Bulk create. Accepts either a JSON array of extension objects (Python
/// `:batch` shape; duplicate → 409) or a PHP `range` object
/// (`{"start": "1001", "range": 5, ...template}` — sequential numbers from
/// `extension_edit.php`'s `$extension++` loop; existing numbers are skipped,
/// the run is clamped to the remaining `limit.extensions` cap).
/// Returns 201 `{"created": n}`.
#[tracing::instrument(skip(payload))]
async fn batch_create_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    // Range form (`{start, range}`) skips existing numbers PHP-style;
    // array form 409s on the first duplicate (Python `:batch` contract).
    let is_range = payload.as_array().is_none();
    let items: Vec<ExtensionCreate> = if let Some(arr) = payload.as_array() {
        match serde_json::from_value::<Vec<ExtensionCreate>>(serde_json::Value::Array(arr.clone()))
        {
            Ok(v) => v,
            Err(e) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({"detail": format!("invalid batch: {e}")})),
                )
                    .into_response();
            }
        }
    } else if payload.get("start").and_then(|s| s.as_str()).is_some() {
        // PHP range form: numeric start, sequential `$extension++`.
        let start = payload
            .get("start")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .trim()
            .to_owned();
        let base: i64 = match start.parse() {
            Ok(n) => n,
            Err(_) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({"detail": "range start must be numeric"})),
                )
                    .into_response();
            }
        };
        let n = payload
            .get("range")
            .and_then(|r| r.as_u64())
            .unwrap_or(1)
            .min(1000);
        let mut template = payload.as_object().cloned().unwrap_or_default();
        template.remove("start");
        template.remove("range");
        let mut out = Vec::new();
        for i in 0..n {
            let mut row = serde_json::Value::Object(template.clone());
            row["extension"] = serde_json::json!(format!("{}", base + i as i64));
            match serde_json::from_value::<ExtensionCreate>(row) {
                Ok(item) => out.push(item),
                Err(e) => {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        Json(serde_json::json!({"detail": format!("invalid range template: {e}")})),
                    )
                        .into_response();
                }
            }
        }
        out
    } else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "expected array or {start, range} object"})),
        )
            .into_response();
    };
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "tenant txn failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    // PHP parity: clamp the run to the remaining domain cap.
    let mut remaining: Option<u64> = None;
    if let Some(limit) = extension_limit(db, domain_id).await {
        use sea_orm::PaginatorTrait;
        let total = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
            .count(db)
            .await
            .unwrap_or(0);
        let left = (limit.max(0) as u64).saturating_sub(total);
        if left == 0 {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "detail": format!("maximum extensions {limit} reached")
                })),
            )
                .into_response();
        }
        remaining = Some(left);
    }
    let now = crate::routing_db::now();
    let mut created = 0u64;
    let mut skipped = 0u64;
    for item in &items {
        if let Some(left) = remaining {
            if created >= left {
                break;
            }
        }
        let ext = item.extension.trim().to_owned();
        if let Err(msg) = validate(&ext) {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": format!("invalid extension {ext}: {msg}")})),
            )
                .into_response();
        }
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let dup = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
            .filter(pbx_db::entities::extensions::Column::Extension.eq(ext.clone()))
            .one(&txn)
            .await
            .unwrap_or(None);
        if dup.is_some() {
            if is_range {
                // PHP `extension_exists` skip: range runs keep numbering,
                // existing numbers are silently skipped.
                skipped += 1;
                continue;
            }
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"detail": format!("Extension {ext} already exists")})),
            )
                .into_response();
        }
        let password = pbx_db::extensions::ensure_password(item.password.as_deref());
        let caller_name = item
            .effective_caller_id_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| ext.clone());
        let follow_me_uuid = item
            .follow_me_uuid
            .as_deref()
            .and_then(|s| s.parse::<uuid::Uuid>().ok());
        let am = pbx_db::entities::extensions::ActiveModel {
            extension_uuid: Set(uuid::Uuid::now_v7()),
            domain_id: Set(domain_id),
            extension: Set(ext.clone()),
            number_alias: Set(item.number_alias.clone()),
            password: Set(Some(password)),
            accountcode: Set(item.accountcode.clone()),
            effective_caller_id_name: Set(caller_name),
            effective_caller_id_number: Set(item.effective_caller_id_number.clone()),
            outbound_caller_id_name: Set(item.outbound_caller_id_name.clone()),
            outbound_caller_id_number: Set(item.outbound_caller_id_number.clone()),
            emergency_caller_id_name: Set(item.emergency_caller_id_name.clone()),
            emergency_caller_id_number: Set(item.emergency_caller_id_number.clone()),
            directory_first_name: Set(item.directory_first_name.clone()),
            directory_last_name: Set(item.directory_last_name.clone()),
            directory_visible: Set(item.directory_visible.unwrap_or(false)),
            directory_exten_visible: Set(item.directory_exten_visible.unwrap_or(false)),
            max_registrations: Set(item.max_registrations.clone()),
            limit_max: Set(item.limit_max.clone()),
            limit_destination: Set(item.limit_destination.clone()),
            missed_call_app: Set(item.missed_call_app.clone()),
            missed_call_data: Set(item.missed_call_data.clone()),
            user_context: Set(Some(item.user_context.clone().unwrap_or_else(|| {
                pbx_db::extensions::default_user_context(None, &claims.domain_id)
            }))),
            toll_allow: Set(norm_toll_allow(item.toll_allow.clone())),
            call_timeout: Set(item.call_timeout),
            call_group: Set(item.call_group.clone()),
            call_screen_enabled: Set(item.call_screen_enabled.unwrap_or(false)),
            user_record: Set(item.user_record.clone()),
            hold_music: Set(item.hold_music.clone()),
            auth_acl: Set(item.auth_acl.clone()),
            cidr: Set(norm_cidr(item.cidr.clone())),
            sip_force_contact: Set(item.sip_force_contact.clone()),
            sip_force_expires: Set(item.sip_force_expires),
            nibble_account: Set(item.nibble_account.clone()),
            mwi_account: Set(norm_mwi_account(
                item.mwi_account.clone(),
                &claims.domain_id,
            )),
            sip_bypass_media: Set(item.sip_bypass_media.clone()),
            absolute_codec_string: Set(item.absolute_codec_string.clone()),
            force_ping: Set(item.force_ping.unwrap_or(false)),
            dial_string: Set(item.dial_string.clone()),
            do_not_disturb: Set(item.do_not_disturb.unwrap_or(false)),
            forward_all_destination: Set(item.forward_all_destination.clone()),
            forward_all_enabled: Set(item.forward_all_enabled.unwrap_or(false)),
            forward_busy_destination: Set(item.forward_busy_destination.clone()),
            forward_busy_enabled: Set(item.forward_busy_enabled.unwrap_or(false)),
            forward_no_answer_destination: Set(item.forward_no_answer_destination.clone()),
            forward_no_answer_enabled: Set(item.forward_no_answer_enabled.unwrap_or(false)),
            forward_user_not_registered_destination: Set(item
                .forward_user_not_registered_destination
                .clone()),
            forward_user_not_registered_enabled: Set(item
                .forward_user_not_registered_enabled
                .unwrap_or(false)),
            follow_me_uuid: Set(follow_me_uuid),
            follow_me_enabled: Set(item.follow_me_enabled.unwrap_or(false)),
            unique_id: Set(item.unique_id.clone()),
            dial_user: Set(item.dial_user.clone()),
            dial_domain: Set(item.dial_domain.clone()),
            follow_me_destinations: Set(item.follow_me_destinations.clone()),
            extension_language: Set(item.extension_language.clone()),
            extension_dialect: Set(item.extension_dialect.clone()),
            extension_voice: Set(item.extension_voice.clone()),
            extension_type: Set(item.extension_type.clone()),
            enabled: Set(item.enabled.unwrap_or(true)),
            description: Set(item.description.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        use sea_orm::ActiveModelTrait;
        if let Err(e) = am.insert(&txn).await {
            tracing::error!(error = %e, extension = %ext, "extension insert failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("insert failed: {e}")})),
            )
                .into_response();
        }
        upsert_linked_mailbox(
            &txn,
            domain_id,
            &ext,
            item.description.clone(),
            &MailboxSpec::from(item),
        )
        .await;
        created += 1;
    }
    let _ = txn.commit().await;
    propagate_to_freeswitch();
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"created": created, "skipped": skipped})),
    )
        .into_response()
}

/// Copy an extension (`extension_copy.php` parity): duplicate every column
/// of the source row under a new UUID + new extension number (409 when the
/// target exists), fresh generated password, description suffixed ` (copy)`.
/// Requires the `extension_copy` permission.
#[tracing::instrument(skip(payload), fields(id = %id))]
async fn copy_extension(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_copy") {
        return (s, j).into_response();
    }
    let new_ext = payload
        .get("extension")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_owned();
    if let Err(msg) = validate(&new_ext) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": msg})),
        )
            .into_response();
    }
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    if over_limit(db, domain_id).await.is_some() {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"detail": "maximum extensions reached"})),
        )
            .into_response();
    }
    let src = pbx_db::entities::extensions::Entity::find_by_id(id)
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
        .one(db)
        .await
        .unwrap_or(None);
    let Some(src) = src else {
        tracing::warn!(id = %id, "extension copy rejected: source not found");
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "Extension not found"})),
        )
            .into_response();
    };
    let dup = pbx_db::entities::extensions::Entity::find()
        .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
        .filter(pbx_db::entities::extensions::Column::Extension.eq(new_ext.clone()))
        .one(db)
        .await
        .unwrap_or(None);
    if dup.is_some() {
        tracing::warn!(extension = %new_ext, "extension copy rejected: duplicate");
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": format!("Extension {new_ext} already exists")})),
        )
            .into_response();
    }
    let now = crate::routing_db::now();
    let src_desc = src.description.clone().unwrap_or_default();
    // PHP copy reads the source mailbox by number_alias (numeric) else extension.
    let src_vmid = src
        .number_alias
        .clone()
        .filter(|a| !a.trim().is_empty() && a.trim().chars().all(|c| c.is_ascii_digit()))
        .unwrap_or_else(|| src.extension.clone());
    let mut am: pbx_db::entities::extensions::ActiveModel = src.into();
    am.extension_uuid = Set(uuid::Uuid::now_v7());
    am.extension = Set(new_ext.clone());
    if let Some(a) = payload.get("number_alias").and_then(|v| v.as_str()) {
        am.number_alias = Set(Some(a.to_owned()));
    }
    // PHP `extension_copy.php`: fresh password on every copy. The source
    // accountcode is preserved as-is; PHP's copy script assigns an unset
    // `$password` variable there (upstream bug), which we do not replicate.
    am.password = Set(Some(pbx_db::extensions::ensure_password(None)));
    am.description = Set(Some(format!("{src_desc} (copy)").trim_start().to_owned()));
    am.created_at = Set(now);
    am.updated_at = Set(now);
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "tenant txn failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    use sea_orm::ActiveModelTrait;
    if let Err(e) = am.insert(&txn).await {
        tracing::error!(error = %e, extension = %new_ext, "extension copy failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"detail": format!("copy failed: {e}")})),
        )
            .into_response();
    }
    // PHP `extension_copy.php`: clone the source mailbox settings under the
    // new number with a fresh password. Same txn so Postgres RLS sees it.
    {
        use sea_orm::{ColumnTrait, EntityTrait};
        let src_vm = pbx_db::entities::voicemails::Entity::find()
            .filter(pbx_db::entities::voicemails::Column::DomainId.eq(domain_id))
            .filter(pbx_db::entities::voicemails::Column::VoicemailId.eq(src_vmid))
            .one(&txn)
            .await
            .unwrap_or(None);
        if let Some(vm) = src_vm {
            let spec = MailboxSpec {
                password: None,
                enabled: Some(vm.enabled),
                mail_to: vm.voicemail_mail_to.clone(),
                file: vm.voicemail_file.clone(),
                local_after_email: vm.voicemail_local_after_email,
                transcription_enabled: vm.voicemail_transcription_enabled,
                transcription_prompt: vm.voicemail_transcription_prompt.clone(),
                transcription_prompt_enabled: vm.voicemail_transcription_prompt_enabled,
            };
            upsert_linked_mailbox(
                &txn,
                domain_id,
                &new_ext,
                Some(format!("{src_desc} (copy)").trim_start().to_owned()),
                &spec,
            )
            .await;
        }
    }
    let _ = txn.commit().await;
    propagate_to_freeswitch();
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"extension": new_ext})),
    )
        .into_response()
}

/// CSV-style import (`extension_imports.php` parity): accepts
/// `{"extensions": [...]}` rows, inserts valid new rows, collects per-row
/// errors (blank/invalid extension, duplicates, domain cap) without
/// aborting the batch. Returns 201 `{"imported": n, "errors": [...]}`.
#[tracing::instrument(skip(payload))]
async fn import_extensions(
    AxExtension(claims): AxExtension<pbx_core::auth::Claims>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some((s, j)) = crate::tenant::deny_if_missing(&claims, "extension_add") {
        return (s, j).into_response();
    }
    let rows = payload
        .get("extensions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let db = crate::routing_db::conn().await;
    let domain_id = crate::routing_db::tenant_domain(db, &claims).await;
    let txn = match crate::routing_db::tenant_txn(db, &claims).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "tenant txn failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"detail": format!("txn failed: {e}")})),
            )
                .into_response();
        }
    };
    let now = crate::routing_db::now();
    let mut imported = 0u64;
    let mut errors = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let item: ExtensionCreate = match serde_json::from_value(row.clone()) {
            Ok(v) => v,
            Err(e) => {
                errors
                    .push(serde_json::json!({"row": i + 1, "error": format!("invalid row: {e}")}));
                continue;
            }
        };
        let ext = item.extension.trim().to_owned();
        if let Err(msg) = validate(&ext) {
            errors.push(serde_json::json!({"row": i + 1, "error": msg}));
            continue;
        }
        if over_limit(db, domain_id).await.is_some() {
            errors.push(serde_json::json!({"row": i + 1, "error": "maximum extensions reached"}));
            continue;
        }
        let dup = pbx_db::entities::extensions::Entity::find()
            .filter(pbx_db::entities::extensions::Column::DomainId.eq(domain_id))
            .filter(pbx_db::entities::extensions::Column::Extension.eq(ext.clone()))
            .one(&txn)
            .await
            .unwrap_or(None);
        if dup.is_some() {
            errors.push(
                serde_json::json!({"row": i + 1, "error": format!("Extension {ext} already exists")}),
            );
            continue;
        }
        let password = pbx_db::extensions::ensure_password(item.password.as_deref());
        let caller_name = item
            .effective_caller_id_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| ext.clone());
        let am = pbx_db::entities::extensions::ActiveModel {
            extension_uuid: Set(uuid::Uuid::now_v7()),
            domain_id: Set(domain_id),
            extension: Set(ext.clone()),
            number_alias: Set(item.number_alias.clone()),
            password: Set(Some(password)),
            accountcode: Set(item.accountcode.clone()),
            effective_caller_id_name: Set(caller_name),
            effective_caller_id_number: Set(item.effective_caller_id_number.clone()),
            outbound_caller_id_name: Set(item.outbound_caller_id_name.clone()),
            outbound_caller_id_number: Set(item.outbound_caller_id_number.clone()),
            emergency_caller_id_name: Set(item.emergency_caller_id_name.clone()),
            emergency_caller_id_number: Set(item.emergency_caller_id_number.clone()),
            directory_first_name: Set(item.directory_first_name.clone()),
            directory_last_name: Set(item.directory_last_name.clone()),
            directory_visible: Set(item.directory_visible.unwrap_or(false)),
            directory_exten_visible: Set(item.directory_exten_visible.unwrap_or(false)),
            max_registrations: Set(item.max_registrations.clone()),
            limit_max: Set(item.limit_max.clone()),
            limit_destination: Set(item.limit_destination.clone()),
            missed_call_app: Set(item.missed_call_app.clone()),
            missed_call_data: Set(item.missed_call_data.clone()),
            user_context: Set(Some(item.user_context.clone().unwrap_or_else(|| {
                pbx_db::extensions::default_user_context(None, &claims.domain_id)
            }))),
            toll_allow: Set(norm_toll_allow(item.toll_allow.clone())),
            call_timeout: Set(item.call_timeout),
            call_group: Set(item.call_group.clone()),
            call_screen_enabled: Set(item.call_screen_enabled.unwrap_or(false)),
            user_record: Set(item.user_record.clone()),
            hold_music: Set(item.hold_music.clone()),
            auth_acl: Set(item.auth_acl.clone()),
            cidr: Set(norm_cidr(item.cidr.clone())),
            sip_force_contact: Set(item.sip_force_contact.clone()),
            sip_force_expires: Set(item.sip_force_expires),
            nibble_account: Set(item.nibble_account.clone()),
            mwi_account: Set(norm_mwi_account(
                item.mwi_account.clone(),
                &claims.domain_id,
            )),
            sip_bypass_media: Set(item.sip_bypass_media.clone()),
            absolute_codec_string: Set(item.absolute_codec_string.clone()),
            force_ping: Set(item.force_ping.unwrap_or(false)),
            dial_string: Set(item.dial_string.clone()),
            do_not_disturb: Set(item.do_not_disturb.unwrap_or(false)),
            forward_all_destination: Set(item.forward_all_destination.clone()),
            forward_all_enabled: Set(item.forward_all_enabled.unwrap_or(false)),
            forward_busy_destination: Set(item.forward_busy_destination.clone()),
            forward_busy_enabled: Set(item.forward_busy_enabled.unwrap_or(false)),
            forward_no_answer_destination: Set(item.forward_no_answer_destination.clone()),
            forward_no_answer_enabled: Set(item.forward_no_answer_enabled.unwrap_or(false)),
            forward_user_not_registered_destination: Set(item
                .forward_user_not_registered_destination
                .clone()),
            forward_user_not_registered_enabled: Set(item
                .forward_user_not_registered_enabled
                .unwrap_or(false)),
            follow_me_uuid: Set(item
                .follow_me_uuid
                .as_deref()
                .and_then(|s| s.parse::<uuid::Uuid>().ok())),
            follow_me_enabled: Set(item.follow_me_enabled.unwrap_or(false)),
            unique_id: Set(item.unique_id.clone()),
            dial_user: Set(item.dial_user.clone()),
            dial_domain: Set(item.dial_domain.clone()),
            follow_me_destinations: Set(item.follow_me_destinations.clone()),
            extension_language: Set(item.extension_language.clone()),
            extension_dialect: Set(item.extension_dialect.clone()),
            extension_voice: Set(item.extension_voice.clone()),
            extension_type: Set(item.extension_type.clone()),
            enabled: Set(item.enabled.unwrap_or(true)),
            description: Set(item.description.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        if let Err(e) = am.insert(&txn).await {
            tracing::warn!(row = i + 1, extension = %ext, error = %e, "extension import row failed");
            errors.push(serde_json::json!({"row": i + 1, "error": format!("insert failed: {e}")}));
            continue;
        }
        upsert_linked_mailbox(
            &txn,
            domain_id,
            &ext,
            item.description.clone(),
            &MailboxSpec::from(&item),
        )
        .await;
        imported += 1;
    }
    let _ = txn.commit().await;
    if imported > 0 {
        propagate_to_freeswitch();
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"imported": imported, "errors": errors})),
    )
        .into_response()
}

pub fn extensions_router() -> Router {
    Router::new()
        .route(
            "/api/v1/extensions",
            post(create_extension).get(list_extensions),
        )
        // Trailing-slash alias (Python router accepts both).
        .route(
            "/api/v1/extensions/",
            post(create_extension).get(list_extensions),
        )
        // Bulk create (PHP range form + Python `:batch` array form).
        .route("/api/v1/extensions:batch", post(batch_create_extensions))
        // CSV-style import (extension_imports.php parity).
        .route("/api/v1/extensions/import", post(import_extensions))
        .route("/api/v1/extensions/{id}/copy", post(copy_extension))
        .route(
            "/api/v1/extensions/{id}",
            axum::routing::get(get_extension)
                .patch(patch_extension)
                .delete(delete_extension),
        )
        .layer(axum::middleware::from_fn(crate::require_tenant))
}
