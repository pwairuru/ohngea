//! `ivr_menus` + `ivr_menu_options` — app/ivr_menus/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ivr_menus")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ivr_menu_uuid: String,
    pub domain_id: Uuid,
    pub name: String,
    pub extension: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
