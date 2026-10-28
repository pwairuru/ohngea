//! `ring_group_members` — destinations of a ring group.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ring_group_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ring_group_member_uuid: String,
    pub ring_group_uuid: String,
    pub domain_id: String,
    pub destination: String,
    pub delay: i32,
    pub timeout: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
