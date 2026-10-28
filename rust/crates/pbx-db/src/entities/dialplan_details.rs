//! `dialplan_details` entity — exact columns from app/dialplans/models.py:67-121.
//!
//! The DB column stays `type`; the Rust field is renamed with
//! `column_name` because `type` is a Rust keyword (same trick as Python's
//! `sa_column` for the gateway `register` column).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dialplan_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub detail_uuid: Uuid,
    pub dialplan_uuid: Uuid,
    pub tag: String,
    #[sea_orm(column_name = "type")]
    pub detail_type: String,
    pub data: String,
    pub order: i32,
    pub domain_id: Option<Uuid>,
    pub dialplan_detail_break: Option<String>,
    pub dialplan_detail_inline: Option<String>,
    pub dialplan_detail_group: Option<i32>,
    pub detail_enabled: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
