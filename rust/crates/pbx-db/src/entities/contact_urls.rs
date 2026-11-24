//! `contact_urls` — app/contacts/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_urls")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_url_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub url_label: Option<String>,
    pub url_address: String,
    pub url_type: Option<String>,
    pub url_primary: bool,
    pub url_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
