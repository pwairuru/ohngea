//! `contact_notes` — app/contacts/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_notes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_note_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub contact_note: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
