//! `dialplans` entity — exact columns from app/dialplans/models.py:10-64.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dialplans")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub dialplan_uuid: Uuid,
    pub name: String,
    pub context: String,
    pub enabled: bool,
    pub domain_id: Uuid,
    pub description: Option<String>,
    pub dialplan_continue: bool,
    pub dialplan_order: Option<i32>,
    pub hostname: Option<String>,
    pub app_uuid: Option<String>,
    pub dialplan_number: Option<String>,
    pub dialplan_destination: Option<bool>,
    pub dialplan_xml: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
