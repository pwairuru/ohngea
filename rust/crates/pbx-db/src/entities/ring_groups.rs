//! `ring_groups` + `ring_group_members` — columns from app/ring_groups/models.py.
//! `domain_id` is String (tenant claims carry opaque strings in tests,
//! UUIDs stringify in prod). Members mirror RingGroupMember destinations.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ring_groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ring_group_uuid: String,
    pub domain_id: String,
    pub name: String,
    pub extension: String,
    pub strategy: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
