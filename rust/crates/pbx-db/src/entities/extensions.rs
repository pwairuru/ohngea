//! `extensions` entity — exact columns from app/extensions/models.py:12-81.
//! Validation (`^\d{2,7}$`) and password/user_context defaults live in
//! [`crate::extensions`] (mirrors `service.py::create_extension`).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "extensions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub extension_uuid: Uuid,
    pub domain_id: Uuid,
    pub extension: String,
    pub number_alias: Option<String>,
    pub password: Option<String>,
    pub accountcode: Option<String>,
    pub effective_caller_id_name: String,
    pub effective_caller_id_number: Option<String>,
    pub outbound_caller_id_name: Option<String>,
    pub outbound_caller_id_number: Option<String>,
    pub emergency_caller_id_name: Option<String>,
    pub emergency_caller_id_number: Option<String>,
    pub directory_first_name: Option<String>,
    pub directory_last_name: Option<String>,
    pub directory_visible: bool,
    pub directory_exten_visible: bool,
    pub max_registrations: Option<String>,
    pub limit_max: Option<String>,
    pub limit_destination: Option<String>,
    pub missed_call_app: Option<String>,
    pub missed_call_data: Option<String>,
    pub user_context: Option<String>,
    pub toll_allow: Option<String>,
    pub call_timeout: Option<i32>,
    pub call_group: Option<String>,
    pub call_screen_enabled: bool,
    pub user_record: Option<String>,
    pub hold_music: Option<String>,
    pub auth_acl: Option<String>,
    pub cidr: Option<String>,
    pub sip_force_contact: Option<String>,
    pub sip_force_expires: Option<i32>,
    pub nibble_account: Option<String>,
    pub mwi_account: Option<String>,
    pub sip_bypass_media: Option<String>,
    pub absolute_codec_string: Option<String>,
    pub force_ping: bool,
    pub dial_string: Option<String>,
    pub do_not_disturb: bool,
    pub forward_all_destination: Option<String>,
    pub forward_all_enabled: bool,
    pub forward_busy_destination: Option<String>,
    pub forward_busy_enabled: bool,
    pub forward_no_answer_destination: Option<String>,
    pub forward_no_answer_enabled: bool,
    pub forward_user_not_registered_destination: Option<String>,
    pub forward_user_not_registered_enabled: bool,
    pub follow_me_uuid: Option<Uuid>,
    pub follow_me_enabled: bool,
    pub unique_id: Option<String>,
    pub dial_user: Option<String>,
    pub dial_domain: Option<String>,
    pub follow_me_destinations: Option<String>,
    pub extension_language: Option<String>,
    pub extension_dialect: Option<String>,
    pub extension_voice: Option<String>,
    pub extension_type: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
