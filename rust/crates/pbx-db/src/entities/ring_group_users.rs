use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ring_group_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ring_group_user_uuid: String,
    pub domain_id: Uuid,
    pub ring_group_uuid: String,
    pub user_uuid: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
