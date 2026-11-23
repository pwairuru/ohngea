//! `gateways` entity — exact columns from app/gateways/models.py:11-62.
//!
//! The DB column stays `register`; the Rust field is `gateway_register`
//! with a `column_name` override — the same trick Python uses with
//! `sa_column=Column("register", ...)` to avoid shadowing `register`.
//! Default helpers live in [`crate::gateways`].

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "gateways")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub gateway_uuid: Uuid,
    pub gateway: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub proxy: Option<String>,
    #[sea_orm(column_name = "register")]
    pub gateway_register: bool,
    pub enabled: bool,
    pub domain_id: Option<Uuid>,
    pub distinct_to: Option<bool>,
    pub auth_username: Option<String>,
    pub realm: Option<String>,
    pub from_user: Option<String>,
    pub from_domain: Option<String>,
    pub register_proxy: Option<String>,
    pub outbound_proxy: Option<String>,
    pub expire_seconds: Option<i32>,
    pub register_transport: Option<String>,
    pub contact_params: Option<String>,
    pub retry_seconds: Option<i32>,
    pub extension: Option<String>,
    pub ping: Option<String>,
    pub ping_min: Option<i32>,
    pub ping_max: Option<i32>,
    pub contact_in_ping: Option<bool>,
    pub caller_id_in_from: Option<bool>,
    pub supress_cng: Option<bool>,
    pub sip_cid_type: Option<String>,
    pub codec_prefs: Option<String>,
    pub channels: Option<i32>,
    pub extension_in_contact: Option<String>,
    pub context: Option<String>,
    pub profile: Option<String>,
    pub hostname: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
