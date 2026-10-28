//! Idempotent seed loaders mirroring app/seeds/*.py.
//! Dedupe keys match Python: vars → (domain_id, var_name, var_category);
//! sofia → global_setting_name (UNIQUE); default_settings → uuid
//! (ON CONFLICT uuid DO NOTHING); domain_settings → (domain, category, subcategory, key).

use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::{default_settings, domain_settings, sofia_global_settings, vars};

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

include!("seeds_data.rs");

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
