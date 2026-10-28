//! `voicemails` + `voicemail_greetings` + `voicemail_messages` — app/voicemails/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemails")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_uuid: String,
    pub domain_id: String,
    pub voicemail_id: String,
    pub voicemail_password: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
