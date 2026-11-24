//! `contact_addresses` — app/contacts/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_addresses")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_address_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub address_label: Option<String>,
    pub address_street: Option<String>,
    pub address_extended: Option<String>,
    pub address_community: Option<String>,
    pub address_locality: Option<String>,
    pub address_region: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_country: Option<String>,
    pub address_type: Option<String>,
    pub address_primary: bool,
    pub address_latitude: Option<String>,
    pub address_longitude: Option<String>,
    pub address_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
