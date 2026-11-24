//! `languages` — v_languages reference data (seeded from PHP defaults).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "languages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub language_uuid: Uuid,
    pub language: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
