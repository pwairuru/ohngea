//! `contact_relations` — v_contact_relations (contact ↔ contact links).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_relations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_relation_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub relation_label: Option<String>,
    pub relation_contact_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
