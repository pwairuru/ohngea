//! `call_block` — columns from app/call_block/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "call_block")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub call_block_uuid: String,
    pub domain_id: Uuid,
    pub call_block_direction: Option<String>,
    pub extension_uuid: Option<String>,
    pub call_block_name: String,
    pub call_block_country_code: Option<String>,
    pub call_block_number: String,
    pub call_block_enabled: bool,
    pub call_block_description: Option<String>,
    pub call_block_action: Option<String>,
    pub call_block_count: Option<i32>,
    pub call_block_app: Option<String>,
    pub call_block_data: Option<String>,
    pub date_added: Option<DateTime<Utc>>,
    pub insert_user: Option<String>,
    pub update_user: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
