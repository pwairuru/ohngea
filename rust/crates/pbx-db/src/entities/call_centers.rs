//! `call_centers` (queues) + agents + tiers — app/call_centers/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_centers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_center_queue_uuid: String,
    pub domain_id: String,
    pub queue_name: String,
    pub queue_extension: Option<String>,
    pub queue_strategy: String,
    pub queue_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
