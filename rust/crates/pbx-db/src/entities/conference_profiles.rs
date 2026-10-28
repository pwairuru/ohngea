//! `conference_profiles` — app/conference_profiles/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_profile_uuid: String,
    pub domain_id: Uuid,
    pub profile_name: String,
    pub profile_enabled: bool,
    pub profile_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
