//! `follow_me` — columns from app/follow_me/models.py (+024 ignore_busy).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "follow_me")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub follow_me_uuid: String,
    pub domain_id: Uuid,
    pub follow_me_enabled: bool,
    pub cid_name_prefix: Option<String>,
    pub cid_number_prefix: Option<String>,
    pub call_prompt: Option<String>,
    pub dial_string: Option<String>,
    pub description: Option<String>,
    pub follow_me_ignore_busy: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
