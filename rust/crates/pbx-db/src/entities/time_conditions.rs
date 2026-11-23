//! `time_conditions` — columns from app/time_conditions/models.py (alembic 004).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "time_conditions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub time_condition_uuid: String,
    pub domain_id: Option<Uuid>,
    pub name: String,
    pub extension: String,
    pub context: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub dialplan_uuid: Option<String>,
    pub time_hour: Option<String>,
    pub time_minute: Option<String>,
    pub time_wday: Option<String>,
    pub time_mday: Option<String>,
    pub time_mon: Option<String>,
    pub time_year: Option<String>,
    pub time_yday: Option<String>,
    pub time_week: Option<String>,
    pub time_mweek: Option<String>,
    pub minute_of_day: Option<String>,
    pub time_of_day: Option<String>,
    pub date_time: Option<String>,
    pub condition: Option<String>,
    pub true_destination: Option<String>,
    pub false_destination: Option<String>,
    pub timezone: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
