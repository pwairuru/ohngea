//! `device_lines` entity — exact columns from app/devices/models.py:43-67.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_lines")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_line_uuid: Uuid,
    pub domain_id: Uuid,
    pub device_uuid: Uuid,
    pub line_number: String,
    pub server_address: Option<String>,
    pub server_address_primary: Option<String>,
    pub server_address_secondary: Option<String>,
    pub outbound_proxy_primary: Option<String>,
    pub outbound_proxy_secondary: Option<String>,
    pub label: Option<String>,
    pub display_name: Option<String>,
    pub user_id: Option<String>,
    pub auth_id: Option<String>,
    pub password: Option<String>,
    pub sip_port: Option<i32>,
    pub sip_transport: Option<String>,
    pub register_expires: Option<i32>,
    pub shared_line: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
