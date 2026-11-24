//! `extension_settings` — columns from app/extension_settings/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "extension_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub extension_setting_uuid: Uuid,
    pub domain_id: Uuid,
    pub extension_uuid: Uuid,
    pub extension_setting_type: Option<String>,
    pub extension_setting_name: String,
    pub extension_setting_value: String,
    pub extension_setting_enabled: bool,
    pub extension_setting_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
