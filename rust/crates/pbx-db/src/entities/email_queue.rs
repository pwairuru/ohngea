//! `email_queue` entity — core columns from app/email_queue/models.py.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "email_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub email_queue_uuid: Uuid,
    pub domain_id: Uuid,
    pub email_date: Option<DateTime<Utc>>,
    pub email_from: String,
    pub email_to: String,
    pub email_subject: Option<String>,
    pub email_body: Option<String>,
    pub email_status: String,
    pub email_retry_count: i32,
    pub email_attempts: i32,
    pub email_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
