//! `contact_settings` — v_contact_settings (per-contact key/value settings).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_setting_uuid: Uuid,
    pub contact_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_setting_category: Option<String>,
    pub contact_setting_subcategory: Option<String>,
    pub contact_setting_name: Option<String>,
    pub contact_setting_value: Option<String>,
    pub contact_setting_order: Option<i32>,
    pub contact_setting_enabled: Option<bool>,
    pub contact_setting_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
