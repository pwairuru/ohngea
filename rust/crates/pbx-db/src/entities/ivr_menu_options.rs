//! `ivr_menu_options` — digits→action rows for a menu.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ivr_menu_options")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub option_uuid: String,
    pub ivr_menu_uuid: String,
    pub domain_id: Uuid,
    pub digits: String,
    pub action: String,
    pub param: Option<String>,
    pub option_order: i32,
    pub ivr_menu_id: Option<String>,
    pub ivr_menu_option_enabled: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
