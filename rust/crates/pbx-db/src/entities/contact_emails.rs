//! `contact_emails` — app/contacts/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_emails")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_email_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub email_label: Option<String>,
    pub email_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
