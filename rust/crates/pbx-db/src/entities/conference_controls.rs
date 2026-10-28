//! `conference_controls` — app/conference_controls/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_controls")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_control_uuid: String,
    pub domain_id: Uuid,
    pub control_name: String,
    pub control_enabled: bool,
    pub control_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
