//! Idempotent seed loaders mirroring app/seeds/*.py.
//! Dedupe keys match Python: vars → (domain_id, var_name, var_category);
//! sofia → global_setting_name (UNIQUE); default_settings → uuid
//! (ON CONFLICT uuid DO NOTHING); domain_settings → (domain, category, subcategory, key).

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::{
    countries, default_settings, dialplan_details, dialplans, domain_settings, domains, groups,
    group_permissions, languages, permissions, sip_profile_domains, sip_profile_settings,
    sip_profiles, sofia_global_settings, user_groups, users, vars,
};

/// One row of app/seeds/vars.py VARS.
pub struct VarSeed {
    pub var_category: &'static str,
    pub var_name: &'static str,
    pub var_value: &'static str,
    pub var_command: Option<&'static str>,
    pub var_enabled: bool,
    pub var_order: i32,
    pub var_description: Option<&'static str>,
    pub var_hostname: Option<&'static str>,
}

/// One row of app/seeds/sofia.py SOFIA_GLOBAL_SETTINGS.
pub struct SofiaSeed {
    pub uuid: &'static str,
    pub name: &'static str,
    pub value: &'static str,
    pub enabled: bool,
    pub description: Option<&'static str>,
}

/// One row of app/seeds/domain_settings.py DOMAIN_DEFAULTS.
pub struct DomainDefaultSeed {
    pub category: &'static str,
    pub subcategory: &'static str,
    pub key: &'static str,
    pub value: &'static str,
}

/// One row of app/seeds/default_settings.py DEFAULT_SETTINGS.
pub struct DefaultSettingSeed {
    pub uuid: &'static str,
    pub category: &'static str,
    pub subcategory: &'static str,
    pub name: &'static str,
    pub value: Option<&'static str>,
    pub enabled: bool,
    pub description: Option<&'static str>,
}

/// One detail row of app/seeds/dialplans.py DEFAULT_DIALPLANS.
pub struct DialplanDetailSeed {
    pub tag: &'static str,
    pub r#type: &'static str,
    pub data: &'static str,
    pub order: i32,
}

/// One row of app/seeds/dialplans.py DEFAULT_DIALPLANS.
pub struct DialplanSeed {
    pub name: &'static str,
    pub order: i32,
    pub description: Option<&'static str>,
    pub details: &'static [DialplanDetailSeed],
}

/// One domain row of app/seeds/sip_profiles.py SIP_PROFILES.
pub struct SipDomainSeed {
    pub name: &'static str,
    pub alias: &'static str,
    pub parse: &'static str,
}

/// One setting row of app/seeds/sip_profiles.py SIP_PROFILES.
pub struct SipSettingSeed {
    pub name: &'static str,
    pub value: Option<&'static str>,
    pub enabled: bool,
}

/// One profile of app/seeds/sip_profiles.py SIP_PROFILES.
pub struct SipProfileSeed {
    pub name: &'static str,
    pub enabled: bool,
    pub port: i32,
    pub description: Option<&'static str>,
    pub domains: &'static [SipDomainSeed],
    pub settings: &'static [SipSettingSeed],
}

include!("seeds_data.rs");
include!("seeds_data_telephony.rs");
include!("seeds_reference.rs");

fn parse_uuid(s: &str) -> Result<Uuid, DbErr> {
    Uuid::parse_str(s).map_err(|e| DbErr::Custom(format!("bad seed uuid {s}: {e}")))
}

/// True when `err` is a unique/duplicate-key violation (sqlite `UNIQUE
/// constraint failed`, postgres `duplicate key value violates unique
/// constraint`). Seed inserts are check-then-insert: concurrent boots race
/// the check, so the loser must treat the conflict as already-seeded
/// instead of failing the whole boot (vars_nonempty_after_boot_seed flake).
fn is_unique_violation(err: &DbErr) -> bool {
    let s = err.to_string().to_lowercase();
    s.contains("unique") || s.contains("duplicate") || s.contains("already exists")
}

