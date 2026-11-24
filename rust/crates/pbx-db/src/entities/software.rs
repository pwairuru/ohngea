//! `software` — v_software version registry.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "software")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub software_uuid: Uuid,
    pub software_name: Option<String>,
    pub software_url: Option<String>,
    pub software_version: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
