//! `contacts` entity — core columns from app/contacts/models.py `ContactBase`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "contacts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub contact_uuid: Uuid,
    pub domain_id: Uuid,
    pub name: String,
    pub number: String,
    pub email: Option<String>,
    pub organization: Option<String>,
    pub contact_organization: Option<String>,
    pub contact_type: Option<String>,
    pub contact_parent_uuid: Option<Uuid>,
    pub contact_name_prefix: Option<String>,
    pub contact_name_given: Option<String>,
    pub contact_name_middle: Option<String>,
    pub contact_name_family: Option<String>,
    pub contact_name_suffix: Option<String>,
    pub contact_nickname: Option<String>,
    pub contact_title: Option<String>,
    pub contact_role: Option<String>,
    pub contact_category: Option<String>,
    pub contact_url: Option<String>,
    pub contact_time_zone: Option<String>,
    pub contact_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
