//! `access_control_nodes` — columns from app/access_controls/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "access_control_nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub access_control_node_uuid: String,
    pub access_control_uuid: String,
    pub domain_id: Uuid,
    pub node_type: String,
    pub node_cidr: String,
    pub node_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
