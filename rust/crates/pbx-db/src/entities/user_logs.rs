//! `user_logs` — superset of PHP `v_user_logs` + Python `UserLog`.
//! PHP edit form: username/type/result/remote_address/user_agent/timestamp.
//! Python model: username/action/detail/ip_address/created_at.
//! Stored canonical: PHP names + Python aliases via `alias.rs` + serializer.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_log_uuid: String,
    pub domain_id: Option<Uuid>,
    pub user_uuid: Option<String>,
    pub username: Option<String>,
    pub log_type: Option<String>,
    pub result: Option<String>,
    pub detail: Option<String>,
    pub remote_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
