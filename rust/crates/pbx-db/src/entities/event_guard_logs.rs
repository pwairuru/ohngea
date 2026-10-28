//! `event_guard_logs` — columns from app/event_guard/models.py (alembic 017).
//! Global table (no `domain_id`, no RLS) like the Python ground truth.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_guard_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub event_guard_log_uuid: String,
    pub hostname: Option<String>,
    pub log_date: DateTime<Utc>,
    pub filter: Option<String>,
    pub ip_address: Option<String>,
    pub extension: Option<String>,
    pub user_agent: Option<String>,
    pub log_status: Option<String>,
    pub insert_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
