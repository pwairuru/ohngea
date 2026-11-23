//! `voicemail_messages` — stored messages (metadata only here).

use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemail_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_message_uuid: Uuid,
    pub domain_id: Uuid,
    pub voicemail_uuid: Uuid,
    pub file_name: Option<String>,
    pub message_length: Option<i32>,
    // PHP v_voicemail_messages parity
    pub created_epoch: Option<i64>,
    pub read_epoch: Option<i64>,
    pub caller_id_name: Option<String>,
    pub caller_id_number: Option<String>,
    pub message_status: Option<String>,
    pub message_priority: Option<String>,
    pub message_intro_base64: Option<String>,
    pub message_base64: Option<String>,
    pub message_transcription: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
