use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_control_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_control_detail_uuid: String,
    pub conference_control_uuid: String,
    pub domain_id: Uuid,
    pub control_digits: Option<String>,
    pub control_action: Option<String>,
    pub control_data: Option<String>,
    pub control_enabled: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
