//! `ring_groups` + `ring_group_members` — columns from app/ring_groups/models.py.
//! `domain_id` is Uuid (unified RULING; test opaque strings map via UUIDv5 in routing_db::domain),
//! UUIDs stringify in prod). Members mirror RingGroupMember destinations.
//!
//! `domain_id` stays NON-nullable by design: PHP `ring_groups.php` has no
//! NULL arm (domain branch is strict `=`, `ring_group_domain` in the edit
//! page is commented out). Cross-tenant visibility comes from
//! `ring_group_users` membership (tiered list), not shared rows.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ring_groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ring_group_uuid: String,
    pub domain_id: Uuid,
    pub name: String,
    pub extension: String,
    pub strategy: String,
    pub enabled: bool,
    pub description: Option<String>,
    // PHP v_ring_groups full parity (ring_group_ prefix stripped)
    pub greeting: Option<String>,
    pub exit_key: Option<String>,
    pub call_timeout: Option<i32>,
    pub forward_destination: Option<String>,
    pub forward_enabled: Option<bool>,
    pub caller_id_name: Option<String>,
    pub caller_id_number: Option<String>,
    pub cid_name_prefix: Option<String>,
    pub cid_number_prefix: Option<String>,
    pub timeout_app: Option<String>,
    pub timeout_data: Option<String>,
    pub distinctive_ring: Option<String>,
    pub ringback: Option<String>,
    pub call_screen_enabled: Option<bool>,
    pub call_forward_enabled: Option<bool>,
    pub follow_me_enabled: Option<bool>,
    pub missed_call_app: Option<String>,
    pub missed_call_data: Option<String>,
    pub context: Option<String>,
    pub dialplan_uuid: Option<String>,
    pub forward_toll_allow: Option<String>,
    pub wait_announcement: Option<String>,
    pub wait_interval: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
