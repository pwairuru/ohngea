//! `default_settings` entity — exact columns from app/admin_settings/models.py
//! (alembic 035_provision_defaults). Global defaults, no domain_id.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "default_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub default_setting_uuid: Uuid,
    pub default_setting_category: String,
    pub default_setting_subcategory: String,
    pub default_setting_name: String,
    pub default_setting_value: Option<String>,
    pub default_setting_order: Option<i32>,
    pub default_setting_enabled: bool,
    pub default_setting_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
