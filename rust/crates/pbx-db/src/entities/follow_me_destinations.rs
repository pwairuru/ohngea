//! `follow_me_destinations` — columns from app/follow_me/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "follow_me_destinations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub follow_me_destination_uuid: String,
    pub domain_id: Uuid,
    pub follow_me_uuid: String,
    pub follow_me_destination: String,
    pub follow_me_delay: i32,
    pub follow_me_timeout: i32,
    pub follow_me_prompt: Option<String>,
    pub follow_me_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
