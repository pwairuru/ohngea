//! `device_profile_settings` entity — exact columns from app/devices/models.py:154-167.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_profile_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_profile_setting_uuid: Uuid,
    pub domain_id: Uuid,
    pub device_profile_uuid: Uuid,
    pub profile_setting_name: String,
    pub profile_setting_value: Option<String>,
    pub profile_setting_enabled: bool,
    pub profile_setting_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
