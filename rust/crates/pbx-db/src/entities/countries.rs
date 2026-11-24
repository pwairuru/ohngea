//! `countries` — v_countries reference data (seeded from PHP defaults).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "countries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub country_uuid: Uuid,
    pub country: String,
    pub iso_a2: Option<String>,
    pub iso_a3: Option<String>,
    pub num: Option<i32>,
    pub country_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
