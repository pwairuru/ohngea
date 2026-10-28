//! `device_profile_keys` entity — exact columns from app/devices/models.py:130-151.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_profile_keys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_profile_key_uuid: Uuid,
    pub domain_id: Uuid,
    pub device_profile_uuid: Uuid,
    pub profile_key_id: Option<i32>,
    pub profile_key_category: Option<String>,
    pub profile_key_vendor: Option<String>,
    pub profile_key_type: Option<String>,
    pub profile_key_subtype: Option<String>,
    pub profile_key_line: Option<i32>,
    pub profile_key_value: Option<String>,
    pub profile_key_extension: Option<String>,
    pub profile_key_protected: bool,
    pub profile_key_label: Option<String>,
    pub profile_key_icon: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
