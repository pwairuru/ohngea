//! `voicemail_messages` — stored messages (metadata only here).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemail_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_message_uuid: String,
    pub domain_id: String,
    pub voicemail_uuid: String,
    pub file_name: Option<String>,
    pub message_length: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
