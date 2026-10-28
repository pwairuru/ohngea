//! `streams` — columns from app/streams/models.py (alembic 012).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "streams")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub stream_uuid: String,
    pub domain_id: Uuid,
    pub stream_name: String,
    pub stream_location: Option<String>,
    pub stream_enabled: bool,
    pub stream_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
