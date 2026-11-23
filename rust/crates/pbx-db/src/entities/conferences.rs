//! `conferences` — PHP v_conferences parity.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conferences")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_uuid: String,
    pub domain_id: Option<Uuid>,
    pub conference_name: String,
    pub conference_extension: String,
    pub conference_profile: String,
    pub conference_enabled: bool,
    pub dialplan_uuid: Option<String>,
    pub pin_number: Option<String>,
    pub email_address: Option<String>,
    pub account_code: Option<String>,
    pub flags: Option<String>,
    pub conference_order: Option<i32>,
    pub conference_description: Option<String>,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
