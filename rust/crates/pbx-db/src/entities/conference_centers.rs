//! `conference_centers` — app/conference_centers/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_centers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_center_uuid: String,
    pub domain_id: Uuid,
    pub conference_center_name: String,
    pub conference_center_extension: Option<String>,
    pub conference_center_description: Option<String>,
    pub conference_center_enabled: bool,
    pub dialplan_uuid: Option<String>,
    pub conference_center_pin_length: Option<i32>,
    pub conference_center_greeting: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
