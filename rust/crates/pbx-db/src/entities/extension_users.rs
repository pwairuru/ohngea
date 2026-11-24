use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "extension_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub extension_user_uuid: String,
    pub domain_id: Uuid,
    pub extension_uuid: String,
    pub user_uuid: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
