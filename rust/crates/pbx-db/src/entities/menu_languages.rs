//! `menu_languages` — v_menu_languages (per-language menu titles).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "menu_languages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_language_uuid: Uuid,
    pub menu_uuid: Option<Uuid>,
    pub menu_item_uuid: Option<Uuid>,
    pub menu_language: Option<String>,
    pub menu_item_title: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
