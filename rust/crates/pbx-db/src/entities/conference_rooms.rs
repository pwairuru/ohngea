//! `conference_rooms` — PHP v_conference_rooms parity.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_rooms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_room_uuid: String,
    pub domain_id: Uuid,
    pub conference_center_uuid: String,
    pub conference_room_name: String,
    pub max_members: Option<i32>,
    pub enabled: bool,
    pub profile: Option<String>,
    pub moderator_pin: Option<String>,
    pub participant_pin: Option<String>,
    pub start_datetime: Option<DateTime<Utc>>,
    pub stop_datetime: Option<DateTime<Utc>>,
    pub wait_mod: Option<bool>,
    pub moderator_endconf: Option<bool>,
    pub announce_name: Option<bool>,
    pub announce_count: Option<bool>,
    pub announce_recording: Option<bool>,
    pub sounds: Option<String>,
    pub mute: Option<bool>,
    pub created: Option<String>,
    pub created_by: Option<String>,
    pub email_address: Option<String>,
    pub account_code: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
