//! `call_flows` — columns from app/call_flows/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_flows")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_flow_uuid: Uuid,
    pub domain_id: Option<Uuid>,
    pub dialplan_uuid: Option<Uuid>,
    pub call_flow_name: String,
    pub call_flow_extension: String,
    pub call_flow_feature_code: Option<String>,
    pub call_flow_context: String,
    pub call_flow_status: Option<String>,
    pub call_flow_pin_number: Option<String>,
    pub call_flow_label: Option<String>,
    pub call_flow_enabled: bool,
    pub call_flow_description: Option<String>,
    pub call_flow_sound: Option<String>,
    pub call_flow_app: Option<String>,
    pub call_flow_data: Option<String>,
    pub call_flow_alternate_label: Option<String>,
    pub call_flow_alternate_destination: Option<String>,
    pub call_flow_alternate_sound: Option<String>,
    pub call_flow_alternate_app: Option<String>,
    pub call_flow_alternate_data: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
