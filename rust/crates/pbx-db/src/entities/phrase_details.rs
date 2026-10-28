//! `phrase_details` — columns from app/phrases/models.py (alembic 012 subset +
//! model-superset group/tag/pattern/method/type columns, all nullable).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "phrase_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub phrase_detail_uuid: String,
    pub domain_id: Uuid,
    pub phrase_uuid: String,
    pub phrase_detail_group: Option<i32>,
    pub phrase_detail_tag: Option<String>,
    pub phrase_detail_pattern: Option<String>,
    pub phrase_detail_function: Option<String>,
    pub phrase_detail_data: Option<String>,
    pub phrase_detail_method: Option<String>,
    pub phrase_detail_type: Option<String>,
    pub phrase_detail_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
