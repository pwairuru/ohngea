//! `conference_sessions` — app/conference_centers/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_session_uuid: String,
    pub domain_id: Uuid,
    pub conference_center_uuid: Option<String>,
    pub conference_room_uuid: Option<String>,
    pub profile: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
