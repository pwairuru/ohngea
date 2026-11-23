//! `ivr_menus` + `ivr_menu_options` — app/ivr_menus/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ivr_menus")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ivr_menu_uuid: String,
    pub domain_id: Option<Uuid>,
    pub name: String,
    pub extension: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub greet_long: Option<String>,
    pub greet_short: Option<String>,
    pub invalid_sound: Option<String>,
    pub exit_sound: Option<String>,
    pub confirm_macro: Option<String>,
    pub confirm_key: Option<String>,
    pub tts_engine: Option<String>,
    pub tts_voice: Option<String>,
    pub confirm_attempts: Option<i32>,
    pub timeout: Option<i32>,
    pub inter_digit_timeout: Option<i32>,
    pub max_failures: Option<i32>,
    pub max_timeouts: Option<i32>,
    pub digit_len: Option<i32>,
    pub dialplan_uuid: Option<String>,
    pub parent_uuid: Option<String>,
    pub language: Option<String>,
    pub dialect: Option<String>,
    pub voice: Option<String>,
    pub pin_number: Option<String>,
    pub ringback: Option<String>,
    pub cid_prefix: Option<String>,
    pub context: Option<String>,
    pub direct_dial: Option<bool>,
    pub exit_app: Option<String>,
    pub exit_data: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
