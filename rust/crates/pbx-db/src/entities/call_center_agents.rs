//! `call_center_agents` — PHP v_call_center_agents parity.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_center_agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_center_agent_uuid: String,
    pub domain_id: Uuid,
    pub queue_id: String,
    pub user_uuid: Option<String>,
    pub agent_name: String,
    pub agent_type: Option<String>,
    pub agent_call_timeout: Option<i32>,
    pub agent_id: Option<String>,
    pub agent_password: Option<String>,
    pub agent_contact: String,
    pub agent_status: String,
    pub agent_logout: Option<String>,
    pub agent_max_no_answer: Option<i32>,
    pub agent_wrap_up_time: Option<i32>,
    pub agent_reject_delay_time: Option<i32>,
    pub agent_busy_delay_time: Option<i32>,
    pub agent_no_answer_delay_time: Option<i32>,
    pub agent_record: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
