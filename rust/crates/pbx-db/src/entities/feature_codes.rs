//! `feature_codes` — columns from app/feature_codes/models.py (alembic 016).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "feature_codes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub feature_code_uuid: String,
    pub domain_id: Uuid,
    pub feature_code_name: String,
    pub feature_code_value: String,
    pub feature_code_enabled: bool,
    pub feature_code_description: Option<String>,
    pub insert_user: Option<String>,
    pub update_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
