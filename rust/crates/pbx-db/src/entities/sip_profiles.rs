//! `sip_profiles` entity — exact columns from app/sip_profiles/models.py:10-43.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sip_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sip_profile_uuid: Uuid,
    pub name: String,
    pub enabled: bool,
    pub domain_id: Uuid,
    pub sip_port: i32,
    pub sip_ip: Option<String>,
    pub ext_sip_ip: Option<String>,
    pub ext_rtp_ip: Option<String>,
    pub realm: Option<String>,
    pub context: Option<String>,
    pub challenge_realm: Option<String>,
    pub wss_port: Option<i32>,
    pub wss_binding: Option<String>,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub sip_secure: Option<bool>,
    pub rtp_ip: Option<String>,
    pub rtp_port_range: Option<String>,
    pub hold_music: Option<String>,
    pub apply_inbound_acl: Option<String>,
    pub local_network_acl: Option<String>,
    pub record_path: Option<String>,
    pub record_template: Option<String>,
    pub description: Option<String>,
    pub hostname: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
