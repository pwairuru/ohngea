//! `voicemails` + `voicemail_greetings` + `voicemail_messages` — app/voicemails/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "voicemails")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub voicemail_uuid: String,
    pub domain_id: Option<Uuid>,
    pub voicemail_id: String,
    pub voicemail_password: String,
    pub enabled: bool,
    pub description: Option<String>,
    // PHP v_voicemails parity
    pub voicemail_tutorial: Option<bool>,
    pub greeting_id: Option<String>,
    pub voicemail_alternate_greet_id: Option<String>,
    pub voicemail_recording_instructions: Option<bool>,
    pub voicemail_recording_options: Option<String>,
    pub voicemail_mail_to: Option<String>,
    pub voicemail_sms_to: Option<String>,
    pub voicemail_transcription_enabled: Option<bool>,
    pub voicemail_transcription_prompt_enabled: Option<bool>,
    pub voicemail_transcription_prompt: Option<String>,
    pub voicemail_attach_file: Option<bool>,
    pub voicemail_file: Option<String>,
    pub voicemail_local_after_email: Option<bool>,
    pub voicemail_local_after_forward: Option<bool>,
    pub alternate_voicemail_destination: Option<String>,
    pub alternate_voicemail_enabled: Option<bool>,
    pub voicemail_name_base64: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
