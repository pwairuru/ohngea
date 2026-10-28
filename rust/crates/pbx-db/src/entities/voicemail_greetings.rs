//! `voicemail_greetings` — per-mailbox greetings.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemail_greetings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_greeting_uuid: String,
    pub domain_id: String,
    pub voicemail_uuid: String,
    pub greeting_name: String,
    pub greeting_text: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
