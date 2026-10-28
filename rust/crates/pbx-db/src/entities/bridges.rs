//! `bridges` — columns from app/bridges/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bridges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bridge_uuid: String,
    pub domain_id: Uuid,
    pub bridge_name: String,
    pub bridge_destination: Option<String>,
    pub bridge_enabled: bool,
    pub bridge_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
