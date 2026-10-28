//! `fax_logs` — app/fax/models.py (core columns).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fax_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub fax_log_uuid: String,
    pub domain_id: Uuid,
    pub fax_uuid: String,
    pub fax_result_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
