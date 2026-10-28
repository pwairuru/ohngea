//! `domain_settings` entity — exact columns from app/domains/models.py.
//! Per-domain overrides; win over `default_settings` in [`crate::seeds::resolve_setting`].

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "domain_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub domain_id: Uuid,
    pub category: String,
    pub subcategory: String,
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
