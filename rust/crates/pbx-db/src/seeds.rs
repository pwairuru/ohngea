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
    countries, default_settings, dialplan_details, dialplans, domain_settings, domains,
    group_permissions, groups, languages, permissions, sip_profile_domains, sip_profile_settings,
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
            domain_id: Set(Some(domain_id)),
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
    "domain_setting_add",
    "domain_setting_all",
    "domain_setting_category_edit",
    "domain_setting_delete",
    "domain_setting_edit",
    "domain_setting_view",
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
    "service_add",
    "service_all",
    "service_delete",
    "service_edit",
    "service_view",
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
    // --- PHP permission-vocabulary parity (all fusionpbx app_config.php
    // `permissions` names; assignable catalog, enforced at CRUD granularity) ---
    "access_control_node_add",
    "access_control_node_delete",
    "access_control_node_edit",
    "access_control_node_view",
    "active_call_barge",
    "active_call_whisper",
    "active_queue_add",
    "active_queue_delete",
    "active_queue_edit",
    "active_queue_view",
    "api_key",
    "bridge_all",
    "bridge_delete",
    "bridge_destinations",
    "bridge_edit",
    "bridge_import",
    "call_active_all",
    "call_active_application",
    "call_active_codec",
    "call_active_direction",
    "call_active_domain",
    "call_active_eavesdrop",
    "call_active_hangup",
    "call_active_park",
    "call_active_profile",
    "call_active_rec",
    "call_active_secure",
    "call_active_transfer",
    "call_active_view",
    "call_block_all",
    "call_block_delete",
    "call_block_domain",
    "call_block_edit",
    "call_block_extension",
    "call_block_ivr",
    "call_block_ring_group",
    "call_block_voicemail",
    "call_broadcast_accountcode",
    "call_broadcast_all",
    "call_broadcast_caller_id",
    "call_broadcast_concurrent_limit",
    "call_broadcast_delete",
    "call_broadcast_destination_number",
    "call_broadcast_edit",
    "call_broadcast_phone_numbers",
    "call_broadcast_send",
    "call_broadcast_start_time",
    "call_broadcast_timeout",
    "call_broadcast_toll_allow",
    "call_broadcast_voicemail_detection",
    "call_center_active_options",
    "call_center_agent_add",
    "call_center_agent_delete",
    "call_center_agent_edit",
    "call_center_agent_view",
    "call_center_all",
    "call_center_announce_frequency",
    "call_center_announce_position",
    "call_center_announce_sound",
    "call_center_destinations",
    "call_center_email_address",
    "call_center_log_add",
    "call_center_log_delete",
    "call_center_log_edit",
    "call_center_log_view",
    "call_center_outbound_caller_id_name",
    "call_center_outbound_caller_id_number",
    "call_center_queue_add",
    "call_center_queue_context",
    "call_center_queue_delete",
    "call_center_queue_edit",
    "call_center_queue_view",
    "call_center_tier_add",
    "call_center_tier_delete",
    "call_center_tier_edit",
    "call_center_tier_view",
    "call_flow_all",
    "call_flow_context",
    "call_flow_delete",
    "call_flow_destinations",
    "call_flow_edit",
    "call_forward",
    "call_forward_all",
    "call_recording_add",
    "call_recording_all",
    "call_recording_delete",
    "call_recording_download",
    "call_recording_edit",
    "call_recording_play",
    "call_recording_transcribe",
    "call_recording_view",
    "click_to_call_call",
    "click_to_call_view",
    "conference_account_code",
    "conference_all",
    "conference_center_all",
    "conference_center_destinations",
    "conference_context",
    "conference_control_delete",
    "conference_control_detail_add",
    "conference_control_detail_delete",
    "conference_control_detail_edit",
    "conference_control_detail_view",
    "conference_control_edit",
    "conference_delete",
    "conference_destinations",
    "conference_edit",
    "conference_email_address",
    "conference_interactive_deaf",
    "conference_interactive_energy",
    "conference_interactive_gain",
    "conference_interactive_kick",
    "conference_interactive_lock",
    "conference_interactive_mute",
    "conference_interactive_video",
    "conference_interactive_view",
    "conference_interactive_volume",
    "conference_profile_delete",
    "conference_profile_edit",
    "conference_profile_param_add",
    "conference_profile_param_delete",
    "conference_profile_param_edit",
    "conference_profile_param_view",
    "conference_room_account_code",
    "conference_room_announce_count",
    "conference_room_announce_name",
    "conference_room_announce_recording",
    "conference_room_email_address",
    "conference_room_enabled",
    "conference_room_max_members",
    "conference_room_moderator_endconf",
    "conference_room_mute",
    "conference_room_profile",
    "conference_room_record",
    "conference_room_schedule",
    "conference_room_sounds",
    "conference_room_view_all",
    "conference_room_wait_mod",
    "conference_session_detail_add",
    "conference_session_detail_delete",
    "conference_session_detail_edit",
    "conference_session_detail_view",
    "conference_session_details",
    "conference_session_edit",
    "conference_session_play",
    "conference_user_add",
    "conference_user_delete",
    "conference_user_edit",
    "conference_user_view",
    "contact_address_add",
    "contact_address_delete",
    "contact_address_edit",
    "contact_address_view",
    "contact_all",
    "contact_attachment_add",
    "contact_attachment_delete",
    "contact_attachment_edit",
    "contact_attachment_view",
    "contact_domain_view",
    "contact_email_add",
    "contact_email_delete",
    "contact_email_edit",
    "contact_email_view",
    "contact_extension_view",
    "contact_group_add",
    "contact_group_delete",
    "contact_group_edit",
    "contact_group_view",
    "contact_note_add",
    "contact_note_delete",
    "contact_note_edit",
    "contact_note_view",
    "contact_phone_add",
    "contact_phone_delete",
    "contact_phone_edit",
    "contact_phone_view",
    "contact_relation_add",
    "contact_relation_delete",
    "contact_relation_edit",
    "contact_relation_view",
    "contact_setting_add",
    "contact_setting_delete",
    "contact_setting_edit",
    "contact_setting_view",
    "contact_time_add",
    "contact_time_delete",
    "contact_time_edit",
    "contact_time_view",
    "contact_upload",
    "contact_url_add",
    "contact_url_delete",
    "contact_url_edit",
    "contact_url_view",
    "contact_user_add",
    "contact_user_delete",
    "contact_user_edit",
    "contact_user_view",
    "dashboard_add",
    "dashboard_all",
    "dashboard_delete",
    "dashboard_domain",
    "dashboard_edit",
    "dashboard_view",
    "dashboard_widget_add",
    "dashboard_widget_delete",
    "dashboard_widget_edit",
    "dashboard_widget_group_add",
    "dashboard_widget_group_delete",
    "dashboard_widget_group_edit",
    "dashboard_widget_group_view",
    "dashboard_widget_parent_uuid",
    "dashboard_widget_view",
    "database_add",
    "database_delete",
    "database_edit",
    "database_transaction_add",
    "database_transaction_delete",
    "database_transaction_edit",
    "database_transaction_view",
    "database_view",
    "default_setting_profile",
    "destination_accountcode",
    "destination_all",
    "destination_area_code",
    "destination_caller_id_name",
    "destination_caller_id_number",
    "destination_cid_name_prefix",
    "destination_condition_field",
    "destination_conditions",
    "destination_context",
    "destination_delete",
    "destination_destinations",
    "destination_distinctive_ring",
    "destination_domain",
    "destination_edit",
    "destination_email",
    "destination_emergency",
    "destination_export",
    "destination_fax",
    "destination_hold_music",
    "destination_import",
    "destination_local",
    "destination_map",
    "destination_number",
    "destination_prefix",
    "destination_record",
    "destination_ringback",
    "destination_trunk_prefix",
    "destination_upload",
    "destination_user_uuid",
    "device_address",
    "device_address_uuid",
    "device_all",
    "device_alternate",
    "device_description",
    "device_domain",
    "device_domain_all",
    "device_enable",
    "device_export",
    "device_extension_add",
    "device_extension_delete",
    "device_extension_edit",
    "device_extension_view",
    "device_files",
    "device_firmware",
    "device_import",
    "device_key_add",
    "device_key_delete",
    "device_key_edit",
    "device_key_extension",
    "device_key_icon",
    "device_key_id",
    "device_key_line",
    "device_key_protected",
    "device_key_view",
    "device_label",
    "device_line_add",
    "device_line_auth_id",
    "device_line_delete",
    "device_line_display_name",
    "device_line_edit",
    "device_line_label",
    "device_line_outbound_proxy_primary",
    "device_line_outbound_proxy_secondary",
    "device_line_password",
    "device_line_port",
    "device_line_register_expires",
    "device_line_server_address",
    "device_line_server_address_primary",
    "device_line_server_address_secondary",
    "device_line_shared",
    "device_line_transport",
    "device_line_view",
    "device_location",
    "device_model",
    "device_profile_add",
    "device_profile_all",
    "device_profile_delete",
    "device_profile_domain",
    "device_profile_edit",
    "device_profile_key_add",
    "device_profile_key_all",
    "device_profile_key_delete",
    "device_profile_key_edit",
    "device_profile_key_view",
    "device_profile_select",
    "device_profile_setting_add",
    "device_profile_setting_all",
    "device_profile_setting_delete",
    "device_profile_setting_edit",
    "device_profile_setting_view",
    "device_profile_view",
    "device_serial_number",
    "device_setting_add",
    "device_setting_delete",
    "device_setting_edit",
    "device_setting_view",
    "device_template",
    "device_user",
    "device_username_password",
    "device_vendor",
    "device_vendor_add",
    "device_vendor_delete",
    "device_vendor_edit",
    "device_vendor_function_add",
    "device_vendor_function_delete",
    "device_vendor_function_edit",
    "device_vendor_function_group_add",
    "device_vendor_function_group_delete",
    "device_vendor_function_group_edit",
    "device_vendor_function_group_view",
    "device_vendor_function_view",
    "device_vendor_restore",
    "device_vendor_view",
    "dialplan_all",
    "dialplan_context",
    "dialplan_destinations",
    "dialplan_detail_add",
    "dialplan_detail_delete",
    "dialplan_detail_edit",
    "dialplan_detail_view",
    "dialplan_domain",
    "dialplan_global",
    "dialplan_xml",
    "do_not_disturb",
    "domain_profile",
    "domain_select",
    "effective_caller_id_name",
    "effective_caller_id_number",
    "email_queue_all",
    "email_queue_attachment_add",
    "email_queue_attachment_all",
    "email_queue_attachment_delete",
    "email_queue_attachment_edit",
    "email_queue_attachment_view",
    "email_template_all",
    "email_template_edit",
    "emergency_caller_id_name",
    "emergency_caller_id_number",
    "emergency_caller_id_select",
    "emergency_caller_id_select_empty",
    "emergency_logs_view_all",
    "event_add_schedule",
    "event_all",
    "event_api",
    "event_background_job",
    "event_call_detail",
    "event_call_secure",
    "event_call_setup_req",
    "event_call_setup_result",
    "event_call_update",
    "event_channel_answer",
    "event_channel_application",
    "event_channel_bridge",
    "event_channel_callstate",
    "event_channel_create",
    "event_channel_data",
    "event_channel_destroy",
    "event_channel_execute",
    "event_channel_execute_complete",
    "event_channel_hangup",
    "event_channel_hangup_complete",
    "event_channel_hold",
    "event_channel_originate",
    "event_channel_outgoing",
    "event_channel_park",
    "event_channel_progress",
    "event_channel_progress_media",
    "event_channel_state",
    "event_channel_unbridge",
    "event_channel_unhold",
    "event_channel_unpark",
    "event_channel_uuid",
    "event_client_disconnected",
    "event_clone",
    "event_codec",
    "event_command",
    "event_conference_data",
    "event_conference_data_query",
    "event_custom",
    "event_del_schedule",
    "event_detected_speech",
    "event_detected_tone",
    "event_device_state",
    "event_dtmf",
    "event_exe_schedule",
    "event_failure",
    "event_general",
    "event_guard_log_add",
    "event_guard_log_all",
    "event_guard_log_delete",
    "event_guard_log_edit",
    "event_guard_log_unblock",
    "event_guard_log_view",
    "event_heartbeat",
    "event_inbound_chan",
    "event_log",
    "event_media_bug_start",
    "event_media_bug_stop",
    "event_message",
    "event_message_query",
    "event_message_waiting",
    "event_module_load",
    "event_module_unload",
    "event_nat",
    "event_notalk",
    "event_notify",
    "event_notify_in",
    "event_outbound_chan",
    "event_phone_feature",
    "event_phone_feature_subscribe",
    "event_playback_start",
    "event_playback_stop",
    "event_presence_in",
    "event_presence_out",
    "event_presence_probe",
    "event_private_command",
    "event_publish",
    "event_re_schedule",
    "event_record_start",
    "event_record_stop",
    "event_recv_info",
    "event_recv_message",
    "event_recv_rtcp_message",
    "event_reloadxml",
    "event_request_params",
    "event_roster",
    "event_send_info",
    "event_send_message",
    "event_send_rtcp_message",
    "event_server_disconnected",
    "event_session_crash",
    "event_session_heartbeat",
    "event_shutdown",
    "event_shutdown_requested",
    "event_socket_data",
    "event_startup",
    "event_talk",
    "event_text",
    "event_trap",
    "event_unpublish",
    "extension_absolute_codec_string",
    "extension_accountcode",
    "extension_advanced",
    "extension_all",
    "extension_call_group",
    "extension_call_screen",
    "extension_caller_id",
    "extension_cidr",
    "extension_copy",
    "extension_delete",
    "extension_destinations",
    "extension_device_address",
    "extension_device_template",
    "extension_dial_string",
    "extension_directory",
    "extension_domain",
    "extension_edit",
    "extension_enabled",
    "extension_export",
    "extension_extension",
    "extension_force_ping",
    "extension_hold_music",
    "extension_import",
    "extension_language",
    "extension_limit",
    "extension_max_registrations",
    "extension_missed_call",
    "extension_nibble_account",
    "extension_password",
    "extension_registered",
    "extension_setting_add",
    "extension_setting_all",
    "extension_setting_delete",
    "extension_setting_edit",
    "extension_setting_view",
    "extension_toll",
    "extension_type",
    "extension_user_add",
    "extension_user_context",
    "extension_user_delete",
    "extension_user_edit",
    "extension_user_record",
    "extension_user_view",
    "fax_accountcode",
    "fax_active_all",
    "fax_active_view",
    "fax_caller_id_name",
    "fax_caller_id_number",
    "fax_delete",
    "fax_destination_number",
    "fax_destinations",
    "fax_download_view",
    "fax_edit",
    "fax_email",
    "fax_email_confirmation",
    "fax_extension",
    "fax_extension_add",
    "fax_extension_advanced",
    "fax_extension_copy",
    "fax_extension_delete",
    "fax_extension_edit",
    "fax_extension_view",
    "fax_extension_view_all",
    "fax_extension_view_domain",
    "fax_file_add",
    "fax_file_delete",
    "fax_file_edit",
    "fax_file_view",
    "fax_footer",
    "fax_forward_number",
    "fax_inbox_delete",
    "fax_inbox_view",
    "fax_log_delete",
    "fax_log_view",
    "fax_message",
    "fax_prefix",
    "fax_queue_all",
    "fax_queue_delete",
    "fax_queue_domain",
    "fax_queue_edit",
    "fax_send",
    "fax_send_channels",
    "fax_sent_delete",
    "fax_sent_recipient",
    "fax_sent_view",
    "fax_subject",
    "fax_toll_allow",
    "fax_user_add",
    "fax_user_delete",
    "fax_user_edit",
    "fax_user_view",
    "feature_codes_export",
    "feature_codes_raw",
    "feature_codes_view",
    "fifo_all",
    "fifo_delete",
    "fifo_destinations",
    "fifo_edit",
    "fifo_member_all",
    "fifo_member_delete",
    "fifo_member_edit",
    "follow_me",
    "follow_me_cid_name_prefix",
    "follow_me_cid_number_prefix",
    "follow_me_delete",
    "follow_me_destination_add",
    "follow_me_destination_delete",
    "follow_me_destination_edit",
    "follow_me_destination_view",
    "follow_me_edit",
    "follow_me_ignore_busy",
    "follow_me_prompt",
    "gateway_all",
    "gateway_channels",
    "gateway_destinations",
    "gateway_domain",
    "group_all",
    "group_domain",
    "group_global",
    "group_member_add",
    "group_member_delete",
    "group_member_view",
    "group_permission_add",
    "group_permission_delete",
    "group_permissions",
    "inbound_route_add",
    "inbound_route_advanced",
    "inbound_route_copy",
    "inbound_route_delete",
    "inbound_route_edit",
    "inbound_route_view",
    "install",
    "ivr_menu_all",
    "ivr_menu_audio_edit",
    "ivr_menu_context",
    "ivr_menu_destinations",
    "ivr_menu_domain",
    "ivr_menu_option_add",
    "ivr_menu_option_delete",
    "ivr_menu_option_edit",
    "ivr_menu_option_view",
    "ivr_menus_other_destinations",
    "ivr_menus_sub_destinations",
    "language_destinations",
    "log_download",
    "log_path_view",
    "log_view",
    "menu_add",
    "menu_delete",
    "menu_edit",
    "menu_item_add",
    "menu_item_delete",
    "menu_item_edit",
    "menu_item_group_add",
    "menu_item_group_delete",
    "menu_item_group_edit",
    "menu_item_group_view",
    "menu_item_view",
    "menu_language_add",
    "menu_language_delete",
    "menu_language_edit",
    "menu_language_view",
    "menu_restore",
    "menu_view",
    "module_add",
    "module_delete",
    "module_edit",
    "module_view",
    "music_on_hold_all",
    "music_on_hold_domain",
    "music_on_hold_global",
    "music_on_hold_map",
    "music_on_hold_name",
    "music_on_hold_path",
    "number_alias",
    "number_translation_detail_add",
    "number_translation_detail_all",
    "number_translation_detail_delete",
    "number_translation_detail_edit",
    "number_translation_detail_view",
    "operator_panel_agents",
    "operator_panel_call_details",
    "operator_panel_calls",
    "operator_panel_coach",
    "operator_panel_conferences",
    "operator_panel_eavesdrop",
    "operator_panel_extensions",
    "operator_panel_hangup",
    "operator_panel_manage",
    "operator_panel_on_demand",
    "operator_panel_originate",
    "operator_panel_record",
    "operator_panel_transfer_attended",
    "operator_panel_view",
    "other_destinations",
    "outbound_caller_id_name",
    "outbound_caller_id_number",
    "outbound_caller_id_select",
    "outbound_route_add",
    "outbound_route_any_gateway",
    "outbound_route_copy",
    "outbound_route_delete",
    "outbound_route_edit",
    "outbound_route_pin_database",
    "outbound_route_pin_number",
    "outbound_route_view",
    "permission_add",
    "permission_all",
    "permission_delete",
    "permission_edit",
    "permission_view",
    "phrase_all",
    "phrase_destinations",
    "phrase_domain",
    "phrase_execute",
    "phrase_play",
    "playback_destinations",
    "recording_all",
    "recording_destinations",
    "recording_download",
    "recording_map",
    "recording_password",
    "recording_play",
    "recording_upload",
    "registration_all",
    "registration_domain",
    "registration_reload",
    "ring_group_all",
    "ring_group_call_screen_enabled",
    "ring_group_caller_id_name",
    "ring_group_caller_id_number",
    "ring_group_cid_name_prefix",
    "ring_group_cid_number_prefix",
    "ring_group_context",
    "ring_group_destination_add",
    "ring_group_destination_delete",
    "ring_group_destination_edit",
    "ring_group_destination_view",
    "ring_group_destinations",
    "ring_group_domain",
    "ring_group_forward",
    "ring_group_forward_toll_allow",
    "ring_group_missed_call",
    "ring_group_prompt",
    "ring_group_user_add",
    "ring_group_user_delete",
    "ring_group_user_edit",
    "ring_group_user_view",
    "sip_status_command",
    "sip_status_flush_cache",
    "sip_status_switch_status",
    "sofia_global_setting_add",
    "sofia_global_setting_all",
    "sofia_global_setting_delete",
    "sofia_global_setting_edit",
    "sofia_global_setting_view",
    "software_add",
    "software_delete",
    "software_edit",
    "software_view",
    "stream_all",
    "stream_delete",
    "stream_edit",
    "stream_map",
    "switch_channels",
    "switch_registrations",
    "switch_uptime",
    "switch_version",
    "system_status_sofia_status",
    "system_status_sofia_status_profile",
    "system_view_backup",
    "system_view_cpu",
    "system_view_database",
    "system_view_hdd",
    "system_view_info",
    "system_view_memcache",
    "system_view_network",
    "system_view_ram",
    "system_view_support",
    "time_condition_all",
    "time_condition_context",
    "time_condition_delete",
    "time_condition_destinations",
    "time_condition_domain",
    "time_condition_edit",
    "tone_destinations",
    "upgrade_apps",
    "upgrade_schema",
    "upgrade_source",
    "upgrade_switch",
    "user_all",
    "user_domain",
    "user_group_add",
    "user_group_delete",
    "user_group_edit",
    "user_group_view",
    "user_groups",
    "user_import",
    "user_log_all",
    "user_password",
    "user_setting_add",
    "user_setting_category_edit",
    "user_setting_delete",
    "user_setting_edit",
    "user_status",
    "user_type",
    "view_call_block",
    "view_groups",
    "view_users",
    "voicemail_all",
    "voicemail_delete",
    "voicemail_destination_add",
    "voicemail_destination_delete",
    "voicemail_destination_edit",
    "voicemail_destination_view",
    "voicemail_destinations",
    "voicemail_domain",
    "voicemail_edit",
    "voicemail_export",
    "voicemail_file",
    "voicemail_forward",
    "voicemail_greeting_add",
    "voicemail_greeting_delete",
    "voicemail_greeting_download",
    "voicemail_greeting_edit",
    "voicemail_greeting_play",
    "voicemail_greeting_upload",
    "voicemail_greeting_view",
    "voicemail_import",
    "voicemail_local_after_email",
    "voicemail_message_add",
    "voicemail_message_delete",
    "voicemail_message_transcribe",
    "voicemail_message_view",
    "voicemail_option_add",
    "voicemail_option_delete",
    "voicemail_option_edit",
    "voicemail_option_view",
    "voicemail_recording_instructions",
    "voicemail_recording_options",
    "voicemail_sms_edit",
    "voicemail_transcription_enabled",
    "xml_cdr_account_code",
    "xml_cdr_add",
    "xml_cdr_all",
    "xml_cdr_answered",
    "xml_cdr_answered_inbound",
    "xml_cdr_answered_outbound",
    "xml_cdr_application_log",
    "xml_cdr_archive",
    "xml_cdr_b_leg",
    "xml_cdr_call_center_queues",
    "xml_cdr_call_disposition",
    "xml_cdr_call_log",
    "xml_cdr_call_stats",
    "xml_cdr_caller_destination",
    "xml_cdr_caller_id_name",
    "xml_cdr_caller_id_number",
    "xml_cdr_cc_agent_leg",
    "xml_cdr_cc_side",
    "xml_cdr_channel_data",
    "xml_cdr_codecs",
    "xml_cdr_custom_fields",
    "xml_cdr_delete",
    "xml_cdr_destination",
    "xml_cdr_details",
    "xml_cdr_direction",
    "xml_cdr_domain",
    "xml_cdr_duration",
    "xml_cdr_edit",
    "xml_cdr_enterprise_leg",
    "xml_cdr_export",
    "xml_cdr_export_csv",
    "xml_cdr_export_pdf",
    "xml_cdr_extension",
    "xml_cdr_extension_add",
    "xml_cdr_extension_all",
    "xml_cdr_extension_delete",
    "xml_cdr_extension_edit",
    "xml_cdr_extension_summary_all",
    "xml_cdr_extension_view",
    "xml_cdr_flow_delete",
    "xml_cdr_flow_view",
    "xml_cdr_hangup_cause",
    "xml_cdr_json_delete",
    "xml_cdr_json_view",
    "xml_cdr_log_delete",
    "xml_cdr_log_view",
    "xml_cdr_lose_race",
    "xml_cdr_mos",
    "xml_cdr_pdd",
    "xml_cdr_recording",
    "xml_cdr_recording_download",
    "xml_cdr_recording_play",
    "xml_cdr_search",
    "xml_cdr_search_advanced",
    "xml_cdr_search_call_center_queues",
    "xml_cdr_search_caller_destination",
    "xml_cdr_search_caller_id",
    "xml_cdr_search_cdr_uuid",
    "xml_cdr_search_destination",
    "xml_cdr_search_direction",
    "xml_cdr_search_duration",
    "xml_cdr_search_extension",
    "xml_cdr_search_hangup_cause",
    "xml_cdr_search_ivr_menus",
    "xml_cdr_search_order",
    "xml_cdr_search_recording",
    "xml_cdr_search_ring_groups",
    "xml_cdr_search_start_range",
    "xml_cdr_search_status",
    "xml_cdr_search_tta",
    "xml_cdr_search_wait",
    "xml_cdr_start",
    "xml_cdr_statistics",
    "xml_cdr_status",
    "xml_cdr_transcript_delete",
    "xml_cdr_transcript_view",
    "xml_cdr_tta",
    "xml_cdr_variables",
    "xml_cdr_view",
    "xml_cdr_wait",
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
