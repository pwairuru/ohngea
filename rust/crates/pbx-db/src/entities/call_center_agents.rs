//! `call_center_agents` — agent rows belonging to a queue.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_center_agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_center_agent_uuid: String,
    pub domain_id: String,
    pub queue_id: String,
    pub agent_name: String,
    pub agent_contact: String,
    pub agent_status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
