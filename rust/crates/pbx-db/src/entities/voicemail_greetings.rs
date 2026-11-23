//! `voicemail_greetings` — per-mailbox greetings.

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemail_greetings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_greeting_uuid: Uuid,
    pub domain_id: Uuid,
    pub voicemail_uuid: Uuid,
    pub greeting_name: String,
    pub greeting_text: Option<String>,
    // PHP v_voicemail_greetings parity
    pub greeting_id: Option<String>,
    pub greeting_voice: Option<String>,
    pub greeting_message: Option<String>,
    pub greeting_filename: Option<String>,
    pub greeting_description: Option<String>,
    pub greeting_base64: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
