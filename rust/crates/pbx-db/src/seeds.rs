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
    default_settings, dialplan_details, dialplans, domain_settings, domains, sip_profile_domains,
    sip_profile_settings, sip_profiles, sofia_global_settings, vars,
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

fn parse_uuid(s: &str) -> Result<Uuid, DbErr> {
    Uuid::parse_str(s).map_err(|e| DbErr::Custom(format!("bad seed uuid {s}: {e}")))
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
        vars::Entity::insert(vars::ActiveModel {
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
        .await?;
        n += 1;
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
        sofia_global_settings::Entity::insert(sofia_global_settings::ActiveModel {
            sofia_global_setting_uuid: Set(parse_uuid(s.uuid)?),
            global_setting_name: Set(s.name.to_owned()),
            global_setting_value: Set(s.value.to_owned()),
            global_setting_enabled: Set(s.enabled),
            global_setting_description: Set(s.description.map(str::to_owned)),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(db)
        .await?;
        n += 1;
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
        domain_settings::Entity::insert(domain_settings::ActiveModel {
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
        .await?;
        n += 1;
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
        default_settings::Entity::insert(default_settings::ActiveModel {
            default_setting_uuid: Set(uuid),
            default_setting_category: Set(s.category.to_owned()),
            default_setting_subcategory: Set(s.subcategory.to_owned()),
            default_setting_name: Set(s.name.to_owned()),
            default_setting_value: Set(s.value.map(str::to_owned)),
            default_setting_enabled: Set(s.enabled),
            default_setting_description: Set(s.description.map(str::to_owned)),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(db)
        .await?;
        n += 1;
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
    domains::Entity::insert(domains::ActiveModel {
        domain_id: Set(domain_id),
        domain_name: Set(domain_name.to_owned()),
        enabled: Set(true),
        description: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .exec(db)
    .await?;
    Ok(1)
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
        dialplans::Entity::insert(dialplans::ActiveModel {
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
            dialplan_details::Entity::insert(dialplan_details::ActiveModel {
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
            .await?;
            n += 1;
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
        sip_profiles::Entity::insert(sip_profiles::ActiveModel {
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
            sip_profile_domains::Entity::insert(sip_profile_domains::ActiveModel {
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
            .await?;
            n += 1;
        }
        for s in p.settings {
            let snow = Utc::now();
            sip_profile_settings::Entity::insert(sip_profile_settings::ActiveModel {
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
            .await?;
            n += 1;
        }
    }
    Ok(n)
}

/// Idempotent boot seed: fills empty tables only (row-count guarded).
/// Ensures the tenant `domains` row first (FK parent), then seeds each
/// empty store: vars + domain_settings + dialplans + sip_profiles
/// (per-domain), sofia + default_settings (global). Returns rows inserted.
/// Safe to call on every boot and concurrently (unique guards dedupe).
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
