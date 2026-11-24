use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_vendor_functions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_vendor_function_uuid: String,
    pub device_vendor_uuid: String,
    pub label: Option<String>,
    pub subtype: Option<String>,
    pub value: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
