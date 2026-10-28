//! `fifo_members` — columns from app/fifo/models.py (alembic 038).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fifo_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fifo_member_uuid: String,
    pub domain_id: Uuid,
    pub fifo_uuid: String,
    pub member_contact: Option<String>,
    pub member_call_timeout: Option<i32>,
    pub member_simultaneous: Option<i32>,
    pub member_wrap_up_time: Option<i32>,
    pub member_enabled: bool,
    pub insert_user: Option<String>,
    pub update_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