/// Insert-or-ignore: `Ok(true)` when the row was inserted, `Ok(false)` when
/// a concurrent boot won the race (unique conflict). Other errors propagate.
fn inserted<T>(res: Result<T, DbErr>) -> Result<bool, DbErr> {
    match res {
        Ok(_) => Ok(true),
        Err(e) if is_unique_violation(&e) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Seed per-domain vars. Returns rows inserted (0 on reseed).
pub async fn seed_vars(db: &DatabaseConnection, domain_id: Uuid) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for s in VARS_SEED {
        let exists = vars::Entity::find()
            .filter(vars::Column::DomainId.eq(domain_id))
            .filter(vars::Column::VarName.eq(s.var_name))
            .filter(vars::Column::VarCategory.eq(s.var_category))
            .one(db)
            .await?;
        if exists.is_some() {
            continue;
        }
        let now = Utc::now();
        let r = vars::Entity::insert(vars::ActiveModel {
            var_uuid: Set(Uuid::now_v7()),
            domain_id: Set(domain_id),
            var_category: Set(s.var_category.to_owned()),
            var_name: Set(s.var_name.to_owned()),
            var_value: Set(s.var_value.to_owned()),
            var_enabled: Set(s.var_enabled),
            var_description: Set(s.var_description.map(str::to_owned)),
            var_order: Set(Some(s.var_order)),
            var_command: Set(s.var_command.map(str::to_owned)),
            var_hostname: Set(s.var_hostname.map(str::to_owned)),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(db)
        .await;
        if inserted(r)? {
            n += 1;
        }
    }
    Ok(n)
}

/// Seed global sofia settings. Returns rows inserted.
pub async fn seed_sofia(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for s in SOFIA_SEED {
        let exists = sofia_global_settings::Entity::find()
            .filter(sofia_global_settings::Column::GlobalSettingName.eq(s.name))
            .one(db)
            .await?;
        if exists.is_some() {
            continue;
        }
        let now = Utc::now();
        let r = sofia_global_settings::Entity::insert(sofia_global_settings::ActiveModel {
            sofia_global_setting_uuid: Set(parse_uuid(s.uuid)?),
            global_setting_name: Set(s.name.to_owned()),
            global_setting_value: Set(s.value.to_owned()),
            global_setting_enabled: Set(s.enabled),
            global_setting_description: Set(s.description.map(str::to_owned)),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(db)
        .await;
        if inserted(r)? {
            n += 1;
        }
    }
    Ok(n)
}

/// Seed per-domain domain_settings. Returns rows inserted.
pub async fn seed_domain_settings(db: &DatabaseConnection, domain_id: Uuid) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for s in DOMAIN_DEFAULTS_SEED {
        let exists = domain_settings::Entity::find()
            .filter(domain_settings::Column::DomainId.eq(domain_id))
            .filter(domain_settings::Column::Category.eq(s.category))
            .filter(domain_settings::Column::Subcategory.eq(s.subcategory))
            .filter(domain_settings::Column::Key.eq(s.key))
            .one(db)
            .await?;
        if exists.is_some() {
            continue;
        }
        let r = domain_settings::Entity::insert(domain_settings::ActiveModel {
            id: Set(Uuid::now_v7()),
            domain_id: Set(domain_id),
            category: Set(s.category.to_owned()),
            subcategory: Set(s.subcategory.to_owned()),
            key: Set(s.key.to_owned()),
            value: Set(s.value.to_owned()),
            enabled: Set(true),
            created_at: Set(Utc::now()),
        })
        .exec(db)
        .await;
        if inserted(r)? {
            n += 1;
        }
    }
    Ok(n)
}

/// Seed global default_settings subset. Returns rows inserted.
pub async fn seed_default_settings(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for s in DEFAULT_SETTINGS_SEED {
        let uuid = parse_uuid(s.uuid)?;
        let exists = default_settings::Entity::find_by_id(uuid).one(db).await?;
        if exists.is_some() {
            continue;
        }
        let now = Utc::now();
        let r = default_settings::Entity::insert(default_settings::ActiveModel {
            default_setting_uuid: Set(uuid),
            default_setting_category: Set(s.category.to_owned()),
            default_setting_subcategory: Set(s.subcategory.to_owned()),
            default_setting_name: Set(s.name.to_owned()),
            default_setting_value: Set(s.value.map(str::to_owned)),
            default_setting_order: Set(None),
            default_setting_enabled: Set(s.enabled),
            default_setting_description: Set(s.description.map(str::to_owned)),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(db)
        .await;
        if inserted(r)? {
            n += 1;
        }
    }
    Ok(n)
}

/// Seed a tenant domain row (parent for all per-domain seeds). Idempotent by id.
pub async fn seed_domains(
    db: &DatabaseConnection,
    domain_id: Uuid,
    domain_name: &str,
) -> Result<u64, DbErr> {
    if domains::Entity::find_by_id(domain_id)
        .one(db)
        .await?
        .is_some()
    {
        return Ok(0);
    }
    let now = Utc::now();
    let r = domains::Entity::insert(domains::ActiveModel {
        domain_id: Set(domain_id),
        domain_name: Set(domain_name.to_owned()),
        enabled: Set(true),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .exec(db)
    .await;
    if inserted(r)? {
        Ok(1)
    } else {
        Ok(0)
    }
}

/// Seed per-domain dialplans + details. Mirrors
/// `app/seeds/dialplans.py::seed_default_dialplans`: context = domain_name,
/// `${domain_name}`/`${domain}` substituted in detail data.
/// Returns rows inserted (4 parents + 8 details = 12 on fresh seed, 0 on reseed).
pub async fn seed_dialplans(
    db: &DatabaseConnection,
    domain_id: Uuid,
    domain_name: &str,
) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for s in DIALPLAN_SEED {
        let exists = dialplans::Entity::find()
            .filter(dialplans::Column::DomainId.eq(domain_id))
            .filter(dialplans::Column::Name.eq(s.name))
            .one(db)
            .await?;
        if exists.is_some() {
            continue;
        }
        let now = Utc::now();
        let dialplan_uuid = Uuid::now_v7();
        let r = dialplans::Entity::insert(dialplans::ActiveModel {
            dialplan_uuid: Set(dialplan_uuid),
            name: Set(s.name.to_owned()),
            context: Set(domain_name.to_owned()),
            enabled: Set(true),
            domain_id: Set(domain_id),
            description: Set(s.description.map(str::to_owned)),
            dialplan_continue: Set(false),
            dialplan_order: Set(Some(s.order)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
        .exec(db)
        .await?;
        n += 1;
        for det in s.details {
            let data = det
                .data
                .replace("${domain_name}", domain_name)
                .replace("${domain}", domain_name);
            let dnow = Utc::now();
            let r = dialplan_details::Entity::insert(dialplan_details::ActiveModel {
                detail_uuid: Set(Uuid::now_v7()),
                dialplan_uuid: Set(dialplan_uuid),
                tag: Set(det.tag.to_owned()),
                detail_type: Set(det.r#type.to_owned()),
                data: Set(data),
                order: Set(det.order),
                domain_id: Set(Some(domain_id)),
                detail_enabled: Set(Some(true)),
                created_at: Set(dnow),
                updated_at: Set(dnow),
                ..Default::default()
            })
            .exec(db)
            .await;
            let inserted_row = inserted(r)?;
            if inserted_row {
                n += 1;
            }
        }
    }
    Ok(n)
}

/// Seed per-domain SIP profiles + domains + settings. Mirrors
/// `app/seeds/sip_profiles.py::seed_sip_profiles`.
///
/// NOTE: all 335 setting rows are inserted verbatim (4 + 4 + 335 = 343 on a
/// fresh seed). Python skips a repeat `setting_name` within one profile when
/// it already exists in the DB; here a pre-existing profile keeps its rows
/// untouched (reseed → 0), so duplicates from the fixture (e.g. two
/// `media-option` rows on `internal`) are preserved exactly as shipped.
pub async fn seed_sip_profiles(db: &DatabaseConnection, domain_id: Uuid) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for p in SIP_PROFILE_SEED {
        let existing = sip_profiles::Entity::find()
            .filter(sip_profiles::Column::DomainId.eq(domain_id))
            .filter(sip_profiles::Column::Name.eq(p.name))
            .one(db)
            .await?;
        if let Some(obj) = existing {
            // upgrade stub rows (name+port only) to full parity on re-seed
            if obj.description.is_none() {
                if let Some(desc) = p.description {
                    let mut am: sip_profiles::ActiveModel = obj.into();
                    am.description = Set(Some(desc.to_owned()));
                    am.updated_at = Set(Utc::now());
                    am.update(db).await?;
                }
            }
            continue;
        }
        let now = Utc::now();
        let sip_profile_uuid = Uuid::now_v7();
        let r = sip_profiles::Entity::insert(sip_profiles::ActiveModel {
            sip_profile_uuid: Set(sip_profile_uuid),
            name: Set(p.name.to_owned()),
            enabled: Set(p.enabled),
            domain_id: Set(domain_id),
            sip_port: Set(p.port),
            description: Set(p.description.map(str::to_owned)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
        .exec(db)
        .await?;
        n += 1;
        for d in p.domains {
            let dnow = Utc::now();
            let r = sip_profile_domains::Entity::insert(sip_profile_domains::ActiveModel {
                sip_profile_domain_uuid: Set(Uuid::now_v7()),
                domain_id: Set(domain_id),
                sip_profile_uuid: Set(sip_profile_uuid),
                domain_name: Set(d.name.to_owned()),
                domain_alias: Set(Some(d.alias.to_owned())),
                domain_parse: Set(Some(d.parse.to_owned())),
                created_at: Set(dnow),
                updated_at: Set(dnow),
            })
            .exec(db)
            .await;
            let inserted_row = inserted(r)?;
            if inserted_row {
                n += 1;
            }
        }
        for s in p.settings {
            let snow = Utc::now();
            let r = sip_profile_settings::Entity::insert(sip_profile_settings::ActiveModel {
                sip_profile_setting_uuid: Set(Uuid::now_v7()),
                domain_id: Set(domain_id),
                sip_profile_uuid: Set(sip_profile_uuid),
                setting_name: Set(s.name.to_owned()),
                setting_value: Set(s.value.map(str::to_owned)),
                enabled: Set(s.enabled),
                created_at: Set(snow),
                updated_at: Set(snow),
                ..Default::default()
            })
            .exec(db)
            .await;
            let inserted_row = inserted(r)?;
            if inserted_row {
                n += 1;
            }
        }
    }
    Ok(n)
}

/// Idempotent boot seed: fills empty tables only (row-count guarded).
/// Ensures the tenant `domains` row first (FK parent), then seeds each
/// empty store: vars + domain_settings + dialplans + sip_profiles
/// (per-domain), sofia + default_settings (global). Returns rows inserted.
/// Safe to call on every boot and concurrently (unique guards dedupe).
/// Seed global reference tables (countries + languages, PHP defaults).
/// Returns rows inserted (0 on reseed).
pub async fn seed_reference(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut n = 0u64;
    let now = Utc::now();
    for (name, code) in LANGUAGE_SEED {
        let am = languages::ActiveModel {
            language_uuid: Set(Uuid::now_v7()),
            language: Set((*name).to_owned()),
            code: Set(Some((*code).to_owned())),
            created_at: Set(now),
        };
        if inserted(am.insert(db).await)? {
            n += 1;
        }
    }
    for (name, a2, a3, num, code) in COUNTRY_SEED {
        let am = countries::ActiveModel {
            country_uuid: Set(Uuid::now_v7()),
            country: Set((*name).to_owned()),
            iso_a2: Set(Some((*a2).to_owned())),
            iso_a3: Set(Some((*a3).to_owned())),
            num: Set(Some(*num)),
            country_code: Set(Some((*code).to_owned())),
            created_at: Set(now),
        };
        if inserted(am.insert(db).await)? {
            n += 1;
        }
    }
    Ok(n)
}

pub async fn seed_on_boot(
    db: &DatabaseConnection,
    domain_id: Uuid,
    domain_name: &str,
) -> Result<u64, DbErr> {
    let mut total = 0u64;
    // Parent first: FK targets in batch1/2a reference domains.
    total += seed_domains(db, domain_id, domain_name).await?;
    // Per-table guards: Entity::find counts (portable sqlite + postgres,
    // no raw-SQL dialect risk). Seed only when empty.
    // (vars/domain_settings/dialplans/sip_profiles scoped to this domain;
    // sofia/default_settings global.)
    let vars_n = vars::Entity::find()
        .filter(vars::Column::DomainId.eq(domain_id))
        .count(db)
        .await?;
    if vars_n == 0 {
        total += seed_vars(db, domain_id).await?;
    }
    let sofia_n = sofia_global_settings::Entity::find().count(db).await?;
    if sofia_n == 0 {
        total += seed_sofia(db).await?;
    }
    let defaults_n = default_settings::Entity::find().count(db).await?;
    if defaults_n == 0 {
        total += seed_default_settings(db).await?;
    }
    let dset_n = domain_settings::Entity::find()
        .filter(domain_settings::Column::DomainId.eq(domain_id))
        .count(db)
        .await?;
    if dset_n == 0 {
        total += seed_domain_settings(db, domain_id).await?;
    }
    let dp_n = dialplans::Entity::find()
        .filter(dialplans::Column::DomainId.eq(domain_id))
        .count(db)
        .await?;
    if dp_n == 0 {
        total += seed_dialplans(db, domain_id, domain_name).await?;
    }
    let sip_n = sip_profiles::Entity::find()
        .filter(sip_profiles::Column::DomainId.eq(domain_id))
        .count(db)
        .await?;
    if sip_n == 0 {
        total += seed_sip_profiles(db, domain_id).await?;
    }
    let lang_n = languages::Entity::find().count(db).await?;
    if lang_n == 0 {
        total += seed_reference(db).await?;
    }
    Ok(total)
}

/// In-memory lookup of a default by (category, subcategory, name).
#[must_use]
pub fn find_default_setting(
    category: &str,
    subcategory: &str,
    name: &str,
) -> Option<&'static DefaultSettingSeed> {
    DEFAULT_SETTINGS_SEED
        .iter()
        .find(|r| r.category == category && r.subcategory == subcategory && r.name == name)
}

/// Resolve an effective setting value: per-domain row wins over the global default.
///
/// NOTE: intentionally does NOT filter on `enabled`. This matches the Python
/// ground truth: `app/seeds/default_settings.py` ships many rows with
/// `enabled: False` still carrying values, `app/vars/service.py::list_vars`
/// filters only by `domain_id` (no enabled predicate), and the only
/// enabled-filtered lookup in `app/admin_settings/router.py::
/// render_email_template` applies to email templates — a different table not
/// handled here. Disabled rows therefore still resolve; callers needing an
/// enabled-only view must filter themselves.
pub async fn resolve_setting(
    db: &DatabaseConnection,
    domain_id: Uuid,
    category: &str,
    subcategory: &str,
    key: &str,
) -> Result<Option<String>, DbErr> {
    if let Some(row) = domain_settings::Entity::find()
        .filter(domain_settings::Column::DomainId.eq(domain_id))
        .filter(domain_settings::Column::Category.eq(category))
        .filter(domain_settings::Column::Subcategory.eq(subcategory))
        .filter(domain_settings::Column::Key.eq(key))
        .one(db)
        .await?
    {
        return Ok(Some(row.value));
    }
    let row = default_settings::Entity::find()
        .filter(default_settings::Column::DefaultSettingCategory.eq(category))
        .filter(default_settings::Column::DefaultSettingSubcategory.eq(subcategory))
        .filter(default_settings::Column::DefaultSettingName.eq(key))
        .one(db)
        .await?;
    Ok(row.and_then(|r| r.default_setting_value))
}

// ---- bootstrap seeds (Task 2: domains bootstrap parity) ----
//
// Mirrors Python `app/domains/service.py::create_domain` + PHP
// `resources/classes/groups.php::defaults()`: global groups, the permission
// catalog the API enforces, full grants to superadmin, and an admin user in
// the new tenant. All idempotent (check-then-insert); safe to re-run.

/// Every permission name the Rust API enforces via `deny_if_missing`, plus
/// `domain_all` (cross-tenant list/get scope). The full 1036-name Python
/// `ALL_PERMISSIONS` union lives in `app/seeds/permissions.py`; the Rust
/// catalog seeds exactly what the Rust handlers check so a fresh bootstrap
/// yields working `user_permissions` rows with no dead names.
pub static BOOTSTRAP_PERMISSIONS: &[&str] = &[
    "access_control_add",
    "access_control_delete",
    "access_control_edit",
    "access_control_view",
    "active_calls_hangup",
    "backup_add",
    "backup_delete",
    "backup_view",
    "bridge_add",
    "bridge_view",
    "call_block_add",
    "call_block_view",
    "call_broadcast_add",
    "call_broadcast_view",
    "call_center_active_view",
    "call_center_add",
    "call_center_view",
    "call_flow_add",
    "call_flow_view",
    "call_forward_add",
    "call_forward_view",
    "cdr_add",
    "cdr_view",
    "conference_active_view",
    "conference_add",
    "conference_center_add",
    "conference_center_delete",
    "conference_center_edit",
    "conference_center_view",
    "conference_control_add",
    "conference_control_view",
    "conference_profile_add",
    "conference_profile_view",
    "conference_room_add",
    "conference_room_delete",
    "conference_room_edit",
    "conference_room_view",
    "conference_session_add",
    "conference_session_delete",
    "conference_session_view",
    "conference_view",
    "contact_add",
    "contact_delete",
    "contact_edit",
    "contact_view",
    "default_setting_add",
    "default_setting_delete",
    "default_setting_edit",
    "default_setting_view",
    "destination_add",
    "destination_view",
    "device_add",
    "device_delete",
    "device_edit",
    "device_view",
    "dialplan_add",
    "dialplan_delete",
    "dialplan_edit",
    "dialplan_view",
    "domain_add",
    "domain_all",
    "domain_delete",
    "domain_edit",
    "domain_view",
    "email_queue_add",
    "email_queue_delete",
    "email_queue_edit",
    "email_queue_view",
    "email_template_add",
    "email_template_delete",
    "email_template_view",
    "emergency_logs_add",
    "emergency_logs_delete",
    "emergency_logs_view",
    "event_guard_add",
    "event_guard_delete",
    "event_guard_view",
    "extension_add",
    "extension_view",
    "fax_add",
    "fax_queue_add",
    "fax_queue_view",
    "fax_view",
    "feature_code_add",
    "feature_code_view",
    "fifo_add",
    "fifo_member_add",
    "fifo_member_view",
    "fifo_view",
    "follow_me_add",
    "follow_me_view",
    "gateway_add",
    "gateway_delete",
    "gateway_edit",
    "gateway_view",
    "group_add",
    "group_delete",
    "group_edit",
    "group_permission_edit",
    "group_permission_view",
    "group_view",
    "ivr_menu_add",
    "ivr_menu_delete",
    "ivr_menu_edit",
    "ivr_menu_view",
    "music_on_hold_add",
    "music_on_hold_delete",
    "music_on_hold_edit",
    "music_on_hold_view",
    "number_translation_add",
    "number_translation_delete",
    "number_translation_edit",
    "number_translation_view",
    "phrase_add",
    "phrase_delete",
    "phrase_edit",
    "phrase_view",
    "pin_number_add",
    "pin_number_delete",
    "pin_number_edit",
    "pin_number_view",
    "recording_add",
    "recording_delete",
    "recording_edit",
    "recording_view",
    "ring_group_add",
    "ring_group_delete",
    "ring_group_edit",
    "ring_group_view",
    "sip_profile_add",
    "sip_profile_delete",
    "sip_profile_domain_add",
    "sip_profile_domain_delete",
    "sip_profile_domain_edit",
    "sip_profile_domain_view",
    "sip_profile_edit",
    "sip_profile_setting_add",
    "sip_profile_setting_delete",
    "sip_profile_setting_edit",
    "sip_profile_setting_view",
    "sip_profile_view",
    "sofia_add",
    "sofia_view",
    "stream_add",
    "stream_view",
    "time_condition_add",
    "time_condition_view",
    "user_add",
    "user_delete",
    "user_edit",
    "user_log_add",
    "user_log_delete",
    "user_log_view",
    "user_setting_view",
    "user_view",
    "var_add",
    "var_delete",
    "var_edit",
    "var_view",
    "voicemail_add",
    "voicemail_view",
    "webrtc_add",
    "webrtc_delete",
    "webrtc_edit",
    "webrtc_view",
];

/// (name, level, protected, description) — PHP `groups.php::defaults()` six
/// plus Python's `gateway` group (union of both ground truths).
pub static BOOTSTRAP_GROUPS: &[(&str, i32, bool, &str)] = &[
    ("superadmin", 80, true, "Super Administrator Group"),
    ("admin", 50, true, "Administrator Group"),
    ("user", 30, false, "User Group"),
    ("agent", 20, false, "Call Center Agent Group"),
    ("fax", 20, false, "Fax User Group"),
    ("public", 10, false, "Public Group"),
    ("gateway", 10, false, "Gateway Group"),
];

/// Seed the global permission catalog. Returns rows inserted (0 on reseed).
pub async fn seed_bootstrap_permissions(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let mut n = 0u64;
    for name in BOOTSTRAP_PERMISSIONS {
        let exists = permissions::Entity::find()
            .filter(permissions::Column::Name.eq((*name).to_owned()))
            .one(db)
            .await?;
        if exists.is_some() {
            continue;
        }
        let r = permissions::Entity::insert(permissions::ActiveModel {
            permission_uuid: Set(Uuid::now_v7()),
            name: Set((*name).to_owned()),
            description: Set(None),
        })
        .exec(db)
        .await;
        if inserted(r)? {
            n += 1;
        }
    }
    Ok(n)
}

/// Seed global groups (`domain_id` NULL, visible to every tenant — PHP
/// `defaults()` parity). Idempotent by name. Returns the superadmin group id.
pub async fn seed_bootstrap_groups(db: &DatabaseConnection) -> Result<Uuid, DbErr> {
    let mut superadmin_id = None;
    for (name, level, protected, desc) in BOOTSTRAP_GROUPS {
        let existing = groups::Entity::find()
            .filter(groups::Column::Name.eq((*name).to_owned()))
            .filter(groups::Column::DomainId.is_null())
            .one(db)
            .await?;
        let gid = match existing {
            Some(g) => g.group_uuid,
            None => {
                let id = Uuid::now_v7();
                let now = Utc::now();
                let r = groups::Entity::insert(groups::ActiveModel {
                    group_uuid: Set(id),
                    domain_id: Set(None),
                    name: Set((*name).to_owned()),
                    description: Set(Some((*desc).to_owned())),
                    group_level: Set(*level),
                    group_protected: Set(*protected),
                    created_at: Set(now),
                    updated_at: Set(now),
                })
                .exec(db)
                .await;
                if inserted(r)? {
                    id
                } else {
                    // Lost a boot race: re-read the winner's id.
                    groups::Entity::find()
                        .filter(groups::Column::Name.eq((*name).to_owned()))
                        .filter(groups::Column::DomainId.is_null())
                        .one(db)
                        .await?
                        .map(|g| g.group_uuid)
                        .unwrap_or(id)
                }
            }
        };
        if *name == "superadmin" {
            superadmin_id = Some(gid);
        }
    }
    superadmin_id.ok_or_else(|| DbErr::Custom("bootstrap groups missing superadmin".to_owned()))
}

/// Grant every catalog permission to `group_uuid` (global rows,
/// `domain_id` NULL — PHP parity). Idempotent. Returns rows inserted.
pub async fn seed_bootstrap_group_permissions(
    db: &DatabaseConnection,
    group_uuid: Uuid,
) -> Result<u64, DbErr> {
    let perms = permissions::Entity::find()
        .filter(permissions::Column::Name.is_in(BOOTSTRAP_PERMISSIONS.iter().copied()))
        .all(db)
        .await?;
    let have: std::collections::HashSet<Uuid> = group_permissions::Entity::find()
        .filter(group_permissions::Column::GroupUuid.eq(group_uuid))
        .all(db)
        .await?
        .into_iter()
        .map(|gp| gp.permission_uuid)
        .collect();
    let mut n = 0u64;
    for p in perms {
        if have.contains(&p.permission_uuid) {
            continue;
        }
        let r = group_permissions::Entity::insert(group_permissions::ActiveModel {
            id: Set(Uuid::now_v7()),
            group_uuid: Set(group_uuid),
            permission_uuid: Set(p.permission_uuid),
            domain_id: Set(None),
        })
        .exec(db)
        .await;
        if inserted(r)? {
            n += 1;
        }
    }
    Ok(n)
}

/// Create the tenant admin user + membership in `group_uuid`, scoped to
/// `domain_id`. Idempotent by (domain_id, username): returns the existing
/// user's id (and ensures membership) on reseed. Returns the user id.
pub async fn seed_bootstrap_admin(
    db: &DatabaseConnection,
    domain_id: Uuid,
    group_uuid: Uuid,
    username: &str,
    password_hash: &str,
) -> Result<Uuid, DbErr> {
    let existing = users::Entity::find()
        .filter(users::Column::DomainId.eq(domain_id))
        .filter(users::Column::Username.eq(username.to_owned()))
        .one(db)
        .await?;
    let uid = match existing {
        Some(u) => u.user_uuid,
        None => {
            let id = Uuid::now_v7();
            let now = Utc::now();
            let r = users::Entity::insert(users::ActiveModel {
                user_uuid: Set(id),
                domain_id: Set(domain_id),
                username: Set(username.to_owned()),
                email: Set(None),
                user_email: Set(None),
                password_hash: Set(password_hash.to_owned()),
                enabled: Set(true),
                user_enabled: Set(Some(true)),
                user_status: Set(None),
                user_time_zone: Set(None),
                user_language: Set(None),
                user_type: Set(None),
                api_key: Set(None),
                contact_uuid: Set(None),
                contact_name_given: Set(None),
                contact_name_family: Set(None),
                contact_organization: Set(None),
                user_totp_secret: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            })
            .exec(db)
            .await;
            if inserted(r)? {
                id
            } else {
                users::Entity::find()
                    .filter(users::Column::DomainId.eq(domain_id))
                    .filter(users::Column::Username.eq(username.to_owned()))
                    .one(db)
                    .await?
                    .map(|u| u.user_uuid)
                    .unwrap_or(id)
            }
        }
    };
    let member = user_groups::Entity::find()
        .filter(user_groups::Column::UserUuid.eq(uid))
        .filter(user_groups::Column::GroupUuid.eq(group_uuid))
        .one(db)
        .await?;
    if member.is_none() {
        let r = user_groups::Entity::insert(user_groups::ActiveModel {
            id: Set(Uuid::now_v7()),
            user_uuid: Set(uid),
            group_uuid: Set(group_uuid),
            domain_id: Set(domain_id),
        })
        .exec(db)
        .await;
        let _ = inserted(r)?;
    }
    Ok(uid)
}
