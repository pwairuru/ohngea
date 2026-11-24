//! `contact_attachments` — v_contact_attachments (file attachments on a contact).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_attachments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_attachment_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub attachment_primary: Option<bool>,
    pub attachment_filename: Option<String>,
    pub attachment_content: Option<String>,
    pub attachment_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
