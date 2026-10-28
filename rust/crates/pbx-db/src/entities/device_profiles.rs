//! `device_profiles` entity — exact columns from app/devices/models.py:115-127.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_profile_uuid: Uuid,
    pub domain_id: Uuid,
    pub device_profile_name: String,
    pub device_profile_enabled: bool,
    pub device_profile_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
