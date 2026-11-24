//! `databases` — v_databases external DB connection registry.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "databases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub database_uuid: Uuid,
    pub database_driver: Option<String>,
    pub database_type: Option<String>,
    pub database_host: Option<String>,
    pub database_port: Option<String>,
    pub database_name: Option<String>,
    pub database_username: Option<String>,
    pub database_password: Option<String>,
    pub database_path: Option<String>,
    pub database_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
