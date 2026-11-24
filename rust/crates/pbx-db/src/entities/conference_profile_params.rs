use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "conference_profile_params")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conference_profile_param_uuid: String,
    pub conference_profile_uuid: String,
    pub domain_id: Uuid,
    pub profile_param_name: Option<String>,
    pub profile_param_value: Option<String>,
    pub profile_param_enabled: Option<bool>,
    pub profile_param_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
