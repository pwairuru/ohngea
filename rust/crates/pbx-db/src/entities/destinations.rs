//! `destinations` — columns from app/destinations/models.py
//! (core DDL columns from alembic 011 + legacy alias columns kept nullable).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "destinations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub destination_uuid: String,
    pub domain_id: Uuid,
    pub destination_type: String,
    pub destination_number: String,
    pub destination_prefix: Option<String>,
    pub destination_enabled: bool,
    pub destination_description: Option<String>,
    pub destination_context: Option<String>,
    pub destination_app: Option<String>,
    pub destination_data: Option<String>,
    pub destination_caller_id_name: Option<String>,
    pub destination_caller_id_number: Option<String>,
    pub destination_hold_music: Option<String>,
    pub destination_record: Option<bool>,
    pub dialplan_uuid: Option<String>,
    pub fax_uuid: Option<String>,
    pub user_uuid: Option<String>,
    pub group_uuid: Option<String>,
    pub provider_uuid: Option<String>,
    pub destination_trunk_prefix: Option<String>,
    pub trunk_prefix: Option<String>,
    pub destination_area_code: Option<String>,
    pub area_code: Option<String>,
    pub destination_condition_field: Option<String>,
    pub condition_field: Option<String>,
    pub destination_number_regex: Option<String>,
    pub number_regex: Option<String>,
    pub destination_distinctive_ring: Option<String>,
    pub distinctive_ring: Option<String>,
    pub destination_ringback: Option<String>,
    pub ringback: Option<String>,
    pub destination_accountcode: Option<String>,
    pub accountcode: Option<String>,
    pub destination_type_voice: Option<bool>,
    pub type_voice: Option<bool>,
    pub destination_type_fax: Option<bool>,
    pub type_fax: Option<bool>,
    pub destination_type_emergency: Option<bool>,
    pub type_emergency: Option<bool>,
    pub destination_type_text: Option<bool>,
    pub type_text: Option<bool>,
    pub destination_conditions: Option<String>,
    pub conditions: Option<String>,
    pub destination_actions: Option<String>,
    pub actions: Option<String>,
    pub destination_alternate_app: Option<String>,
    pub alternate_app: Option<String>,
    pub destination_alternate_data: Option<String>,
    pub alternate_data: Option<String>,
    pub destination_caller_id: Option<String>,
    pub destination_insert_user: Option<String>,
    pub insert_user: Option<String>,
    pub destination_insert_date: Option<DateTime<Utc>>,
    pub destination_update_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
