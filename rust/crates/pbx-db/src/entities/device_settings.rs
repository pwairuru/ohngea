//! `device_settings` entity — exact columns from app/devices/models.py:70-87.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_setting_uuid: Uuid,
    pub domain_id: Uuid,
    pub device_uuid: Uuid,
    pub device_profile_uuid: Option<Uuid>,
    pub device_setting_category: Option<String>,
    pub device_setting_subcategory: Option<String>,
    pub device_setting_name: String,
    pub device_setting_value: Option<String>,
    pub device_setting_enabled: bool,
    pub device_setting_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
