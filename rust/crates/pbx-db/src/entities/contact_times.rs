//! `contact_times` — v_contact_times (availability windows on a contact).

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contact_times")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_time_uuid: Uuid,
    pub domain_id: Uuid,
    pub contact_uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub time_start: Option<String>,
    pub time_stop: Option<String>,
    pub time_description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
