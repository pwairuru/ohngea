use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "database_transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub database_transaction_uuid: String,
    pub domain_id: Uuid,
    pub user_uuid: Option<String>,
    pub app_name: Option<String>,
    pub app_uuid: Option<String>,
    pub transaction_code: Option<String>,
    pub transaction_address: Option<String>,
    pub transaction_type: Option<String>,
    pub transaction_date: Option<DateTime<Utc>>,
    pub transaction_old: Option<String>,
    pub transaction_new: Option<String>,
    pub transaction_result: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
