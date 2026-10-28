//! `sip_profile_domains` entity — exact columns from app/sip_profiles/models.py:46-58.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sip_profile_domains")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sip_profile_domain_uuid: Uuid,
    pub domain_id: Uuid,
    pub sip_profile_uuid: Uuid,
    pub domain_name: String,
    pub domain_alias: Option<String>,
    pub domain_parse: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
