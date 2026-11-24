//! `call_centers` (queues) + agents + tiers — PHP v_call_center_queues parity.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_centers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_center_queue_uuid: String,
    pub domain_id: Uuid,
    pub queue_name: String,
    pub queue_extension: Option<String>,
    pub queue_strategy: String,
    pub queue_enabled: bool,
    pub dialplan_uuid: Option<String>,
    pub greeting: Option<String>,
    pub moh_sound: Option<String>,
    pub record_template: Option<String>,
    pub language: Option<String>,
    pub dialect: Option<String>,
    pub voice: Option<String>,
    pub queue_limit: Option<i32>,
    pub time_base_score: Option<String>,
    pub time_base_score_sec: Option<i32>,
    pub max_wait_time: Option<i32>,
    pub max_wait_time_with_no_agent: Option<i32>,
    pub max_wait_time_with_no_agent_time_reached: Option<i32>,
    pub tier_rules_apply: Option<bool>,
    pub tier_rule_wait_second: Option<i32>,
    pub tier_rule_no_agent_no_wait: Option<bool>,
    pub timeout_action: Option<String>,
    pub discard_abandoned_after: Option<i32>,
    pub abandoned_resume_allowed: Option<bool>,
    pub tier_rule_wait_multiply_level: Option<bool>,
    pub cid_prefix: Option<String>,
    pub outbound_caller_id_name: Option<String>,
    pub outbound_caller_id_number: Option<String>,
    pub announce_position: Option<bool>,
    pub announce_sound: Option<String>,
    pub announce_frequency: Option<i32>,
    pub cc_exit_keys: Option<String>,
    pub email_address: Option<String>,
    pub context: Option<String>,
    pub queue_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
