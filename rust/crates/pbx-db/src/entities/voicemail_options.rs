//! `voicemail_options` — app/voicemails/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemail_options")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_option_uuid: Uuid,
    pub domain_id: Uuid,
    pub voicemail_uuid: Uuid,
    pub digits: Option<String>,
    pub action: Option<String>,
    pub param: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
