//! `menus` — v_menus navigation roots.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "menus")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_uuid: Uuid,
    pub menu_name: String,
    pub menu_language: Option<String>,
    pub menu_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
