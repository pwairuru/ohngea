//! `sip_profile_settings` entity — exact columns from app/sip_profiles/models.py:61-74.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sip_profile_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sip_profile_setting_uuid: Uuid,
    pub domain_id: Uuid,
    pub sip_profile_uuid: Uuid,
    pub setting_name: String,
    pub setting_value: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
