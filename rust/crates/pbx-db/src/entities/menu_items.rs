//! `menu_items` — v_menu_items admin menu tree.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "menu_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_item_uuid: Uuid,
    pub menu_uuid: Option<Uuid>,
    pub menu_item_parent_uuid: Option<Uuid>,
    pub item_uuid: Option<String>,
    pub menu_item_title: String,
    pub menu_item_link: Option<String>,
    pub menu_item_icon: Option<String>,
    pub menu_item_icon_color: Option<String>,
    pub menu_item_category: Option<String>,
    pub menu_item_protected: Option<bool>,
    pub menu_item_order: Option<i32>,
    pub menu_item_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
