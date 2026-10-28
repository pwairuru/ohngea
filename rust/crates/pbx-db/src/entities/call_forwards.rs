//! `call_forwards` — columns from app/call_forward/models.py (alembic 027).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_forwards")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_forward_uuid: String,
    pub domain_id: Uuid,
    pub extension_uuid: Option<String>,
    pub forward_all_destination: Option<String>,
    pub forward_all_enabled: bool,
    pub forward_busy_destination: Option<String>,
    pub forward_busy_enabled: bool,
    pub forward_no_answer_destination: Option<String>,
    pub forward_no_answer_enabled: bool,
    pub forward_user_not_registered_destination: Option<String>,
    pub forward_user_not_registered_enabled: bool,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
