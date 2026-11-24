use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "xml_cdr_extensions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub xml_cdr_extension_uuid: String,
    pub domain_id: Uuid,
    pub xml_cdr_uuid: String,
    pub extension_uuid: Option<String>,
    pub start_stamp: Option<DateTime<Utc>>,
    pub end_stamp: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
