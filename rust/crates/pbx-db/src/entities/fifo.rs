//! `fifo` + `fifo_members` — columns from app/fifo/models.py
//! (alembic 013 minimal core + model-superset nullable columns; members 038).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fifo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fifo_uuid: String,
    pub domain_id: Uuid,
    pub dialplan_uuid: Option<String>,
    pub fifo_name: String,
    pub fifo_extension: Option<String>,
    pub fifo_agent_queue: Option<String>,
    pub fifo_agent_status: Option<String>,
    pub fifo_enabled: bool,
    pub fifo_description: Option<String>,
    pub fifo_strategy: Option<String>,
    pub fifo_members: Option<String>,
    pub fifo_timeout_seconds: Option<String>,
    pub fifo_exit_key: Option<String>,
    pub fifo_exit_action: Option<String>,
    pub fifo_music: Option<String>,
    pub fifo_order: Option<i32>,
    pub fifo_caller_id_name: Option<String>,
    pub fifo_caller_id_number: Option<String>,
    pub insert_user: Option<String>,
    pub update_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
