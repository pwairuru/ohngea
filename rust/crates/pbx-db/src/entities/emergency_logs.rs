//! `emergency_logs` — columns from app/emergency_logs/models.py (alembic 015).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "emergency_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub emergency_log_uuid: String,
    pub domain_id: Uuid,
    pub extension: Option<String>,
    pub event: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
