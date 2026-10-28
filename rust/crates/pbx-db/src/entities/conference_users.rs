//! `conference_users` — app/conferences/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_user_uuid: String,
    pub domain_id: Uuid,
    pub conference_uuid: String,
    pub user_uuid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
