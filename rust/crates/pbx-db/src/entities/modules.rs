//! `modules` — v_modules FreeSWITCH module registry.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "modules")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub module_uuid: String,
    pub module_label: Option<String>,
    pub module_name: Option<String>,
    pub module_category: Option<String>,
    pub module_order: Option<i32>,
    pub module_enabled: Option<bool>,
    pub module_default_enabled: Option<bool>,
    pub module_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
