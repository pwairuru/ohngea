//! `phrases` + `phrase_details` — columns from app/phrases/models.py (alembic 012).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "phrases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub phrase_uuid: String,
    pub domain_id: Uuid,
    pub phrase_name: String,
    pub phrase_language: Option<String>,
    pub phrase_enabled: bool,
    pub phrase_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
