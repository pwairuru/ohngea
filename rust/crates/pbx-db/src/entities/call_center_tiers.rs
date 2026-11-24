//! `call_center_tiers` — agent↔queue tier bindings.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_center_tiers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_center_tier_uuid: String,
    pub domain_id: Uuid,
    pub queue_id: String,
    pub call_center_queue_uuid: Option<String>,
    pub call_center_agent_uuid: Option<String>,
    pub agent_name: String,
    pub queue_name: Option<String>,
    pub tier_level: i32,
    pub tier_position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
