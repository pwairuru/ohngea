//! `vars` entity — exact columns from app/vars/models.py + alembic 029
//! (var_command, var_hostname). Dedupe key: (domain_id, var_name, var_category).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vars")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub var_uuid: Uuid,
    pub domain_id: Uuid,
    pub var_category: String,
    pub var_name: String,
    pub var_value: String,
    pub var_enabled: bool,
    pub var_description: Option<String>,
    pub var_order: Option<i32>,
    pub var_command: Option<String>,
    pub var_hostname: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
