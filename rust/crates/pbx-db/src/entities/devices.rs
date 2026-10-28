//! `devices` entity — exact columns from app/devices/models.py:10-40.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_uuid: Uuid,
    pub mac: String,
    pub vendor: String,
    pub model: Option<String>,
    pub label: Option<String>,
    pub enabled: bool,
    pub domain_id: Uuid,
    pub extension_uuid: Option<Uuid>,
    pub device_profile_uuid: Option<Uuid>,
    pub device_address: Option<String>,
    pub device_label: Option<String>,
    pub device_vendor: Option<String>,
    pub device_model: Option<String>,
    pub device_firmware_version: Option<String>,
    pub device_template: Option<String>,
    pub device_location: Option<String>,
    pub device_serial_number: Option<String>,
    pub device_description: Option<String>,
    pub device_username: Option<String>,
    pub device_password: Option<String>,
    pub device_provisioned_date: Option<DateTime<Utc>>,
    pub device_provisioned_ip: Option<String>,
    pub device_provisioned_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
