//! `call_broadcasts` — columns from app/call_broadcast/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_broadcasts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_broadcast_uuid: String,
    pub domain_id: Uuid,
    pub broadcast_name: String,
    pub broadcast_description: Option<String>,
    pub broadcast_start_time: Option<i64>,
    pub broadcast_timeout: Option<i32>,
    pub broadcast_concurrent_limit: Option<i32>,
    pub recording_uuid: Option<String>,
    pub broadcast_caller_id_name: Option<String>,
    pub broadcast_caller_id_number: Option<String>,
    pub broadcast_destination_type: Option<String>,
    pub broadcast_destination_data: Option<String>,
    pub broadcast_phone_numbers: Option<String>,
    pub broadcast_avmd: Option<bool>,
    pub broadcast_accountcode: Option<String>,
    pub broadcast_toll_allow: Option<String>,
    pub broadcast_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
