//! `number_translation_details` — columns from app/number_translations/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "number_translation_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub number_translation_detail_uuid: String,
    pub domain_id: Uuid,
    pub number_translation_uuid: String,
    pub number_translation_detail_regex: String,
    pub number_translation_detail_replace: String,
    pub number_translation_detail_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
