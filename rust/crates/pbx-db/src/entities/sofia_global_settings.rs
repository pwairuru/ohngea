//! `sofia_global_settings` entity — exact columns from
//! app/sofia_global_settings/models.py. Global table: no domain_id.
//! UNIQUE: global_setting_name (alembic 016).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sofia_global_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sofia_global_setting_uuid: Uuid,
    pub global_setting_name: String,
    pub global_setting_value: String,
    pub global_setting_enabled: bool,
    pub global_setting_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
