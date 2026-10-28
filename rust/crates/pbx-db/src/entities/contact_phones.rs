//! `contact_phones` — app/contacts/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_phones")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_phone_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub phone_label: Option<String>,
    pub phone_number: String,
    pub phone_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
