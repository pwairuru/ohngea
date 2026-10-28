//! `webrtc_settings` — columns from app/webrtc_settings/models.py (alembic 020).
//! PK is `domain_id` (one row per tenant); `ice_servers` is JSON.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "webrtc_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub domain_id: Uuid,
    pub enabled: bool,
    pub auth_mode: String,
    pub wss_url: Option<String>,
    pub sip_domain: Option<String>,
    pub ice_servers: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
