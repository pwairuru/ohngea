//! `number_translations` + `number_translation_details` — columns from
//! app/number_translations/models.py (alembic 013; regex/replace nullable
//! per the Python model so compat inserts never fail).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "number_translations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub number_translation_uuid: String,
    pub domain_id: Uuid,
    pub number_translation_name: String,
    pub number_translation_regex: Option<String>,
    pub number_translation_replace: Option<String>,
    pub number_translation_enabled: bool,
    pub number_translation_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
