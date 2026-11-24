//! `contact_users` — v_contact_users (contact ↔ user links).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_user_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
