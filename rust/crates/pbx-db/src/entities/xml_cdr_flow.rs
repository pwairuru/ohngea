use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "xml_cdr_flow")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub xml_cdr_flow_uuid: String,
    pub xml_cdr_uuid: String,
    pub domain_id: Uuid,
    pub call_flow: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
