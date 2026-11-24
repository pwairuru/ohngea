//! `menu_item_groups` — v_menu_item_groups (menu item ↔ group links).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "menu_item_groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_item_group_uuid: Uuid,
    pub menu_uuid: Option<Uuid>,
    pub menu_item_uuid: Option<Uuid>,
    pub group_name: Option<String>,
    pub group_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
